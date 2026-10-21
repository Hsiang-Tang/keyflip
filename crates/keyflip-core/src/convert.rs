//! Turn a run of zhuyin syllables into Chinese text (best path over the lexicon)
//! and list alternatives for one syllable.

use crate::lexicon::{Entry, Lexicon};
use crate::user::{UserModel, USER_COUNT};

/// One syllable of a run: its reading and, if the user picked a character, that choice.
#[derive(Debug, Clone)]
pub struct Unit {
    pub reading: String,
    pub fixed: Option<String>,
}

fn matches_at(entry: &Entry, units: &[Unit], start: usize) -> bool {
    if start + entry.readings.len() > units.len() {
        return false;
    }
    entry
        .readings
        .iter()
        .zip(&units[start..])
        .all(|(r, u)| *r == u.reading)
}

fn consistent_with_fixed(entry: &Entry, units: &[Unit], start: usize) -> bool {
    entry
        .text
        .chars()
        .zip(&units[start..])
        .all(|(ch, u)| match &u.fixed {
            None => true,
            Some(f) => f.chars().eq(std::iter::once(ch)),
        })
}

fn entries_from<'a>(
    lex: &'a Lexicon,
    user: Option<&'a UserModel>,
    reading: &str,
) -> impl Iterator<Item = &'a Entry> {
    let phrases = user.map(|u| u.phrases_from(reading)).unwrap_or(&[]);
    lex.entries_from(reading).iter().chain(phrases)
}

/// Cost per syllable where an entry disagrees with a learned preference.
fn mismatch_penalty() -> f64 {
    (USER_COUNT as f64).ln()
}

/// One Chinese character per unit, chosen by the best segmentation. Entries
/// agreeing with learned preferences count as `USER_COUNT` (UL-3, UL-4).
pub fn convert(lex: &Lexicon, user: Option<&UserModel>, units: &[Unit]) -> Vec<String> {
    let n = units.len();
    if n == 0 {
        return Vec::new();
    }
    let readings: Vec<&str> = units.iter().map(|u| u.reading.as_str()).collect();
    let prefs = user.map(|u| u.preferences(&readings)).unwrap_or_else(|| vec![None; n]);
    let mut best = vec![f64::NEG_INFINITY; n + 1];
    let mut back: Vec<Option<(usize, String)>> = vec![None; n + 1];
    best[0] = 0.0;
    for i in 0..n {
        if best[i] == f64::NEG_INFINITY {
            continue;
        }
        for entry in entries_from(lex, user, &units[i].reading) {
            if !matches_at(entry, units, i) || !consistent_with_fixed(entry, units, i) {
                continue;
            }
            let j = i + entry.readings.len();
            let (mut agree, mut disagree) = (0, 0);
            for (ch, pref) in entry.text.chars().zip(&prefs[i..j]) {
                match pref {
                    Some(p) if *p == ch => agree += 1,
                    Some(_) => disagree += 1,
                    None => {}
                }
            }
            let count = if agree > 0 && disagree == 0 {
                entry.count.max(USER_COUNT)
            } else {
                entry.count
            };
            let score = best[i] + lex.score(count) - disagree as f64 * mismatch_penalty();
            if score > best[j] {
                best[j] = score;
                back[j] = Some((i, entry.text.clone()));
            }
        }
    }
    let mut out = vec![String::new(); n];
    let mut j = n;
    while j > 0 {
        match &back[j] {
            Some((i, text)) => {
                for (k, ch) in text.chars().enumerate() {
                    out[i + k] = ch.to_string();
                }
                j = *i;
            }
            None => {
                // No path: show the reading itself for the remaining units.
                for k in 0..j {
                    if out[k].is_empty() {
                        out[k] = units[k].reading.clone();
                    }
                }
                break;
            }
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub text: String,
    /// How many units this candidate covers, starting at the target.
    pub len: usize,
}

/// Alternatives for the unit at `target`: learned choices (UL-2), then words,
/// then single characters, each most frequent first.
pub fn candidates(
    lex: &Lexicon,
    user: Option<&UserModel>,
    units: &[Unit],
    target: usize,
) -> Vec<Candidate> {
    let mut learned = Vec::new();
    let mut words = Vec::new();
    let mut chars = Vec::new();
    let mut seen = std::collections::HashSet::new();
    if let Some(u) = user {
        let readings: Vec<&str> = units[target..].iter().map(|u| u.reading.as_str()).collect();
        for (text, len) in u.learned_for(&readings) {
            if seen.insert(text.clone()) {
                learned.push(Candidate { text, len });
            }
        }
    }
    let phrases = user.map(|u| u.phrases_from(&units[target].reading)).unwrap_or(&[]);
    for entry in phrases.iter().chain(lex.entries_from(&units[target].reading)) {
        if !matches_at(entry, units, target) {
            continue;
        }
        if !seen.insert(entry.text.clone()) {
            continue;
        }
        let cand = Candidate {
            text: entry.text.clone(),
            len: entry.readings.len(),
        };
        if cand.len > 1 {
            words.push(cand);
        } else {
            chars.push(cand);
        }
    }
    learned.extend(words);
    learned.extend(chars);
    learned
}
