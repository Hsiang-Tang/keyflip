//! `%LOCALAPPDATA%\keyflip\config.json`: `{"cjk_spacing": bool}`.
//! A missing or invalid file means defaults; nothing here may fail.

use keyflip_core::Options;

/// Parse config text. Unknown or mistyped fields are ignored.
pub fn parse_options(text: &str) -> Options {
    let mut opt = Options::default();
    if let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(text) {
        if let Some(serde_json::Value::Bool(b)) = map.get("cjk_spacing") {
            opt.cjk_spacing = *b;
        }
    }
    opt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_cjk_spacing() {
        assert!(parse_options(r#"{"cjk_spacing": true}"#).cjk_spacing);
        assert!(!parse_options(r#"{"cjk_spacing": false}"#).cjk_spacing);
    }

    #[test]
    fn bad_input_gives_defaults() {
        let d = Options::default().cjk_spacing;
        for t in ["", "not json", "[]", "null", r#"{"cjk_spacing": "yes"}"#, r#"{"other": 1}"#] {
            assert_eq!(parse_options(t).cjk_spacing, d, "{t}");
        }
    }
}
