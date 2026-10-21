//! Behavior the shells depend on: which keys are consumed, and what is committed when.

use keyflip_core::{bundled_engine, parse_keys, Engine, KeyCode, KeyEvent, Options, Snapshot};

fn engine() -> Engine {
    bundled_engine(Options::default())
}

fn feed(engine: &mut Engine, keys: &str) -> Snapshot {
    let mut last = Snapshot::default();
    for ev in parse_keys(keys).unwrap() {
        last = engine.handle_key(ev);
    }
    last
}

fn commit_of(keys: &str) -> String {
    let mut e = engine();
    let mut out = String::new();
    for ev in parse_keys(keys).unwrap() {
        out.push_str(&e.handle_key(ev).commit);
    }
    out
}

#[test]
fn idle_enter_tab_and_space_belong_to_the_application() {
    let mut e = engine();
    for keys in ["<E>", "<T>", "<_>", "<S-E>", "<B>", "<X>", "<L>"] {
        assert!(!feed(&mut e, keys).handled, "{keys} must pass through when idle");
    }
}

#[test]
fn letters_are_always_consumed_so_shell_never_double_types() {
    let mut e = engine();
    for ev in parse_keys("su3Hsiang").unwrap() {
        assert!(e.handle_key(ev).handled);
    }
}

#[test]
fn shift_enter_while_composing_only_confirms_then_passes_through() {
    let mut e = engine();
    feed(&mut e, "su3cl3");
    let first = feed(&mut e, "<S-E>");
    assert!(first.handled);
    assert_eq!(first.commit, "你好");
    let second = feed(&mut e, "<S-E>");
    assert!(!second.handled, "second Shift+Enter is the application's newline");
}

#[test]
fn enter_twice_confirms_then_passes_through() {
    let mut e = engine();
    feed(&mut e, "Hsiang");
    assert_eq!(feed(&mut e, "<E>").commit, "Hsiang");
    assert!(!feed(&mut e, "<E>").handled);
}

#[test]
fn ctrl_combo_commits_while_composing_and_passes_through_when_idle() {
    let mut e = engine();
    let mut ctrl_c = KeyEvent::plain(KeyCode::Char('c'));
    ctrl_c.ctrl = true;
    assert!(!e.handle_key(ctrl_c).handled);
    feed(&mut e, "su3cl3");
    let snap = e.handle_key(ctrl_c);
    assert!(snap.handled);
    assert_eq!(snap.commit, "你好");
    assert!(!e.handle_key(ctrl_c).handled);
}

#[test]
fn default_punctuation_is_half_width_and_shift_space_makes_it_full_width() {
    assert_eq!(commit_of("su3cl3,<E>"), "你好,");
    assert_eq!(commit_of("su3cl3<S-_>,<E>"), "你好，");
    assert_eq!(commit_of("su3cl3<S-_><S-_>,<E>"), "你好,");
}

#[test]
fn full_width_flag_is_reported() {
    let mut e = engine();
    assert!(feed(&mut e, "<S-_>").full_width);
    assert!(!feed(&mut e, "<S-_>").full_width);
}

#[test]
fn down_lists_candidates_for_the_unit_the_left_arrow_selected() {
    let mut e = engine();
    feed(&mut e, "su3cl3<L>");
    let snap = feed(&mut e, "<D>");
    let items = snap.candidates.expect("candidate window").items;
    assert_eq!(items.first().map(String::as_str), Some("好"));
    assert!(items.len() > 1);
}

#[test]
fn number_key_picks_a_candidate_and_enter_confirms() {
    let mut e = engine();
    feed(&mut e, "su3cl3<L><D>2");
    let out = feed(&mut e, "<E>").commit;
    assert!(out.starts_with('你') && out.chars().count() == 2 && out != "你好", "{out}");
}

#[test]
fn escape_closes_candidates_but_keeps_the_composition() {
    let mut e = engine();
    feed(&mut e, "su3cl3<L><D>");
    let snap = feed(&mut e, "<X>");
    assert!(snap.candidates.is_none());
    assert_eq!(snap.preedit_text(), "你好");
}

#[test]
fn escape_without_candidates_clears_the_composition() {
    let mut e = engine();
    feed(&mut e, "su3cl3");
    assert_eq!(feed(&mut e, "<X>").preedit_text(), "");
}

#[test]
fn backspace_edits_pending_zhuyin_then_whole_units() {
    let mut e = engine();
    feed(&mut e, "su");
    assert_eq!(feed(&mut e, "<B>").preedit_text(), "ㄋ");
    feed(&mut e, "u3cl3");
    assert_eq!(feed(&mut e, "<B>").preedit_text(), "你");
}

#[test]
fn tab_swaps_a_wrong_guess_in_both_directions() {
    // "ai" + space reads as the zhuyin syllable by default; Tab turns it into English.
    let mut e = engine();
    let chinese = feed(&mut e, "ai_").preedit_text();
    assert!(chinese.chars().all(|c| !c.is_ascii()), "{chinese}");
    assert_eq!(feed(&mut e, "<T>").preedit_text(), "ai ");
    assert_eq!(feed(&mut e, "<T>").preedit_text(), chinese);
}

#[test]
fn enter_commits_raw_letters_for_unfinished_zhuyin() {
    assert_eq!(commit_of("ai<E>"), "ai");
    assert_eq!(commit_of("su<E>"), "su");
}

#[test]
fn numbers_followed_by_chinese_are_split() {
    // 3 + 樓 (ㄌㄡˊ = x . 6)
    assert_eq!(commit_of("3x.6<E>"), "3樓");
    assert_eq!(commit_of("2026su06<E>"), "2026年");
}

#[test]
fn cursor_is_reported_in_utf16_units() {
    let mut e = engine();
    let snap = feed(&mut e, "su3cl3Hi");
    assert_eq!(snap.preedit_text(), "你好Hi");
    assert_eq!(snap.cursor, 4);
}

#[test]
fn cjk_spacing_option_adds_spaces_only_on_commit() {
    let mut e = bundled_engine(Options {
        cjk_spacing: true,
        ..Options::default()
    });
    let mut out = String::new();
    for ev in parse_keys("su3cl3Hsiang<E>").unwrap() {
        out.push_str(&e.handle_key(ev).commit);
    }
    assert_eq!(out, "你好 Hsiang");
}

#[test]
fn up_arrow_is_swallowed_while_composing_so_hosts_do_not_clear_the_text() {
    let mut e = engine();
    assert!(!feed(&mut e, "<U>").handled, "idle Up belongs to the application");
    feed(&mut e, "su3cl3");
    let snap = feed(&mut e, "<U>");
    assert!(snap.handled);
    assert_eq!(snap.preedit_text(), "你好");
}
