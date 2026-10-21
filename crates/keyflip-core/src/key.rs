//! Key events handed to the engine by a platform shell.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    /// The physical key on a US keyboard, as its unshifted character
    /// (`a`-`z`, `0`-`9`, `,` `.` `;` `/` `-` `'` and other punctuation).
    Char(char),
    Space,
    Enter,
    Tab,
    Backspace,
    Delete,
    Esc,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub code: KeyCode,
    /// The character this key press produces with the current modifiers
    /// (for example `H` or `?`). `None` for non-printing keys.
    pub text: Option<char>,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

impl KeyEvent {
    pub fn plain(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            text: match code {
                KeyCode::Char(c) => Some(c),
                _ => None,
            },
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
        }
    }

    pub fn shifted(code: KeyCode, text: char) -> KeyEvent {
        KeyEvent {
            shift: true,
            text: Some(text),
            ..KeyEvent::plain(code)
        }
    }

    /// Ctrl, Alt, or Win/Cmd held: the application wants this key.
    pub fn has_command_modifier(&self) -> bool {
        self.ctrl || self.alt || self.meta
    }
}
