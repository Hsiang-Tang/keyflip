//! Windows key events to protocol keys. No Windows API in here, so it is
//! tested on any platform.
//!
//! The shell never judges what a key means. It only names the physical key
//! (`code`: the unshifted US-layout character, or a key name) and reports the
//! modifier state and the text the live keyboard layout would type.
//!
//! CapsLock never counts as Shift. Shift is read from the Shift keys only.

use keyflip_proto::Key;

pub const VK_BACK: u32 = 0x08;
pub const VK_TAB: u32 = 0x09;
pub const VK_RETURN: u32 = 0x0D;
pub const VK_SHIFT: u32 = 0x10;
pub const VK_CONTROL: u32 = 0x11;
pub const VK_MENU: u32 = 0x12;
pub const VK_ESCAPE: u32 = 0x1B;
pub const VK_SPACE: u32 = 0x20;
pub const VK_END: u32 = 0x23;
pub const VK_HOME: u32 = 0x24;
pub const VK_LEFT: u32 = 0x25;
pub const VK_UP: u32 = 0x26;
pub const VK_RIGHT: u32 = 0x27;
pub const VK_DOWN: u32 = 0x28;
pub const VK_DELETE: u32 = 0x2E;
pub const VK_LWIN: u32 = 0x5B;
pub const VK_RWIN: u32 = 0x5C;
pub const VK_LSHIFT: u32 = 0xA0;
pub const VK_RSHIFT: u32 = 0xA1;
pub const VK_LCONTROL: u32 = 0xA2;
pub const VK_RCONTROL: u32 = 0xA3;
pub const VK_LMENU: u32 = 0xA4;
pub const VK_RMENU: u32 = 0xA5;

/// Which modifiers are physically down, plus the CapsLock light.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Mods {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
    /// The CapsLock toggle state (ADR-0006), not whether the key is down.
    pub caps_lock: bool,
}

/// Read modifiers from a 256-entry `GetKeyboardState` array. Entry bit 0x80
/// means "down"; bit 0x01 of VK_CAPITAL (0x14) is the CapsLock light.
pub fn mods_from_state(state: &[u8; 256]) -> Mods {
    let down = |vk: u32| state[vk as usize] & 0x80 != 0;
    Mods {
        caps_lock: state[0x14] & 0x01 != 0,
        shift: down(VK_SHIFT) || down(VK_LSHIFT) || down(VK_RSHIFT),
        ctrl: down(VK_CONTROL) || down(VK_LCONTROL) || down(VK_RCONTROL),
        alt: down(VK_MENU) || down(VK_LMENU) || down(VK_RMENU),
        meta: down(VK_LWIN) || down(VK_RWIN),
    }
}

/// True for keys that are modifiers or toggles by themselves. They are never
/// sent to the engine, so tapping Shift alone cannot do anything.
pub fn is_modifier_vk(vk: u32) -> bool {
    matches!(
        vk,
        VK_SHIFT
            | VK_CONTROL
            | VK_MENU
            | VK_LSHIFT
            | VK_RSHIFT
            | VK_LCONTROL
            | VK_RCONTROL
            | VK_LMENU
            | VK_RMENU
            | VK_LWIN
            | VK_RWIN
            | 0x14 // VK_CAPITAL
            | 0x90 // VK_NUMLOCK
            | 0x91 // VK_SCROLL
    )
}

/// Name for keys that are not printable characters.
fn named(vk: u32) -> Option<&'static str> {
    Some(match vk {
        VK_SPACE => "Space",
        VK_RETURN => "Enter",
        VK_TAB => "Tab",
        VK_BACK => "Backspace",
        VK_DELETE => "Delete",
        VK_ESCAPE => "Esc",
        VK_LEFT => "Left",
        VK_RIGHT => "Right",
        VK_UP => "Up",
        VK_DOWN => "Down",
        VK_HOME => "Home",
        VK_END => "End",
        _ => return None,
    })
}

/// Unshifted US-layout character of a physical key, from the set 1 scan code
/// (the code Windows reports in bits 16-23 of lParam). Main block only.
fn char_from_scan(scan: u32) -> Option<char> {
    const ROW_NUM: &[u8] = b"1234567890-=";
    const ROW_Q: &[u8] = b"qwertyuiop[]";
    const ROW_A: &[u8] = b"asdfghjkl;'`";
    const ROW_Z: &[u8] = b"\\zxcvbnm,./";
    let c = match scan {
        0x02..=0x0D => ROW_NUM[(scan - 0x02) as usize],
        0x10..=0x1B => ROW_Q[(scan - 0x10) as usize],
        0x1E..=0x29 => ROW_A[(scan - 0x1E) as usize],
        0x2B..=0x35 => ROW_Z[(scan - 0x2B) as usize],
        _ => return None,
    };
    Some(c as char)
}

/// Unshifted US-layout character from the virtual-key code (fallback when no
/// usable scan code is present).
fn char_from_vk(vk: u32) -> Option<char> {
    Some(match vk {
        0x30..=0x39 | 0x41..=0x5A => {
            let c = vk as u8 as char;
            c.to_ascii_lowercase()
        }
        0xBA => ';',  // VK_OEM_1
        0xBB => '=',  // VK_OEM_PLUS
        0xBC => ',',  // VK_OEM_COMMA
        0xBD => '-',  // VK_OEM_MINUS
        0xBE => '.',  // VK_OEM_PERIOD
        0xBF => '/',  // VK_OEM_2
        0xC0 => '`',  // VK_OEM_3
        0xDB => '[',  // VK_OEM_4
        0xDC => '\\', // VK_OEM_5
        0xDD => ']',  // VK_OEM_6
        0xDE => '\'', // VK_OEM_7
        _ => return None,
    })
}

/// The physical key's name for the protocol, or `None` when this key is not
/// one the engine knows (function keys, numpad, media keys, modifiers, IME
/// process keys, injected Unicode packets...). Those pass straight through.
pub fn code_of(vk: u32, scan: u32, extended: bool) -> Option<String> {
    if is_modifier_vk(vk) {
        return None;
    }
    if let Some(n) = named(vk) {
        return Some(n.to_owned());
    }
    // Printable keys must be main-block keys; numpad VKs never reach the
    // tables below (their VKs are 0x60..), and extended scan codes such as the
    // numpad slash share a scan code with a main-block key.
    if (0x60..=0x6F).contains(&vk) {
        return None;
    }
    let from_scan = if scan != 0 && !extended {
        char_from_scan(scan)
    } else {
        None
    };
    let c = from_scan.or_else(|| if extended { None } else { char_from_vk(vk) })?;
    Some(c.to_string())
}

/// Pick the typed character out of a `ToUnicodeEx` result. Control characters
/// (Ctrl+A, Enter, Tab...) and sequences of several characters give `None`.
pub fn text_from_utf16(units: &[u16]) -> Option<char> {
    let s = String::from_utf16(units).ok()?;
    let mut it = s.chars();
    match (it.next(), it.next()) {
        (Some(c), None) if !c.is_control() => Some(c),
        _ => None,
    }
}

/// Build the protocol key. `text` is whatever the layout produced.
pub fn translate(
    vk: u32,
    scan: u32,
    extended: bool,
    mods: Mods,
    text: Option<char>,
) -> Option<Key> {
    let code = code_of(vk, scan, extended)?;
    Some(Key {
        code,
        text,
        shift: mods.shift,
        ctrl: mods.ctrl,
        alt: mods.alt,
        meta: mods.meta,
        caps_lock: mods.caps_lock,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none() -> Mods {
        Mods::default()
    }

    #[test]
    fn letters_and_digits_use_the_unshifted_character() {
        assert_eq!(code_of(b'A' as u32, 0, false).unwrap(), "a");
        assert_eq!(code_of(b'Z' as u32, 0, false).unwrap(), "z");
        assert_eq!(code_of(b'7' as u32, 0, false).unwrap(), "7");
        assert_eq!(code_of(0, 0, false), None);
    }

    #[test]
    fn oem_keys_map_to_their_us_characters() {
        let table = [
            (0xBA, ';'),
            (0xBC, ','),
            (0xBE, '.'),
            (0xBF, '/'),
            (0xBD, '-'),
            (0xDE, '\''),
            (0xBB, '='),
            (0xC0, '`'),
            (0xDB, '['),
            (0xDD, ']'),
            (0xDC, '\\'),
        ];
        for (vk, c) in table {
            assert_eq!(code_of(vk, 0, false).unwrap(), c.to_string(), "{vk:#x}");
        }
    }

    #[test]
    fn scan_code_wins_over_a_remapped_layout() {
        // On a Dvorak layout the key at the US "S" position reports VK_O but
        // scan code 0x1F. The zhuyin layout follows the physical key.
        assert_eq!(code_of(b'O' as u32, 0x1F, false).unwrap(), "s");
        // Scan table spot checks along the rows.
        for (scan, c) in [
            (0x02, '1'),
            (0x0B, '0'),
            (0x0C, '-'),
            (0x10, 'q'),
            (0x19, 'p'),
            (0x1E, 'a'),
            (0x26, 'l'),
            (0x27, ';'),
            (0x28, '\''),
            (0x2C, 'z'),
            (0x32, 'm'),
            (0x33, ','),
            (0x34, '.'),
            (0x35, '/'),
        ] {
            assert_eq!(char_from_scan(scan), Some(c), "{scan:#x}");
        }
    }

    #[test]
    fn extended_scan_codes_do_not_become_printable_keys() {
        // Numpad slash: VK_DIVIDE (0x6F), extended, scan 0x35.
        assert_eq!(code_of(0x6F, 0x35, true), None);
        // Numpad digits.
        assert_eq!(code_of(0x61, 0x4F, false), None);
    }

    #[test]
    fn named_keys() {
        for (vk, n) in [
            (VK_SPACE, "Space"),
            (VK_RETURN, "Enter"),
            (VK_TAB, "Tab"),
            (VK_BACK, "Backspace"),
            (VK_DELETE, "Delete"),
            (VK_ESCAPE, "Esc"),
            (VK_LEFT, "Left"),
            (VK_RIGHT, "Right"),
            (VK_UP, "Up"),
            (VK_DOWN, "Down"),
            (VK_HOME, "Home"),
            (VK_END, "End"),
        ] {
            assert_eq!(code_of(vk, 0, false).unwrap(), n);
            // Extended or not, the name is the same (numpad Enter, arrows).
            assert_eq!(code_of(vk, 0x1C, true).unwrap(), n);
        }
    }

    #[test]
    fn modifiers_and_unknown_keys_are_not_sent() {
        for vk in [
            VK_SHIFT, VK_LSHIFT, VK_RSHIFT, VK_CONTROL, VK_MENU, VK_LWIN, 0x14, 0x90,
        ] {
            assert!(translate(vk, 0, false, none(), None).is_none(), "{vk:#x}");
        }
        // F1, PageUp, IME process key, Unicode packet.
        for vk in [0x70, 0x21, 0xE5, 0xE7] {
            assert!(translate(vk, 0, false, none(), None).is_none(), "{vk:#x}");
        }
    }

    #[test]
    fn capslock_never_counts_as_shift() {
        let mut st = [0u8; 256];
        let light_only = Mods {
            caps_lock: true,
            ..Mods::default()
        };
        st[0x14] = 0x01; // CapsLock toggled on (low bit): reported as the light only
        assert_eq!(mods_from_state(&st), light_only);
        st[0x14] = 0x81; // even while the key itself is held
        assert_eq!(mods_from_state(&st), light_only);
        st[VK_LSHIFT as usize] = 0x80;
        assert!(mods_from_state(&st).shift);
        let mut st = [0u8; 256];
        st[VK_RSHIFT as usize] = 0x80;
        assert!(mods_from_state(&st).shift);
        let mut st = [0u8; 256];
        st[VK_CONTROL as usize] = 0x80;
        st[VK_LWIN as usize] = 0x80;
        let m = mods_from_state(&st);
        assert!(m.ctrl && m.meta && !m.shift && !m.alt);
    }

    #[test]
    fn text_may_reflect_capslock_but_shift_does_not() {
        // CapsLock on, no Shift: the layout types "H", shift stays false, and
        // the light travels as caps_lock for the engine to read.
        let m = Mods {
            caps_lock: true,
            ..Mods::default()
        };
        let k = translate(b'H' as u32, 0x23, false, m, Some('H')).unwrap();
        assert_eq!(k.code, "h");
        assert_eq!(k.text, Some('H'));
        assert!(!k.shift && k.caps_lock);
    }

    #[test]
    fn shift_letter_keeps_unshifted_code_and_typed_text() {
        let m = Mods {
            shift: true,
            ..Mods::default()
        };
        let k = translate(b'H' as u32, 0x23, false, m, Some('H')).unwrap();
        assert_eq!(k.code, "h");
        assert_eq!(k.text, Some('H'));
        assert!(k.shift && !k.ctrl && !k.alt && !k.meta);
        let k = translate(0xBF, 0x35, false, m, Some('?')).unwrap();
        assert_eq!(k.code, "/");
        assert_eq!(k.text, Some('?'));
    }

    #[test]
    fn shift_space_and_ctrl_combos() {
        let m = Mods {
            shift: true,
            ..Mods::default()
        };
        let k = translate(VK_SPACE, 0x39, false, m, Some(' ')).unwrap();
        assert_eq!((k.code.as_str(), k.shift), ("Space", true));
        let m = Mods {
            ctrl: true,
            ..Mods::default()
        };
        let k = translate(b'C' as u32, 0x2E, false, m, None).unwrap();
        assert_eq!((k.code.as_str(), k.ctrl, k.text), ("c", true, None));
    }

    #[test]
    fn capslock_light_is_the_toggle_bit_not_the_down_bit() {
        let mut state = [0u8; 256];
        state[0x14] = 0x80;
        assert!(!mods_from_state(&state).caps_lock, "held but light off");
        state[0x14] = 0x01;
        assert!(mods_from_state(&state).caps_lock);
    }

    #[test]
    fn text_from_to_unicode_results() {
        assert_eq!(text_from_utf16(&[b'a' as u16]), Some('a'));
        assert_eq!(text_from_utf16(&[0x0001]), None); // Ctrl+A
        assert_eq!(text_from_utf16(&[0x000D]), None); // Enter
        assert_eq!(text_from_utf16(&[0x0009]), None); // Tab
        assert_eq!(text_from_utf16(&[]), None);
        assert_eq!(text_from_utf16(&[b'a' as u16, b'b' as u16]), None);
        assert_eq!(text_from_utf16(&[0xD83D, 0xDE00]), Some('\u{1F600}'));
        assert_eq!(text_from_utf16(&[0xD83D]), None); // lone surrogate
    }
}
