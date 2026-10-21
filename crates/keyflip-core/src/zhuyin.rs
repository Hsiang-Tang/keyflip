//! Standard (Dachen) zhuyin keyboard layout.

/// Zhuyin symbol typed by a physical key, or `None` for keys that are not zhuyin keys.
pub fn symbol_of_key(c: char) -> Option<char> {
    Some(match c {
        '1' => 'ㄅ',
        '2' => 'ㄉ',
        '5' => 'ㄓ',
        '8' => 'ㄚ',
        '9' => 'ㄞ',
        '0' => 'ㄢ',
        '-' => 'ㄦ',
        'q' => 'ㄆ',
        'w' => 'ㄊ',
        'e' => 'ㄍ',
        'r' => 'ㄐ',
        't' => 'ㄔ',
        'y' => 'ㄗ',
        'u' => 'ㄧ',
        'i' => 'ㄛ',
        'o' => 'ㄟ',
        'p' => 'ㄣ',
        'a' => 'ㄇ',
        's' => 'ㄋ',
        'd' => 'ㄎ',
        'f' => 'ㄑ',
        'g' => 'ㄕ',
        'h' => 'ㄘ',
        'j' => 'ㄨ',
        'k' => 'ㄜ',
        'l' => 'ㄠ',
        ';' => 'ㄤ',
        'z' => 'ㄈ',
        'x' => 'ㄌ',
        'c' => 'ㄏ',
        'v' => 'ㄒ',
        'b' => 'ㄖ',
        'n' => 'ㄙ',
        'm' => 'ㄩ',
        ',' => 'ㄝ',
        '.' => 'ㄡ',
        '/' => 'ㄥ',
        _ => return None,
    })
}

/// Tone mark for the tone keys `3 4 6 7`.
pub fn tone_of_key(c: char) -> Option<char> {
    match c {
        '3' => Some('ˇ'),
        '4' => Some('ˋ'),
        '6' => Some('ˊ'),
        '7' => Some('˙'),
        _ => None,
    }
}

/// Zhuyin symbols for a run of keys (tone keys are not allowed here).
pub fn symbols_of_keys(keys: &[char]) -> Option<String> {
    keys.iter().map(|&c| symbol_of_key(c)).collect()
}

/// Keys that double as punctuation after Chinese text.
pub fn is_punct_key(c: char) -> bool {
    matches!(c, ',' | '.' | ';' | '/' | '-')
}

/// Full-width counterpart of a half-width punctuation character.
pub fn full_width_of(c: char) -> char {
    match c {
        ',' => '，',
        '.' => '。',
        ';' => '；',
        ':' => '：',
        '?' => '？',
        '!' => '！',
        '(' => '（',
        ')' => '）',
        '/' => '／',
        '-' => '－',
        '\'' => '’',
        '"' => '”',
        '<' => '＜',
        '>' => '＞',
        _ => c,
    }
}
