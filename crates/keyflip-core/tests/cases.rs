//! Runs specs/001-mixed-input/acceptance.md's machine-readable cases (tests/cases.txt).

use keyflip_core::{bundled_engine, parse_keys, Options};

fn play(keys: &str) -> String {
    let mut engine = bundled_engine(Options::default());
    let mut committed = String::new();
    for ev in parse_keys(keys).expect("valid notation") {
        committed.push_str(&engine.handle_key(ev).commit);
    }
    committed
}

#[test]
fn acceptance_cases() {
    let cases = include_str!("cases.txt");
    let mut failures = Vec::new();
    let mut count = 0;
    for line in cases.lines() {
        let line = line.trim_end_matches('\r');
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut cols = line.split('\t');
        let id = cols.next().unwrap();
        let keys = cols.next().unwrap_or("");
        let expected = cols.next().unwrap_or("").replace("{sp}", " ");
        let expected = expected.as_str();
        count += 1;
        let got = play(keys);
        if got != expected {
            failures.push(format!("{id}: keys {keys:?} expected {expected:?} got {got:?}"));
        }
    }
    assert!(count > 0, "no cases found");
    assert!(
        failures.is_empty(),
        "{} of {count} cases failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
