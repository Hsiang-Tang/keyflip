//! Learning from picks and user phrases (004-user-lexicon).

use std::sync::{Arc, Mutex};

use keyflip_core::user::UserModel;
use keyflip_core::{parse_keys, Engine, English, Lexicon, Options, Snapshot};

struct Fixture {
    lex: Arc<Lexicon>,
    eng: Arc<English>,
    user: Arc<Mutex<UserModel>>,
}

impl Fixture {
    fn new() -> Fixture {
        Fixture {
            lex: Arc::new(Lexicon::bundled()),
            eng: Arc::new(English::bundled()),
            user: Arc::new(Mutex::new(UserModel::default())),
        }
    }

    fn engine(&self) -> Engine {
        Engine::new(self.lex.clone(), self.eng.clone(), Options::default())
            .with_user(self.user.clone())
    }
}

fn feed(e: &mut Engine, keys: &str) -> Snapshot {
    let mut last = Snapshot::default();
    for ev in parse_keys(keys).unwrap() {
        last = e.handle_key(ev);
    }
    last
}

/// Types `keys`, then everything up to `<E>` that commits.
fn commit(e: &mut Engine, keys: &str) -> String {
    let mut out = String::new();
    for ev in parse_keys(keys).unwrap() {
        out.push_str(&e.handle_key(ev).commit);
    }
    out
}

/// With the window open, moves to `text` and picks it.
fn pick(e: &mut Engine, snap: Snapshot, text: &str) {
    let items = snap.candidates.expect("window open").items;
    let idx = items.iter().position(|i| i == text).unwrap_or_else(|| panic!("{text} in {items:?}"));
    for _ in 0..idx {
        feed(e, "<D>");
    }
    feed(e, "<E>");
}

const MAN_MAN_DE: &str = "a04a042k7";

#[test]
fn a_picked_character_comes_back_after_the_same_syllable_only() {
    let f = Fixture::new();
    let mut e = f.engine();
    assert_eq!(commit(&mut e, &format!("{MAN_MAN_DE}<E>")), "慢慢的");
    let snap = feed(&mut e, &format!("{MAN_MAN_DE}<D>"));
    pick(&mut e, snap, "地");
    assert_eq!(commit(&mut e, "<E>"), "慢慢地");

    assert_eq!(commit(&mut e, &format!("{MAN_MAN_DE}<E>")), "慢慢地", "learned");
    assert_eq!(commit(&mut e, "su32k7<E>"), "你的", "other context unchanged");
    let first = feed(&mut e, &format!("{MAN_MAN_DE}<D>")).candidates.unwrap().items[0].clone();
    assert_eq!(first, "地", "listed first");
}

#[test]
fn a_character_picked_three_times_is_preferred_everywhere() {
    let f = Fixture::new();
    let mut e = f.engine();
    for round in 1..=3 {
        let snap = feed(&mut e, &format!("{MAN_MAN_DE}<D>"));
        pick(&mut e, snap, "地");
        commit(&mut e, "<E>");
        if round < 3 {
            assert_eq!(commit(&mut e, "su32k7<E>"), "你的", "after {round} picks");
        }
    }
    assert_eq!(commit(&mut e, "su32k7<E>"), "你地");
}

#[test]
fn a_picked_word_is_preferred_and_listed_first() {
    let f = Fixture::new();
    let mut e = f.engine();
    let snap = feed(&mut e, "su3cl3<L><L><D>");
    let items = snap.clone().candidates.unwrap().items;
    let word = items
        .iter()
        .filter(|i| i.chars().count() == 2)
        .nth(1)
        .expect("a second two-character word")
        .clone();
    pick(&mut e, snap, &word);
    assert_eq!(commit(&mut e, "<E>"), word);

    assert_eq!(commit(&mut e, "su3cl3<E>"), word);
    let first = feed(&mut e, "su3cl3<L><L><D>").candidates.unwrap().items[0].clone();
    assert_eq!(first, word);
}

#[test]
fn picks_that_are_never_committed_are_not_learned() {
    let f = Fixture::new();
    let mut e = f.engine();
    let snap = feed(&mut e, &format!("{MAN_MAN_DE}<D>"));
    pick(&mut e, snap, "地");
    feed(&mut e, "<X>");
    let snap = feed(&mut e, &format!("{MAN_MAN_DE}<D>"));
    pick(&mut e, snap, "地");
    feed(&mut e, "<B><B><B><X>");
    assert_eq!(f.user.lock().unwrap().learned_len(), 0);
    assert_eq!(commit(&mut e, &format!("{MAN_MAN_DE}<E>")), "慢慢的");
}

#[test]
fn the_pre_check_never_learns() {
    let f = Fixture::new();
    let mut e = f.engine();
    let snap = feed(&mut e, &format!("{MAN_MAN_DE}<D>"));
    pick(&mut e, snap, "地");
    let enter = parse_keys("<E>").unwrap()[0];
    assert!(e.would_handle(enter));
    assert_eq!(f.user.lock().unwrap().learned_len(), 0);
}

#[test]
fn a_user_phrase_converts_by_default_and_is_listed() {
    let f = Fixture::new();
    let bad = f.user.lock().unwrap().load_phrases("鍵翻\tㄐㄧㄢˋ ㄈㄢ\nbroken line\n");
    assert_eq!(bad, 1);
    let mut e = f.engine();
    assert_eq!(commit(&mut e, "ru04z0_<E>"), "鍵翻");
    let items = feed(&mut e, "ru04z0_<L><L><D>").candidates.unwrap().items;
    assert_eq!(items[0], "鍵翻");
}

#[test]
fn learned_choices_survive_a_round_trip_through_text() {
    let f = Fixture::new();
    let mut e = f.engine();
    let snap = feed(&mut e, &format!("{MAN_MAN_DE}<D>"));
    pick(&mut e, snap, "地");
    commit(&mut e, "<E>");
    let text = f.user.lock().unwrap().learned_tsv();

    let g = Fixture::new();
    assert_eq!(g.user.lock().unwrap().load_learned(&text), 0);
    let mut e = g.engine();
    assert_eq!(commit(&mut e, &format!("{MAN_MAN_DE}<E>")), "慢慢地");
}

fn suggestion(snap: &Snapshot) -> Option<String> {
    snap.preedit
        .iter()
        .find(|s| s.kind == keyflip_core::SegKind::Suggestion)
        .map(|s| s.text.clone())
}

#[test]
fn typed_english_words_become_completions() {
    let f = Fixture::new();
    let mut e = f.engine();
    assert_eq!(suggestion(&feed(&mut e, "doc")).as_deref(), Some("tor"));
    feed(&mut e, "<X>");

    // Glued words do not count.
    commit(&mut e, "my<S-->docker_docker2<E>");
    commit(&mut e, "docker_<E>");
    assert_eq!(suggestion(&feed(&mut e, "doc")).as_deref(), Some("tor"), "once is not enough");
    feed(&mut e, "<X>");

    commit(&mut e, "su3cl3docker<E>");
    assert_eq!(suggestion(&feed(&mut e, "doc")).as_deref(), Some("ker"));
    assert_eq!(commit(&mut e, "<T><E>"), "docker");
}
