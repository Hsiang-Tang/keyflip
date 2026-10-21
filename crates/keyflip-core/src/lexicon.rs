//! Zhuyin lexicon: which key sequences are valid syllables, and which words
//! a run of syllables can become.

use std::collections::{HashMap, HashSet};

/// One lexicon entry: the readings (one per character), the text, and a count.
#[derive(Debug, Clone)]
pub struct Entry {
    pub readings: Vec<String>,
    pub text: String,
    pub count: u32,
}

#[derive(Debug, Default)]
pub struct Lexicon {
    /// Symbol prefixes (tone mark stripped) of every valid syllable.
    prefixes: HashSet<String>,
    /// Every valid single-character reading, tone mark included (first tone has none).
    syllables: HashSet<String>,
    /// Entries grouped by their first reading, highest count first.
    by_first: HashMap<String, Vec<Entry>>,
    total: f64,
}

const TONE_MARKS: [char; 4] = ['ˊ', 'ˇ', 'ˋ', '˙'];

pub fn strip_tone(reading: &str) -> &str {
    reading.trim_end_matches(TONE_MARKS)
}

impl Lexicon {
    /// The lexicon compiled into the binary.
    pub fn bundled() -> Lexicon {
        Lexicon::from_tsv(include_str!("../../../data/lexicon.tsv"))
    }

    /// Parse `readings<TAB>text<TAB>count` lines. Malformed lines are skipped.
    pub fn from_tsv(tsv: &str) -> Lexicon {
        let mut lex = Lexicon::default();
        let mut total = 0f64;
        for line in tsv.lines() {
            let mut parts = line.split('\t');
            let (Some(readings), Some(text), Some(count)) =
                (parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            let Ok(count) = count.trim().parse::<u32>() else {
                continue;
            };
            let readings: Vec<String> = readings.split(' ').map(str::to_owned).collect();
            if readings.is_empty() || text.chars().count() != readings.len() {
                continue;
            }
            total += count as f64;
            if readings.len() == 1 {
                let r = &readings[0];
                lex.syllables.insert(r.clone());
                let base = strip_tone(r);
                for (i, _) in base.char_indices() {
                    lex.prefixes.insert(base[..i].to_owned());
                }
                lex.prefixes.insert(base.to_owned());
            }
            lex.by_first
                .entry(readings[0].clone())
                .or_default()
                .push(Entry {
                    readings,
                    text: text.to_owned(),
                    count,
                });
        }
        for entries in lex.by_first.values_mut() {
            entries.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.text.cmp(&b.text)));
        }
        lex.total = total.max(1.0);
        lex
    }

    /// Can `symbols` (no tone) still grow into a valid syllable?
    pub fn is_prefix(&self, symbols: &str) -> bool {
        self.prefixes.contains(symbols)
    }

    /// Is `reading` (tone mark included) a valid syllable?
    pub fn is_syllable(&self, reading: &str) -> bool {
        self.syllables.contains(reading)
    }

    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.by_first.values().flatten()
    }

    pub fn entries_from(&self, first_reading: &str) -> &[Entry] {
        self.by_first
            .get(first_reading)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Highest count among single characters with this reading.
    pub fn best_char_count(&self, reading: &str) -> u32 {
        self.entries_from(reading)
            .iter()
            .filter(|e| e.readings.len() == 1)
            .map(|e| e.count)
            .max()
            .unwrap_or(0)
    }

    /// Log-probability style score of an entry (unigram, smoothed).
    pub fn score(&self, count: u32) -> f64 {
        ((count as f64) + 0.5).ln() - self.total.ln()
    }
}
