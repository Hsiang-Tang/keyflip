//! Turn a run of zhuyin syllables into Chinese text (best path over the lexicon)
//! and list alternatives for one syllable.

use crate::lexicon::{Entry, Lexicon};

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

/// One Chinese character per unit, chosen by the best segmentation.
pub fn convert(lex: &Lexicon, units: &[Unit]) -> Vec<String> {
    let n = units.len();
    if n == 0 {
        return Vec::new();
    }
    let mut best = vec![f64::NEG_INFINITY; n + 1];
    let mut back: Vec<Option<(usize, String)>> = vec![None; n + 1];
    best[0] = 0.0;
    for i in 0..n {
        if best[i] == f64::NEG_INFINITY {
            continue;
        }
        for entry in lex.entries_from(&units[i].reading) {
            if !matches_at(entry, units, i) || !consistent_with_fixed(entry, units, i) {
                continue;
            }
            let j = i + entry.readings.len();
            let score = best[i] + lex.score(entry.count);
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

/// Alternatives for the unit at `target`: multi-character words that start
/// there first, then single characters, each most frequent first.
pub fn candidates(lex: &Lexicon, units: &[Unit], target: usize) -> Vec<Candidate> {
    let mut words = Vec::new();
    let mut chars = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for entry in lex.entries_from(&units[target].reading) {
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
    words.extend(chars);
    words
}
