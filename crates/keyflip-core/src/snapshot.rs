//! What the engine reports after every key.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegKind {
    Chinese,
    Latin,
    /// Typed zhuyin symbols still waiting for a tone key.
    Pending,
    /// The unit the candidate window or cursor currently targets.
    Selected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub text: String,
    pub kind: SegKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidates {
    pub items: Vec<String>,
    pub selected: usize,
    pub page_size: usize,
    /// The grid showing every candidate, `page_size` per row (MI-22).
    pub expanded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Snapshot {
    /// Whether the input method consumed the key.
    pub handled: bool,
    /// Text to insert into the application.
    pub commit: String,
    pub preedit: Vec<Segment>,
    /// Cursor position inside the preedit, in UTF-16 code units.
    pub cursor: usize,
    pub candidates: Option<Candidates>,
    pub full_width: bool,
}

impl Snapshot {
    pub fn preedit_text(&self) -> String {
        self.preedit.iter().map(|s| s.text.as_str()).collect()
    }
}
