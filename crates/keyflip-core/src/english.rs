//! Ranked English word list, used only to break ties on short ambiguous words.

use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct English {
    rank: HashMap<String, u32>,
}

impl English {
    pub fn bundled() -> English {
        English::from_list(include_str!("../../../data/english.txt"))
    }

    /// One lowercase word per line, most common first.
    pub fn from_list(list: &str) -> English {
        let mut rank = HashMap::new();
        for (i, word) in list.lines().enumerate() {
            let word = word.trim();
            if !word.is_empty() {
                rank.entry(word.to_owned()).or_insert(i as u32);
            }
        }
        English { rank }
    }

    /// Words in rank order.
    pub fn words_by_rank(&self) -> Vec<&str> {
        let mut v: Vec<(&String, &u32)> = self.rank.iter().collect();
        v.sort_by_key(|(_, r)| **r);
        v.into_iter().map(|(w, _)| w.as_str()).collect()
    }

    /// 0 is the most common word. `None` if unknown.
    pub fn rank(&self, word: &str) -> Option<u32> {
        self.rank.get(word).copied()
    }
}
