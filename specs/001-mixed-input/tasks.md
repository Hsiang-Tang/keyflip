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
- T-MI-6 Identifier protection. Requirements MI-20, MI-21. Verification: acceptance cases
  I01 to I11 and two behavior tests. Status: done (2026-10-21) for joiners and punctuation
  width; a protected-range list in the snapshot waits for a shell that needs it.
- T-MI-8 Expanded candidate grid. Requirement MI-22. Verification: five behavior tests,
  a protocol compatibility test, and a rendered screenshot of the macOS grid. Status: done
  in the engine and the macOS shell (2026-10-21); the Windows candidate window still draws
  the page that holds the selection, not the full grid.
- T-MI-9 Chinese right after an English word with no Space. Requirement MI-23.
  Verification: cases G01 to G07 and the eval lines "Glued" and "English + digit".
  Status: done (2026-10-21).
- T-MI-10 Zhuyin abbreviations with Tab. Requirement MI-24. Verification: cases R03 to
  R07. Status: done (2026-10-21).
- T-MI-11 CapsLock light means uppercase letters. Requirement MI-25, ADR-0006.
  Verification: cases K01 to K04, protocol and Windows key-map tests. Status: done
  (2026-10-21) in the engine and macOS shell; Windows reads the light, unverified.
- T-MI-12 Common characters before rare ones on a frequency tie. Requirement MI-10.
  Verification: behavior test `rare_characters_never_outrank_common_ones_on_a_frequency_tie`
  and builder tests. Status: done (2026-10-21).
- T-MI-7 Tab prediction. Moved to `specs/003-tab-prediction/`; English completion done
  (2026-10-21).

Rule: never delete or loosen a case to make an implementation pass. New behavior gets a
case before code.
