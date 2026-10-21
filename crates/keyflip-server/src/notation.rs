//! Key notation (`keyflip_core::parse_keys`) to wire keys.

use keyflip_core::{parse_keys, KeyCode, KeyEvent};
use keyflip_proto::Key;

pub fn to_wire(ev: &KeyEvent) -> Key {
    let code = match ev.code {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Space => "Space".into(),
        KeyCode::Enter => "Enter".into(),
        KeyCode::Tab => "Tab".into(),
        KeyCode::Backspace => "Backspace".into(),
        KeyCode::Delete => "Delete".into(),
        KeyCode::Esc => "Esc".into(),
        KeyCode::Left => "Left".into(),
        KeyCode::Right => "Right".into(),
        KeyCode::Up => "Up".into(),
        KeyCode::Down => "Down".into(),
        KeyCode::Home => "Home".into(),
        KeyCode::End => "End".into(),
    };
    Key {
        code,
        text: ev.text,
        shift: ev.shift,
        ctrl: ev.ctrl,
        alt: ev.alt,
        meta: ev.meta,
    }
}

pub fn parse_wire_keys(s: &str) -> Result<Vec<Key>, String> {
    Ok(parse_keys(s)?.iter().map(to_wire).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_and_shifted_letters() {
        let k = parse_wire_keys("aH").unwrap();
        assert_eq!((k[0].code.as_str(), k[0].text, k[0].shift), ("a", Some('a'), false));
        assert_eq!((k[1].code.as_str(), k[1].text, k[1].shift), ("h", Some('H'), true));
    }

    #[test]
    fn named_keys() {
        let k = parse_wire_keys("<E><S-E>_<T><B><X><L>").unwrap();
        let names: Vec<_> = k.iter().map(|k| k.code.as_str()).collect();
        assert_eq!(names, ["Enter", "Enter", "Space", "Tab", "Backspace", "Esc", "Left"]);
        assert!(k[1].shift && !k[0].shift);
        assert_eq!(k[0].text, None);
    }

    #[test]
    fn mapped_keys_are_accepted_by_the_protocol() {
        for k in parse_wire_keys("su3cl3Hi,.;/-'<E><T><U><D><L><R>_").unwrap() {
            k.to_event().unwrap();
        }
    }

    #[test]
    fn bad_notation_is_an_error() {
        assert!(parse_wire_keys("<Q>").is_err());
        assert!(parse_wire_keys("<E").is_err());
    }
}
