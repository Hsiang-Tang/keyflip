//! The keyflip engine: one key in, one full snapshot out.
//!
//! Language is never switched by a modifier tap. Every key is read as zhuyin
//! unless the typed sequence can no longer be zhuyin (see specs/001-mixed-input/plan.md).

use std::sync::Arc;

use crate::convert::{self, Candidate, Unit};
use crate::english::English;
use crate::key::{KeyCode, KeyEvent};
use crate::lexicon::Lexicon;
use crate::snapshot::{Candidates, SegKind, Segment, Snapshot};
use crate::zhuyin;

const PAGE_SIZE: usize = 9;

#[derive(Debug, Clone)]
pub struct Options {
    /// Insert a space between Chinese text and adjacent Latin letters or digits on commit.
    pub cjk_spacing: bool,
    /// A short ambiguous word is read as English only if it ranks inside this many common words.
    pub english_rank_cutoff: u32,
    /// ...and the zhuyin reading's best character has fewer occurrences than this.
    pub rare_char_count: u32,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            cjk_spacing: false,
            english_rank_cutoff: 3000,
            rare_char_count: 50,
        }
    }
}

#[derive(Debug, Clone)]
enum Kind {
    Syllable(String),
    Latin,
    Punct,
}

#[derive(Debug, Clone)]
struct Group {
    kind: Kind,
    /// Output character for Latin and Punct groups.
    ch: char,
    fixed: Option<String>,
}

impl Group {
    fn latin(c: char) -> Group {
        Group {
            kind: Kind::Latin,
            ch: c,
            fixed: None,
        }
    }

    fn syllable(reading: String) -> Group {
        Group {
            kind: Kind::Syllable(reading),
            ch: ' ',
            fixed: None,
        }
    }

    fn is_syllable(&self) -> bool {
        matches!(self.kind, Kind::Syllable(_))
    }
}

/// The last automatic decision, so Tab can flip it.
#[derive(Debug, Clone)]
struct Flip {
    start: usize,
    end: usize,
    was_syllable: bool,
    raw: Vec<char>,
}

#[derive(Debug, Clone)]
struct Window {
    target: usize,
    cands: Vec<Candidate>,
    sel: usize,
}

#[derive(Clone)]
pub struct Engine {
    lex: Arc<Lexicon>,
    eng: Arc<English>,
    opt: Options,
    groups: Vec<Group>,
    /// Zhuyin keys typed but not yet closed by a tone key (always sits at `cursor`).
    pending: Vec<char>,
    cursor: usize,
    full_width: bool,
    window: Option<Window>,
    flip: Option<Flip>,
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '\'')
}

impl Engine {
    pub fn new(lex: Arc<Lexicon>, eng: Arc<English>, opt: Options) -> Engine {
        Engine {
            lex,
            eng,
            opt,
            groups: Vec::new(),
            pending: Vec::new(),
            cursor: 0,
            full_width: false,
            window: None,
            flip: None,
        }
    }

    pub fn reset(&mut self) {
        self.groups.clear();
        self.pending.clear();
        self.cursor = 0;
        self.window = None;
        self.flip = None;
    }

    pub fn is_composing(&self) -> bool {
        !self.groups.is_empty() || !self.pending.is_empty()
    }

    pub fn handle_key(&mut self, ev: KeyEvent) -> Snapshot {
        let mut commit = String::new();
        let handled = self.dispatch(ev, &mut commit);
        self.snapshot(handled, commit)
    }

    /// Would the input method consume this key? Does not change any state.
    /// A shell's pre-check (for example TSF `OnTestKeyDown`) must use this so
    /// it always agrees with the real handling.
    pub fn would_handle(&self, ev: KeyEvent) -> bool {
        self.clone().handle_key(ev).handled
    }

    /// Current state without consuming a key.
    pub fn current(&self) -> Snapshot {
        self.snapshot(false, String::new())
    }

    // ---- dispatch ----

    fn dispatch(&mut self, ev: KeyEvent, commit: &mut String) -> bool {
        if ev.code != KeyCode::Tab {
            self.flip = None;
        }
        if self.window.is_some() {
            if let Some(handled) = self.window_key(ev, commit) {
                return handled;
            }
        }
        if ev.has_command_modifier() {
            return self.passthrough_combo(commit);
        }
        match ev.code {
            KeyCode::Enter if ev.shift => self.passthrough_combo(commit),
            KeyCode::Tab if ev.shift => self.passthrough_combo(commit),
            KeyCode::Enter => {
                if self.is_composing() {
                    self.commit_all(commit);
                    true
                } else {
                    false
                }
            }
            KeyCode::Tab => {
                if !self.is_composing() {
                    return false;
                }
                if !self.pending.is_empty() {
                    self.pending_to_latin();
                } else {
                    self.flip_last();
                }
                true
            }
            KeyCode::Space if ev.shift => {
                self.full_width = !self.full_width;
                true
            }
            KeyCode::Space => self.space(),
            KeyCode::Backspace => {
                if !self.pending.is_empty() {
                    self.pending.pop();
                    true
                } else if self.cursor > 0 {
                    self.groups.remove(self.cursor - 1);
                    self.cursor -= 1;
                    true
                } else {
                    false
                }
            }
            KeyCode::Delete => {
                self.flush_pending();
                if self.cursor < self.groups.len() {
                    self.groups.remove(self.cursor);
                    true
                } else {
                    !self.groups.is_empty()
                }
            }
            KeyCode::Esc => {
                if self.is_composing() {
                    self.reset();
                    true
                } else {
                    false
                }
            }
            KeyCode::Left => self.move_cursor(|c, _| c.saturating_sub(1)),
            KeyCode::Right => self.move_cursor(|c, n| (c + 1).min(n)),
            KeyCode::Home => self.move_cursor(|_, _| 0),
            KeyCode::End => self.move_cursor(|_, n| n),
            // Swallowed while composing: some hosts (macOS IMK) clear the
            // composition when a key they were told is unhandled arrives.
            KeyCode::Up => self.is_composing(),
            KeyCode::Down => {
                self.flush_pending();
                if self.groups.is_empty() {
                    return false;
                }
                self.open_window();
                true
            }
            KeyCode::Char(c) => {
                self.char_key(c, ev);
                true
            }
        }
    }

    /// Ctrl/Alt/Win combos and Shift+Enter/Tab belong to the application.
    /// While composing, the first press only confirms the text.
    fn passthrough_combo(&mut self, commit: &mut String) -> bool {
        if self.is_composing() {
            self.commit_all(commit);
            true
        } else {
            false
        }
    }

    fn move_cursor(&mut self, f: impl Fn(usize, usize) -> usize) -> bool {
        self.flush_pending();
        if self.groups.is_empty() {
            return false;
        }
        self.cursor = f(self.cursor, self.groups.len());
        true
    }

    // ---- characters ----

    fn char_key(&mut self, c: char, ev: KeyEvent) {
        if ev.shift {
            self.flush_pending();
            if c.is_ascii_alphabetic() {
                let up = ev.text.unwrap_or_else(|| c.to_ascii_uppercase());
                self.insert(Group::latin(up));
            } else {
                let t = ev.text.unwrap_or(c);
                let g = self.punct_group(t);
                self.insert(g);
            }
            return;
        }
        if (self.in_word() && is_word_char(c)) || (self.in_number() && (c.is_ascii_digit() || c == '.')) {
            self.insert(Group::latin(c));
            return;
        }
        self.zhuyin_key(c);
    }

    fn zhuyin_key(&mut self, c: char) {
        if zhuyin::tone_of_key(c).is_some() {
            if self.pending.is_empty() {
                // A tone key with nothing to close is a number.
                self.insert(Group::latin(c));
            } else {
                self.complete_with_tone(c);
            }
            return;
        }
        if zhuyin::symbol_of_key(c).is_none() {
            self.flush_pending();
            let g = self.punct_group(c);
            self.insert(g);
            return;
        }
        let mut cand = self.pending.clone();
        cand.push(c);
        let still_zhuyin = zhuyin::symbols_of_keys(&cand)
            .map(|s| self.lex.is_prefix(&s))
            .unwrap_or(false);
        if still_zhuyin {
            self.pending.push(c);
        } else {
            self.break_pending(c);
        }
    }

    /// `c` cannot extend the pending keys as zhuyin.
    fn break_pending(&mut self, c: char) {
        if self.pending_is_lone_punct() {
            self.flush_pending();
            self.zhuyin_key(c);
            return;
        }
        if self.pending.is_empty() {
            self.insert(Group::latin(c));
            return;
        }
        self.pending_to_latin();
        if is_word_char(c) {
            self.insert(Group::latin(c));
        } else {
            self.zhuyin_key(c);
        }
    }

    fn complete_with_tone(&mut self, t: char) {
        let symbols = zhuyin::symbols_of_keys(&self.pending).unwrap_or_default();
        let mark = zhuyin::tone_of_key(t).expect("tone key");
        let reading = format!("{symbols}{mark}");
        if self.lex.is_syllable(&reading) {
            let mut raw = std::mem::take(&mut self.pending);
            raw.push(t);
            self.insert_syllable(reading, raw);
        } else {
            self.pending_to_latin();
            self.insert(Group::latin(t));
        }
    }

    fn space(&mut self) -> bool {
        if !self.pending.is_empty() {
            self.space_with_pending();
            return true;
        }
        if self.in_word() {
            self.insert(Group::latin(' '));
            return true;
        }
        if self.cursor > 0 && !self.groups[self.cursor - 1].is_syllable() {
            self.insert(Group::latin(' '));
            return true;
        }
        if self.groups.is_empty() {
            return false;
        }
        self.open_window();
        true
    }

    fn space_with_pending(&mut self) {
        let raw = self.pending.clone();
        if self.pending_is_lone_punct() && raw[0] != '-' {
            let after_latin = self.cursor > 0 && !self.groups[self.cursor - 1].is_syllable();
            self.flush_pending();
            if after_latin {
                self.insert(Group::latin(' '));
            }
            return;
        }
        let symbols = zhuyin::symbols_of_keys(&raw).unwrap_or_default();
        let all_letters = raw.iter().all(|c| c.is_ascii_alphabetic());
        let word: String = raw.iter().collect();
        let prefer_english = all_letters
            && self
                .eng
                .rank(&word)
                .is_some_and(|r| r < self.opt.english_rank_cutoff)
            && self.lex.best_char_count(&symbols) < self.opt.rare_char_count;
        let mut full = raw.clone();
        full.push(' ');
        if self.lex.is_syllable(&symbols) && !prefer_english {
            self.pending.clear();
            self.insert_syllable(symbols, full);
        } else {
            let start = self.cursor;
            let can_flip = self.lex.is_syllable(&symbols);
            self.pending_to_latin();
            self.insert(Group::latin(' '));
            if can_flip {
                self.flip = Some(Flip {
                    start,
                    end: self.cursor,
                    was_syllable: false,
                    raw: full,
                });
            }
        }
    }

    // ---- groups ----

    fn insert(&mut self, g: Group) {
        self.groups.insert(self.cursor, g);
        self.cursor += 1;
        self.window = None;
    }

    fn insert_syllable(&mut self, reading: String, raw: Vec<char>) {
        let start = self.cursor;
        self.insert(Group::syllable(reading));
        self.flip = Some(Flip {
            start,
            end: self.cursor,
            was_syllable: true,
            raw,
        });
    }

    fn punct_group(&self, c: char) -> Group {
        Group {
            kind: Kind::Punct,
            ch: if self.full_width {
                zhuyin::full_width_of(c)
            } else {
                c
            },
            fixed: None,
        }
    }

    fn pending_is_lone_punct(&self) -> bool {
        self.pending.len() == 1 && zhuyin::is_punct_key(self.pending[0])
    }

    /// Pending keys become English letters, exactly as typed.
    fn pending_to_latin(&mut self) {
        let keys = std::mem::take(&mut self.pending);
        for c in keys {
            self.insert(Group::latin(c));
        }
    }

    /// Pending keys that never found a tone: punctuation if it is a single
    /// punctuation key, otherwise English letters.
    fn flush_pending(&mut self) {
        if self.pending_is_lone_punct() {
            let c = self.pending.remove(0);
            let g = self.punct_group(c);
            self.insert(g);
        } else {
            self.pending_to_latin();
        }
    }

    /// Cursor sits inside an English word (a run of word characters containing a letter).
    fn in_word(&self) -> bool {
        if !self.pending.is_empty() || self.cursor == 0 {
            return false;
        }
        let mut has_letter = false;
        let mut i = self.cursor;
        while i > 0 {
            let g = &self.groups[i - 1];
            if !matches!(g.kind, Kind::Latin) || !is_word_char(g.ch) {
                break;
            }
            has_letter |= g.ch.is_ascii_alphabetic();
            i -= 1;
        }
        i < self.cursor && has_letter
    }

    /// Cursor sits right after a number such as `2026` or `192.168`.
    fn in_number(&self) -> bool {
        if !self.pending.is_empty() || self.cursor == 0 {
            return false;
        }
        let last = &self.groups[self.cursor - 1];
        if !matches!(last.kind, Kind::Latin) {
            return false;
        }
        if last.ch.is_ascii_digit() {
            return true;
        }
        last.ch == '.'
            && self.cursor >= 2
            && matches!(self.groups[self.cursor - 2].kind, Kind::Latin)
            && self.groups[self.cursor - 2].ch.is_ascii_digit()
    }

    /// Tab: swap the last automatic decision between zhuyin and English.
    fn flip_last(&mut self) {
        let Some(f) = self.flip.clone() else { return };
        if f.end != self.groups.len() || self.cursor != self.groups.len() {
            return;
        }
        if f.was_syllable {
            self.groups.truncate(f.start);
            self.cursor = f.start;
            for &c in &f.raw {
                self.insert(Group::latin(c));
            }
            self.flip = Some(Flip {
                start: f.start,
                end: self.cursor,
                was_syllable: false,
                raw: f.raw,
            });
        } else {
            let (&last, body) = f.raw.split_last().expect("non-empty raw");
            let Some(symbols) = zhuyin::symbols_of_keys(body) else {
                return;
            };
            let reading = match zhuyin::tone_of_key(last) {
                Some(mark) => format!("{symbols}{mark}"),
                None if last == ' ' => symbols,
                None => return,
            };
            if !self.lex.is_syllable(&reading) {
                return;
            }
            self.groups.truncate(f.start);
            self.cursor = f.start;
            self.insert_syllable(reading, f.raw);
        }
    }

    // ---- candidate window ----

    fn open_window(&mut self) {
        if self.groups.is_empty() {
            return;
        }
        let target = if self.cursor < self.groups.len() {
            self.cursor
        } else {
            self.groups.len() - 1
        };
        if !self.groups[target].is_syllable() {
            return;
        }
        let units = self.run_units(target);
        let cands = convert::candidates(&self.lex, &units, 0);
        if cands.is_empty() {
            return;
        }
        self.window = Some(Window {
            target,
            cands,
            sel: 0,
        });
    }

    /// Syllable units from `start` to the end of its run.
    fn run_units(&self, start: usize) -> Vec<Unit> {
        self.groups[start..]
            .iter()
            .take_while(|g| g.is_syllable())
            .map(|g| match &g.kind {
                Kind::Syllable(r) => Unit {
                    reading: r.clone(),
                    fixed: g.fixed.clone(),
                },
                _ => unreachable!(),
            })
            .collect()
    }

    /// `Some(handled)` when the window consumed the key; `None` when it closed
    /// and the key should be processed normally.
    fn window_key(&mut self, ev: KeyEvent, commit: &mut String) -> Option<bool> {
        let _ = commit;
        let w = self.window.as_mut()?;
        let n = w.cands.len();
        match ev.code {
            KeyCode::Char(d @ '1'..='9') if !ev.shift && !ev.has_command_modifier() => {
                let idx = (w.sel / PAGE_SIZE) * PAGE_SIZE + (d as usize - '1' as usize);
                if idx < n {
                    self.select_candidate(idx);
                }
                Some(true)
            }
            KeyCode::Down => {
                w.sel = (w.sel + 1) % n;
                Some(true)
            }
            KeyCode::Up => {
                w.sel = (w.sel + n - 1) % n;
                Some(true)
            }
            KeyCode::Space if !ev.shift => {
                let next = (w.sel / PAGE_SIZE + 1) * PAGE_SIZE;
                w.sel = if next < n { next } else { 0 };
                Some(true)
            }
            KeyCode::Enter if !ev.shift => {
                let idx = w.sel;
                self.select_candidate(idx);
                Some(true)
            }
            KeyCode::Esc => {
                self.window = None;
                Some(true)
            }
            _ => {
                self.window = None;
                None
            }
        }
    }

    fn select_candidate(&mut self, idx: usize) {
        let Some(w) = self.window.take() else { return };
        let cand = &w.cands[idx];
        for (k, ch) in cand.text.chars().enumerate() {
            if let Some(g) = self.groups.get_mut(w.target + k) {
                g.fixed = Some(ch.to_string());
            }
        }
        self.cursor = (w.target + cand.len).min(self.groups.len());
    }

    // ---- output ----

    fn group_texts(&self) -> Vec<String> {
        let mut out = vec![String::new(); self.groups.len()];
        let mut i = 0;
        while i < self.groups.len() {
            if self.groups[i].is_syllable() {
                let units = self.run_units(i);
                let texts = convert::convert(&self.lex, &units);
                for (k, t) in texts.into_iter().enumerate() {
                    out[i + k] = t;
                }
                i += units.len();
            } else {
                out[i] = self.groups[i].ch.to_string();
                i += 1;
            }
        }
        out
    }

    fn commit_all(&mut self, commit: &mut String) {
        self.flush_pending();
        let texts = self.group_texts();
        for (i, t) in texts.iter().enumerate() {
            if self.opt.cjk_spacing && i > 0 {
                let prev = &self.groups[i - 1];
                let cur = &self.groups[i];
                let latin_alnum = |g: &Group| matches!(g.kind, Kind::Latin) && g.ch.is_ascii_alphanumeric();
                if (prev.is_syllable() && latin_alnum(cur)) || (latin_alnum(prev) && cur.is_syllable()) {
                    commit.push(' ');
                }
            }
            commit.push_str(t);
        }
        self.reset();
    }

    fn snapshot(&self, handled: bool, commit: String) -> Snapshot {
        let texts = self.group_texts();
        let mut preedit = Vec::new();
        let mut cursor16 = 0usize;
        let pending_text = zhuyin::symbols_of_keys(&self.pending).unwrap_or_default();
        let selected = match &self.window {
            Some(w) => Some(w.target),
            None if self.cursor < self.groups.len() => Some(self.cursor),
            None => None,
        };
        let push_pending = |preedit: &mut Vec<Segment>| {
            if !pending_text.is_empty() {
                preedit.push(Segment {
                    text: pending_text.clone(),
                    kind: SegKind::Pending,
                });
            }
        };
        for (i, t) in texts.iter().enumerate() {
            if i == self.cursor {
                push_pending(&mut preedit);
                cursor16 = preedit.iter().map(|s| s.text.encode_utf16().count()).sum();
            }
            let kind = if selected == Some(i) {
                SegKind::Selected
            } else if self.groups[i].is_syllable() {
                SegKind::Chinese
            } else {
                SegKind::Latin
            };
            preedit.push(Segment {
                text: t.clone(),
                kind,
            });
        }
        if self.cursor >= texts.len() {
            push_pending(&mut preedit);
            cursor16 = preedit.iter().map(|s| s.text.encode_utf16().count()).sum();
        }
        let candidates = self.window.as_ref().map(|w| Candidates {
            items: w.cands.iter().map(|c| c.text.clone()).collect(),
            selected: w.sel,
            page_size: PAGE_SIZE,
        });
        Snapshot {
            handled,
            commit,
            preedit,
            cursor: cursor16,
            candidates,
            full_width: self.full_width,
        }
    }
}
