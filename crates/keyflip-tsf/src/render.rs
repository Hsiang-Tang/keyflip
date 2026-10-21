//! What to draw for one engine answer, as plain data. No Windows API here, so
//! it is tested on any platform. The shell only turns this into TSF calls.

use keyflip_proto::{WireCandidates, WireSegment};

/// How a stretch of the composition is displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunKind {
    /// Underline: chinese, latin and pending text.
    Input,
    /// Highlight: the `selected` segment.
    Selected,
}

/// A stretch of the composition, in UTF-16 code units.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub start: usize,
    pub len: usize,
    pub kind: RunKind,
}

/// Rows of the candidate window for the current page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandView {
    /// `1. text` ... up to nine rows.
    pub rows: Vec<String>,
    /// Row index that is currently selected, if it is on this page.
    pub highlight: Option<usize>,
}

/// The composition and candidate window after one key.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct View {
    pub text: Vec<u16>,
    pub runs: Vec<Run>,
    /// UTF-16 offset of the caret inside `text`.
    pub cursor: usize,
    pub cand: Option<CandView>,
}

impl View {
    pub fn from_parts(
        preedit: &[WireSegment],
        cursor: usize,
        cand: Option<&WireCandidates>,
    ) -> View {
        let mut text: Vec<u16> = Vec::new();
        let mut runs = Vec::new();
        for seg in preedit {
            let start = text.len();
            text.extend(seg.text.encode_utf16());
            let len = text.len() - start;
            if len == 0 {
                continue;
            }
            let kind = if seg.kind == "selected" {
                RunKind::Selected
            } else {
                RunKind::Input
            };
            runs.push(Run { start, len, kind });
        }
        let cursor = cursor.min(text.len());
        View {
            text,
            runs,
            cursor,
            cand: cand.and_then(page_rows),
        }
    }

    /// Nothing to show: no composition and no candidate window.
    pub fn is_idle(&self) -> bool {
        self.text.is_empty() && self.cand.is_none()
    }
}

/// Rows for the page that contains the selected item. `page_size` is capped at
/// nine because the rows are selected with the keys 1 to 9.
pub fn page_rows(c: &WireCandidates) -> Option<CandView> {
    if c.items.is_empty() {
        return None;
    }
    let size = c.page_size.clamp(1, 9);
    let selected = c.selected.min(c.items.len() - 1);
    let first = selected / size * size;
    let rows = c.items[first..(first + size).min(c.items.len())]
        .iter()
        .enumerate()
        .map(|(i, s)| format!("{}. {}", i + 1, s))
        .collect();
    Some(CandView {
        rows,
        highlight: Some(selected - first),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(text: &str, kind: &str) -> WireSegment {
        WireSegment {
            text: text.into(),
            kind: kind.into(),
        }
    }

    fn cands(n: usize, selected: usize, page_size: usize) -> WireCandidates {
        WireCandidates {
            items: (0..n).map(|i| format!("字{i}")).collect(),
            selected,
            page_size,
            expanded: false,
        }
    }

    #[test]
    fn runs_cover_segments_in_utf16_units() {
        let v = View::from_parts(
            &[
                seg("你好", "chinese"),
                seg("Hs", "latin"),
                seg("😀", "selected"),
                seg("", "pending"),
                seg("g", "pending"),
            ],
            4,
            None,
        );
        assert_eq!(v.text, "你好Hs😀g".encode_utf16().collect::<Vec<_>>());
        assert_eq!(
            v.runs,
            vec![
                Run { start: 0, len: 2, kind: RunKind::Input },
                Run { start: 2, len: 2, kind: RunKind::Input },
                Run { start: 4, len: 2, kind: RunKind::Selected },
                Run { start: 6, len: 1, kind: RunKind::Input },
            ]
        );
        assert_eq!(v.cursor, 4);
        assert!(!v.is_idle());
    }

    #[test]
    fn cursor_is_clamped_and_empty_is_idle() {
        let v = View::from_parts(&[seg("ab", "latin")], 99, None);
        assert_eq!(v.cursor, 2);
        let v = View::from_parts(&[], 5, None);
        assert_eq!(v.cursor, 0);
        assert!(v.is_idle());
    }

    #[test]
    fn candidates_alone_are_not_idle() {
        let c = cands(3, 0, 9);
        let v = View::from_parts(&[], 0, Some(&c));
        assert!(!v.is_idle());
    }

    #[test]
    fn first_page_is_numbered_from_one() {
        let v = page_rows(&cands(5, 1, 9)).unwrap();
        assert_eq!(v.rows, ["1. 字0", "2. 字1", "3. 字2", "4. 字3", "5. 字4"]);
        assert_eq!(v.highlight, Some(1));
    }

    #[test]
    fn selected_on_a_later_page_shows_that_page() {
        let v = page_rows(&cands(25, 10, 9)).unwrap();
        assert_eq!(v.rows.len(), 9);
        assert_eq!(v.rows[0], "1. 字9");
        assert_eq!(v.highlight, Some(1));
        let v = page_rows(&cands(25, 24, 9)).unwrap();
        assert_eq!(v.rows, ["1. 字18", "2. 字19", "3. 字20", "4. 字21", "5. 字22", "6. 字23", "7. 字24"]);
        assert_eq!(v.highlight, Some(6));
    }

    #[test]
    fn odd_page_sizes_are_made_safe() {
        assert_eq!(page_rows(&cands(30, 0, 0)).unwrap().rows.len(), 1);
        assert_eq!(page_rows(&cands(30, 0, 50)).unwrap().rows.len(), 9);
        assert_eq!(page_rows(&cands(4, 99, 5)).unwrap().highlight, Some(3));
        assert!(page_rows(&cands(0, 0, 9)).is_none());
    }
}
