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
/// `a-z 0-9 , . ; / - '` and other unshifted keys plain, `A-Z` Shift+letter, `_` space,
/// `~ ! @ # $ % ^ & * ( ) + { } | : " ?` Shift+the key that types them,
/// `<S-k>` Shift+the plain key `k` (for example `<S-->` types `_`),
/// `<E>` Enter, `<S-E>` Shift+Enter, `<T>` Tab, `<L>` `<R>` `<D>` `<U>` arrows,
/// `<B>` Backspace, `<X>` Esc, `<S-_>` Shift+Space, `<C>` toggles the CapsLock light
/// for the keys that follow (it sends no key itself).
pub fn parse_keys(s: &str) -> Result<Vec<KeyEvent>, String> {
    let mut out: Vec<KeyEvent> = Vec::new();
    let mut caps = false;
    // Events pushed since the last pass get the light as it was when they were
    // typed; `<C>` pushes nothing, so the light never changes mid-pass.
    let mut marked = 0;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        for ev in &mut out[marked..] {
            ev.caps_lock = caps;
        }
        marked = out.len();
        if c == '<' {
            let mut name = String::new();
            loop {
                match chars.next() {
                    Some('>') => break,
                    Some(ch) => name.push(ch),
                    None => return Err(format!("unterminated <{name}")),
                }
            }
            if name == "C" {
                caps = !caps;
                continue;
            }
            let (shift, base) = match name.strip_prefix("S-") {
                Some(rest) => (true, rest),
                None => (false, name.as_str()),
            };
            let mut one = base.chars();
            if let (true, Some(k), None) = (shift, one.next(), one.next()) {
                if let Some(t) = shifted_text(k) {
                    out.push(KeyEvent::shifted(KeyCode::Char(k), t));
                    continue;
                }
            }
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
        } else if let Some(k) = unshifted_key(c) {
            out.push(KeyEvent::shifted(KeyCode::Char(k), c));
        } else if c.is_ascii_uppercase() {
            out.push(KeyEvent::shifted(
                KeyCode::Char(c.to_ascii_lowercase()),
                c,
            ));
        } else {
            out.push(KeyEvent::plain(KeyCode::Char(c)));
        }
    }
    for ev in &mut out[marked..] {
        ev.caps_lock = caps;
    }
    Ok(out)
}

/// US layout: the character Shift+`k` types, for a plain key `k`.
fn shifted_text(k: char) -> Option<char> {
    const PAIRS: &str = "`~1!2@3#4$5%6^7&8*9(0)-_=+[{]}\\|;:'\",<.>/?";
    if k.is_ascii_lowercase() {
        return Some(k.to_ascii_uppercase());
    }
    let v: Vec<char> = PAIRS.chars().collect();
    v.chunks(2).find(|p| p[0] == k).map(|p| p[1])
}

/// The plain key whose shifted form is the literal symbol `c` in the notation.
/// `_`, `<`, and `>` are reserved by the notation; write `<S-->`, `<S-,>`, `<S-.>`.
fn unshifted_key(c: char) -> Option<char> {
    const PAIRS: &str = "`~1!2@3#4$5%6^7&8*9(0)=+[{]}\\|;:'\"/?";
    let v: Vec<char> = PAIRS.chars().collect();
    v.chunks(2).find(|p| p[1] == c).map(|p| p[0])
}
