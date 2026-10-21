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
fn enter_commits_unfinished_zhuyin_as_shown_and_tab_first_gives_raw_letters() {
    assert_eq!(commit_of("ai<E>"), "ㄇㄛ");
    assert_eq!(commit_of("su<E>"), "ㄋㄧ");
    assert_eq!(commit_of("ai<T><E>"), "ai");
    assert_eq!(commit_of("su<T><E>"), "su");
}

#[test]
fn every_commit_path_sends_unfinished_zhuyin_as_shown() {
    let mut e = engine();
    feed(&mut e, "1");
    assert_eq!(feed(&mut e, "<S-E>").commit, "ㄅ");
    let mut ctrl_c = KeyEvent::plain(KeyCode::Char('c'));
    ctrl_c.ctrl = true;
    feed(&mut e, "1");
    assert_eq!(e.handle_key(ctrl_c).commit, "ㄅ");
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

#[test]
fn notation_types_shifted_symbols_with_their_physical_key() {
    let evs = parse_keys("<S-->@:").unwrap();
    let got: Vec<(KeyCode, Option<char>, bool)> =
        evs.iter().map(|e| (e.code, e.text, e.shift)).collect();
    assert_eq!(
        got,
        vec![
            (KeyCode::Char('-'), Some('_'), true),
            (KeyCode::Char('2'), Some('@'), true),
            (KeyCode::Char(';'), Some(':'), true),
        ]
    );
}

#[test]
fn protected_url_shows_half_width_while_typing_in_full_width_mode() {
    let mut e = engine();
    let snap = feed(&mut e, "<S-_>https://ai.c");
    assert_eq!(snap.preedit_text(), "https://ai.c");
}

/// Opens the window on a syllable with many candidates.
fn open_window(e: &mut Engine) -> Snapshot {
    feed(e, "u4");
    feed(e, "<D>")
}

#[test]
fn right_expands_the_candidate_window_into_a_grid() {
    let mut e = engine();
    let compact = open_window(&mut e);
    let c = compact.candidates.expect("window open");
    assert!(!c.expanded);
    assert!(c.items.len() > 9, "needs more than one row to test");
    let grid = feed(&mut e, "<R>").candidates.expect("still open");
    assert!(grid.expanded);
    assert_eq!(grid.selected, 0, "expanding does not move the selection");
}

#[test]
fn grid_arrows_move_by_item_and_by_row() {
    let mut e = engine();
    open_window(&mut e);
    feed(&mut e, "<R>");
    assert_eq!(feed(&mut e, "<R>").candidates.unwrap().selected, 1);
    assert_eq!(feed(&mut e, "<D>").candidates.unwrap().selected, 10);
    assert_eq!(feed(&mut e, "<L>").candidates.unwrap().selected, 9);
    assert_eq!(feed(&mut e, "<U>").candidates.unwrap().selected, 0);
}

#[test]
fn grid_digit_picks_from_the_selected_row() {
    let mut e = engine();
    let items = open_window(&mut e).candidates.unwrap().items;
    feed(&mut e, "<R><D>");
    let snap = feed(&mut e, "2");
    assert!(snap.candidates.is_none(), "picking closes the window");
    let out = feed(&mut e, "<E>").commit;
    assert_eq!(out, items[10]);
}

#[test]
fn esc_in_the_grid_returns_to_the_compact_list_first() {
    let mut e = engine();
    open_window(&mut e);
    feed(&mut e, "<R>");
    let snap = feed(&mut e, "<X>");
    assert!(!snap.candidates.expect("still open").expanded);
    assert!(feed(&mut e, "<X>").candidates.is_none());
    assert!(!feed(&mut e, "<E>").commit.is_empty(), "composition survives");
}

#[test]
fn left_in_the_compact_list_still_closes_and_moves_the_cursor() {
    let mut e = engine();
    open_window(&mut e);
    let snap = feed(&mut e, "<L>");
    assert!(snap.candidates.is_none());
}

fn suggestion(snap: &Snapshot) -> Option<String> {
    snap.preedit
        .iter()
        .find(|s| s.kind == keyflip_core::SegKind::Suggestion)
        .map(|s| s.text.clone())
}

#[test]
fn english_prefix_shows_a_completion_after_the_cursor() {
    let mut e = engine();
    let snap = feed(&mut e, "docu");
    assert_eq!(suggestion(&snap).as_deref(), Some("ment"));
    assert_eq!(snap.cursor, 4, "the cursor stays before the suggestion");
}

#[test]
fn tab_accepts_the_completion_and_enter_never_commits_it() {
    assert_eq!(commit_of("docu<T><E>"), "document");
    assert_eq!(commit_of("docu<E>"), "docu");
    assert_eq!(commit_of("docu<T>s<E>"), "documents");
}

#[test]
fn the_completion_follows_the_prefix() {
    let mut e = engine();
    feed(&mut e, "envi");
    assert_eq!(suggestion(&feed(&mut e, "r")).as_deref(), Some("onment"));
}

#[test]
fn no_completion_for_short_words_chinese_paths_or_identifiers() {
    for keys in ["ai", "su3cl3", "~/doc", "my<S-->docu", "Docu", "docu<L>"] {
        let mut e = engine();
        assert_eq!(suggestion(&feed(&mut e, keys)), None, "{keys}");
    }
    assert_eq!(commit_of("ai<T><E>"), "ai", "Tab still flips without a suggestion");
}
