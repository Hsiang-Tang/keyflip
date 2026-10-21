# Tasks: mixed input

Format: task, requirements covered, verification, status.

- T-MI-1 Key table and syllable-prefix set. Requirements MI-2, MI-3. Verification:
  `cargo test -p keyflip-core`. Status: done.
- T-MI-2 Lexicon builder and synthetic lexicon (McBopomofo). Requirement MI-10.
  Verification: `tools/test_build_lexicon.py`. Status: done.
- T-MI-3 Engine state machine and judgment rules. Requirements MI-1 to MI-9 and MI-13 to
  MI-18. Verification: `crates/keyflip-core/tests/cases.txt` (21 cases) and
  `tests/behavior.rs` (18 tests). Status: done.
- T-MI-4 Evaluation tool. Requirement MI-19. Verification:
  `cargo run --release -p keyflip-cli -- eval`. Status: done.
- T-MI-5 Setting for space between Chinese and English. Requirement MI-6. Verification: new
  acceptance case. Status: engine option exists; shell settings UI not done.
- T-MI-6 Identifier protection (ruhi's classification rules, KnowType's range list).
  Requirement: new. Status: not started; see `docs/ROADMAP.md`.
- T-MI-7 Tab prediction (version 2). Status: not started.

Rule: never delete or loosen a case to make an implementation pass. New behavior gets a
case before code.
