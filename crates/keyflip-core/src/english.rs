//! Ranked English word list: breaks ties on short ambiguous words, decides
//! where Chinese splits off a word, and completes words (TP-5).

use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct English {
    rank: HashMap<String, u32>,
    by_rank: Vec<String>,
}

impl English {
    pub fn bundled() -> English {
        English::from_list(include_str!("../../../data/english.txt"))
    }

    /// One lowercase word per line, most common first.
    pub fn from_list(list: &str) -> English {
        let mut rank = HashMap::new();
        let mut by_rank = Vec::new();
        for (i, word) in list.lines().enumerate() {
            let word = word.trim();
            if !word.is_empty() && !rank.contains_key(word) {
                rank.insert(word.to_owned(), i as u32);
                by_rank.push(word.to_owned());
            }
        }
        English { rank, by_rank }
    }

    /// Words in rank order.
    pub fn words_by_rank(&self) -> Vec<&str> {
        let mut v: Vec<(&String, &u32)> = self.rank.iter().collect();
        v.sort_by_key(|(_, r)| **r);
        v.into_iter().map(|(w, _)| w.as_str()).collect()
    }

    /// The rest of the best word among the first `limit` that extends `prefix` by
    /// at least two letters; a shorter listed base of that word wins (TP-5).
    pub fn complete(&self, prefix: &str, limit: usize) -> Option<&str> {
        let fits = |w: &&String| w.len() >= prefix.len() + 2 && w.starts_with(prefix);
        let top = &self.by_rank[..limit.min(self.by_rank.len())];
        let best = top.iter().find(fits)?;
        let base = top
            .iter()
            .filter(fits)
            .filter(|w| best.starts_with(w.as_str()))
            .min_by_key(|w| w.len())
            .unwrap_or(best);
        Some(&base[prefix.len()..])
    }

    /// 0 is the most common word. `None` if unknown.
    pub fn rank(&self, word: &str) -> Option<u32> {
        self.rank.get(word).copied()
    }
}
