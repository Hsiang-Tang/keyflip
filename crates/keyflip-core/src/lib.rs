//! keyflip core: a zhuyin engine that decides between Chinese and English on its own.

pub mod convert;
pub mod engine;
pub mod english;
pub mod key;
pub mod lexicon;
pub mod snapshot;
pub mod zhuyin;

pub use engine::{Engine, Options};
pub use english::English;
pub use key::{KeyCode, KeyEvent};
pub use lexicon::Lexicon;
pub use snapshot::{Candidates, SegKind, Segment, Snapshot};

use std::sync::Arc;

/// An engine over the bundled lexicon and English word list.
pub fn bundled_engine(opt: Options) -> Engine {
    Engine::new(
        Arc::new(Lexicon::bundled()),
        Arc::new(English::bundled()),
        opt,
    )
}

/// Parse the test/CLI key notation into key events.
///
/// `a-z 0-9 , . ; / - '` plain keys, `A-Z` Shift+letter, `_` space,
/// `<E>` Enter, `<S-E>` Shift+Enter, `<T>` Tab, `<L>` `<R>` `<D>` `<U>` arrows,
/// `<B>` Backspace, `<X>` Esc, `<S-_>` Shift+Space.
pub fn parse_keys(s: &str) -> Result<Vec<KeyEvent>, String> {
    let mut out = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '<' {
            let mut name = String::new();
            loop {
                match chars.next() {
                    Some('>') => break,
                    Some(ch) => name.push(ch),
                    None => return Err(format!("unterminated <{name}")),
                }
            }
            let (shift, base) = match name.strip_prefix("S-") {
                Some(rest) => (true, rest),
                None => (false, name.as_str()),
            };
            let code = match base {
                "E" => KeyCode::Enter,
                "T" => KeyCode::Tab,
                "L" => KeyCode::Left,
                "R" => KeyCode::Right,
                "D" => KeyCode::Down,
                "U" => KeyCode::Up,
                "B" => KeyCode::Backspace,
                "X" => KeyCode::Esc,
                "_" => KeyCode::Space,
                other => return Err(format!("unknown key <{other}>")),
            };
            let mut ev = KeyEvent::plain(code);
            ev.shift = shift;
            out.push(ev);
        } else if c == '_' {
            out.push(KeyEvent::plain(KeyCode::Space));
        } else if c.is_ascii_uppercase() {
            out.push(KeyEvent::shifted(
                KeyCode::Char(c.to_ascii_lowercase()),
                c,
            ));
        } else {
            out.push(KeyEvent::plain(KeyCode::Char(c)));
        }
    }
    Ok(out)
}
