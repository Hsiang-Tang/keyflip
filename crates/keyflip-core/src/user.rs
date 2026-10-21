//! User phrases and learned choices (004-user-lexicon). No I/O (UL-10).

use std::collections::HashMap;

use crate::lexicon::Entry;

/// Count given to user phrases and learned preferences; above 的 (615,175).
pub const USER_COUNT: u32 = 1_000_000;
/// Learned choices kept, least recently used dropped first (UL-7).
pub const LEARNED_CAP: usize = 5_000;
/// Picks after which a single character is preferred in any context (UL-3).
const PICKS_FOR_ANYWHERE: u32 = 3;
/// English words kept (UL-12).
pub const ENGLISH_CAP: usize = 2_000;
/// Uses after which an English word is suggested (UL-11).
const USES_FOR_COMPLETION: u32 = 2;
pub const ENGLISH_LEN: std::ops::RangeInclusive<usize> = 5..=20;

const ENGLISH_HEADER: &str = "\
# keyflip learned English words for completion, written by keyflip. A word is
# suggested once it has been typed twice. Delete a line to forget it.
# word\ttimes typed\tlast use (order only)
";

pub const PHRASES_HEADER: &str = "\
# keyflip user phrases. One word per line: text, a Tab, then its readings
# separated by spaces. Lines starting with # are ignored. Example:
# 鍵翻\tㄐㄧㄢˋ ㄈㄢ
";

const LEARNED_HEADER: &str = "\
# keyflip learned choices, written by keyflip. Delete a line to forget it,
# or delete this file to forget everything.
# readings\ttext\tprevious syllable\ttimes picked\tlast use (order only)
";

#[derive(Debug, Clone, Copy)]
struct Use {
    count: u32,
    last: u64,
}

#[derive(Debug, Default)]
pub struct UserModel {
    /// By first reading.
    phrases: HashMap<String, Vec<Entry>>,
    /// (readings joined by spaces, text) -> context reading -> use.
    learned: HashMap<(String, String), HashMap<String, Use>>,
    english: HashMap<String, Use>,
    seq: u64,
    generation: u64,
}

fn valid_reading(r: &str) -> bool {
    !r.is_empty()
        && r.chars().any(|c| ('ㄅ'..='ㄩ').contains(&c))
        && r.chars().all(|c| ('ㄅ'..='ㄩ').contains(&c) || "ˊˇˋ˙".contains(c))
}

fn content_lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .map(|l| l.trim_end_matches('\r'))
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
}

impl UserModel {
    /// Returns the number of malformed lines, which are skipped.
    pub fn load_phrases(&mut self, text: &str) -> usize {
        self.phrases.clear();
        let mut bad = 0;
        for line in content_lines(text) {
            let Some((word, readings)) = line.split_once('\t') else {
                bad += 1;
                continue;
            };
            let word = word.trim();
            let readings: Vec<String> = readings.split_whitespace().map(str::to_owned).collect();
            if readings.is_empty()
                || word.chars().count() != readings.len()
                || !readings.iter().all(|r| valid_reading(r))
            {
                bad += 1;
                continue;
            }
            let list = self.phrases.entry(readings[0].clone()).or_default();
            if !list.iter().any(|e| e.text == word && e.readings == readings) {
                list.push(Entry {
                    readings,
                    text: word.to_owned(),
                    count: USER_COUNT,
                    rare: false,
                });
            }
        }
        self.generation += 1;
        bad
    }

    /// Returns the number of malformed lines, which are skipped.
    pub fn load_learned(&mut self, text: &str) -> usize {
        self.learned.clear();
        self.seq = 0;
        let mut bad = 0;
        for line in content_lines(text) {
            let f: Vec<&str> = line.split('\t').collect();
            let parsed = match f[..] {
                [readings, text, context, count, last] => {
                    let rs: Vec<&str> = readings.split(' ').collect();
                    let well_formed = rs.iter().all(|r| valid_reading(r))
                        && text.chars().count() == rs.len()
                        && (context.is_empty() || valid_reading(context));
                    match (well_formed, count.parse::<u32>(), last.parse::<u64>()) {
                        (true, Ok(count), Ok(last)) if count > 0 => Some((count, last)),
                        _ => None,
                    }
                }
                _ => None,
            };
            let Some((count, last)) = parsed else {
                bad += 1;
                continue;
            };
            self.seq = self.seq.max(last);
            self.learned
                .entry((f[0].to_owned(), f[1].to_owned()))
                .or_default()
                .insert(f[2].to_owned(), Use { count, last });
        }
        self.evict();
        self.generation += 1;
        bad
    }

    /// Oldest first.
    pub fn learned_tsv(&self) -> String {
        let mut rows: Vec<(u64, String)> = Vec::new();
        for ((readings, text), contexts) in &self.learned {
            for (context, u) in contexts {
                rows.push((u.last, format!("{readings}\t{text}\t{context}\t{}\t{}\n", u.count, u.last)));
            }
        }
        rows.sort();
        let mut out = String::from(LEARNED_HEADER);
        for (_, row) in rows {
            out.push_str(&row);
        }
        out
    }

    /// Records one committed pick (UL-1).
    pub fn learn(&mut self, readings: &[String], text: &str, context: &str) {
        self.seq += 1;
        let u = self
            .learned
            .entry((readings.join(" "), text.to_owned()))
            .or_default()
            .entry(context.to_owned())
            .or_insert(Use { count: 0, last: 0 });
        u.count = u.count.saturating_add(1);
        u.last = self.seq;
        self.evict();
        self.generation += 1;
    }

    /// Forgets learned choices and English words (UL-6).
    pub fn clear_learned(&mut self) {
        self.learned.clear();
        self.english.clear();
        self.seq = 0;
        self.generation += 1;
    }

    /// Counts a committed English word if it qualifies (UL-11).
    pub fn learn_word(&mut self, word: &str) {
        if !ENGLISH_LEN.contains(&word.len()) || !word.bytes().all(|b| b.is_ascii_lowercase()) {
            return;
        }
        self.seq += 1;
        let u = self.english.entry(word.to_owned()).or_insert(Use { count: 0, last: 0 });
        u.count = u.count.saturating_add(1);
        u.last = self.seq;
        self.evict_english();
        self.generation += 1;
    }

    /// The rest of the most used qualifying word extending `prefix` (UL-11).
    pub fn complete(&self, prefix: &str) -> Option<&str> {
        self.english
            .iter()
            .filter(|(w, u)| {
                u.count >= USES_FOR_COMPLETION && w.len() >= prefix.len() + 2 && w.starts_with(prefix)
            })
            .max_by(|a, b| a.1.count.cmp(&b.1.count).then(a.1.last.cmp(&b.1.last)))
            .map(|(w, _)| &w[prefix.len()..])
    }

    pub fn english_len(&self) -> usize {
        self.english.len()
    }

    /// Returns the number of malformed lines, which are skipped.
    pub fn load_english(&mut self, text: &str) -> usize {
        self.english.clear();
        let mut bad = 0;
        for line in content_lines(text) {
            let f: Vec<&str> = line.split('\t').collect();
            let parsed = match f[..] {
                [word, count, last]
                    if ENGLISH_LEN.contains(&word.len())
                        && word.bytes().all(|b| b.is_ascii_lowercase()) =>
                {
                    match (count.parse::<u32>(), last.parse::<u64>()) {
                        (Ok(count), Ok(last)) if count > 0 => Some((word, count, last)),
                        _ => None,
                    }
                }
                _ => None,
            };
            let Some((word, count, last)) = parsed else {
                bad += 1;
                continue;
            };
            self.seq = self.seq.max(last);
            self.english.insert(word.to_owned(), Use { count, last });
        }
        self.evict_english();
        self.generation += 1;
        bad
    }

    /// Oldest first.
    pub fn english_tsv(&self) -> String {
        let mut rows: Vec<(u64, &String, u32)> =
            self.english.iter().map(|(w, u)| (u.last, w, u.count)).collect();
        rows.sort();
        let mut out = String::from(ENGLISH_HEADER);
        for (last, w, count) in rows {
            out.push_str(&format!("{w}\t{count}\t{last}\n"));
        }
        out
    }

    /// Bumped on every change, so the caller knows when to save.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn learned_len(&self) -> usize {
        self.learned.values().map(HashMap::len).sum()
    }

    fn evict_english(&mut self) {
        while self.english.len() > ENGLISH_CAP {
            let oldest = self.english.iter().min_by_key(|(_, u)| u.last).map(|(w, _)| w.clone());
            self.english.remove(&oldest.expect("not empty"));
        }
    }

    fn evict(&mut self) {
        let n = self.learned_len();
        if n <= LEARNED_CAP {
            return;
        }
        let mut lasts: Vec<u64> = self
            .learned
            .values()
            .flat_map(|c| c.values().map(|u| u.last))
            .collect();
        lasts.sort_unstable();
        let cutoff = lasts[n - LEARNED_CAP];
        for contexts in self.learned.values_mut() {
            contexts.retain(|_, u| u.last >= cutoff);
        }
        self.learned.retain(|_, c| !c.is_empty());
    }

    pub fn phrases_from(&self, reading: &str) -> &[Entry] {
        self.phrases.get(reading).map(Vec::as_slice).unwrap_or(&[])
    }

    /// The preferred character at each syllable of a run (UL-3). Where learned
    /// choices overlap, the most recent wins.
    pub fn preferences(&self, readings: &[&str]) -> Vec<Option<char>> {
        let mut prefs = vec![None; readings.len()];
        if self.learned.is_empty() {
            return prefs;
        }
        let mut found: Vec<(u64, usize, &str)> = Vec::new();
        for ((joined, text), contexts) in &self.learned {
            let rs: Vec<&str> = joined.split(' ').collect();
            let last = contexts.values().map(|u| u.last).max().unwrap_or(0);
            let total: u32 = contexts.values().map(|u| u.count).sum();
            for p in 0..readings.len().saturating_sub(rs.len() - 1) {
                if readings[p..p + rs.len()] != rs[..] {
                    continue;
                }
                let context = if p > 0 { readings[p - 1] } else { "" };
                if rs.len() > 1 || total >= PICKS_FOR_ANYWHERE {
                    found.push((last, p, text));
                } else if let Some(u) = contexts.get(context) {
                    found.push((u.last, p, text));
                }
            }
        }
        found.sort();
        for (_, p, text) in found {
            for (k, ch) in text.chars().enumerate() {
                prefs[p + k] = Some(ch);
            }
        }
        prefs
    }

    /// Learned (text, syllables covered) starting at `readings[0]`, most recent
    /// first (UL-2).
    pub fn learned_for(&self, readings: &[&str]) -> Vec<(String, usize)> {
        let mut found: Vec<(u64, String, usize)> = Vec::new();
        for ((joined, text), contexts) in &self.learned {
            let rs: Vec<&str> = joined.split(' ').collect();
            if rs.len() <= readings.len() && rs[..] == readings[..rs.len()] {
                let last = contexts.values().map(|u| u.last).max().unwrap_or(0);
                found.push((last, text.clone(), rs.len()));
            }
        }
        found.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        found.into_iter().map(|(_, t, n)| (t, n)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rs(s: &str) -> Vec<String> {
        s.split(' ').map(str::to_owned).collect()
    }

    #[test]
    fn malformed_learned_lines_are_skipped_and_counted() {
        let mut m = UserModel::default();
        let text = "# header\n\
                    ㄉㄜ˙\t地\tㄇㄢˋ\t2\t7\n\
                    ㄉㄜ˙\t地\tㄇㄢˋ\t2\n\
                    abc\t地\t\t1\t1\n\
                    ㄉㄜ˙\t地地\t\t1\t1\n\
                    ㄉㄜ˙\t地\t\tx\t1\n\
                    ㄉㄜ˙\t地\t\t0\t1\n\
                    \n";
        assert_eq!(m.load_learned(text), 5);
        assert_eq!(m.learned_len(), 1);
    }

    #[test]
    fn learned_text_round_trips() {
        let mut m = UserModel::default();
        m.learn(&rs("ㄉㄜ˙"), "地", "ㄇㄢˋ");
        m.learn(&rs("ㄋㄧˇ ㄏㄠˇ"), "擬好", "");
        m.learn(&rs("ㄉㄜ˙"), "地", "ㄇㄢˋ");
        let text = m.learned_tsv();
        let mut n = UserModel::default();
        assert_eq!(n.load_learned(&text), 0);
        assert_eq!(n.learned_tsv(), text);
        assert!(text.contains("ㄉㄜ˙\t地\tㄇㄢˋ\t2\t3\n"), "{text}");
    }

    #[test]
    fn the_cap_drops_the_least_recently_used() {
        let mut m = UserModel::default();
        let chars: Vec<char> = ('一'..).take(LEARNED_CAP + 10).collect();
        for c in &chars {
            m.learn(&rs("ㄧ"), &c.to_string(), "");
        }
        assert_eq!(m.learned_len(), LEARNED_CAP);
        let kept = m.learned_for(&["ㄧ"]);
        assert!(!kept.iter().any(|(t, _)| *t == chars[0].to_string()), "oldest dropped");
        assert_eq!(kept[0].0, chars[LEARNED_CAP + 9].to_string(), "newest first");
    }

    #[test]
    fn english_words_need_two_uses_and_round_trip() {
        let mut m = UserModel::default();
        for w in ["docker", "docker", "doc", "Docker", "dock3r", "documentation"] {
            m.learn_word(w);
        }
        assert_eq!(m.english_len(), 2);
        assert_eq!(m.complete("doc"), Some("ker"));
        assert_eq!(m.complete("docum"), None, "typed once");
        let text = m.english_tsv();
        let mut n = UserModel::default();
        assert_eq!(n.load_english(&format!("{text}bad line\nab\t1\t1\n")), 2);
        assert_eq!(n.english_tsv(), text);
    }

    #[test]
    fn phrases_need_one_reading_per_character() {
        let mut m = UserModel::default();
        let bad = m.load_phrases("# c\n鍵翻\tㄐㄧㄢˋ ㄈㄢ\n鍵\tㄐㄧㄢˋ ㄈㄢ\n鍵翻 ㄐㄧㄢˋ ㄈㄢ\n鍵翻\tab cd\n");
        assert_eq!(bad, 3);
        assert_eq!(m.phrases_from("ㄐㄧㄢˋ").len(), 1);
    }
}
