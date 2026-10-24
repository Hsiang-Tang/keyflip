# Tasks: user lexicon and learning

- T-UL-1 User model in the core: parse, serialize, learn, rank. Requirements UL-1 to
  UL-5, UL-7, UL-10. Verification: unit tests in `user.rs`, behavior tests in
  `tests/user.rs`; evaluation unchanged. Status: done (2026-10-21).
- T-UL-2 File layer in `keyflip-ffi`: load, atomic save, corrupt copy, reload.
  Requirements UL-7 to UL-9. Verification: `cargo test -p keyflip-ffi`. Status: done
  (2026-10-21).
- T-UL-4 English completion learns typed words. Requirements UL-11, UL-12.
  Verification: behavior test `typed_english_words_become_completions`, store tests.
  Status: done (2026-10-21).
- T-UL-3 macOS menu and activation reload. Requirements UL-6, UL-9. Verification: hand
  test in `specs/002-platform-shells/macos.md`. Status: menu and learned picks hand-tested (2026-10-21);
  phrases and clear not yet tried by hand.
- T-UL-5 Short words flipped with Tab stay English. Requirements UL-13, UL-14.
  Verification: behavior test `short_words_flipped_with_tab_stay_english`, model and
  store tests; evaluation unchanged. Status: done (2026-10-24).
