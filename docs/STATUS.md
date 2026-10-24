# Current status

Updated: 2026-10-24.

## In one sentence

The engine and the macOS shell are complete and in daily use by the maintainer; the
repository is public. macOS is the only supported platform (ADR-0008). Learning
(004-user-lexicon) is hand-tested for picks and English completion. Release `v0.1.1`
(signed, notarized) is published on GitHub.

## Verified

- Core: 54 acceptance cases, 32 behavior tests, 9 user-model behavior tests, 6 user
  model unit tests, 6 protocol tests, 5 C-interface tests; `cargo test --workspace`
  passes.
- Identifier protection (MI-20, MI-21): `_` and `@` keep an English word going, and
  URLs, paths, and identifiers keep half-width punctuation in full-width mode.
- Enter commits what is shown (MI-7, changed by the maintainer); Chinese may follow an
  English word with no Space (MI-23): glued sentences 1.0% wrong, down from 100%.
- Zhuyin abbreviations with Tab (MI-24); English completion with Tab or Right (TP-1 to
  TP-8). Known limit: the YouTube Music search box.
- CapsLock light means uppercase English letters (MI-25, ADR-0006).
- Candidate order: common (Big5) characters come before rare ones on a frequency tie
  (MI-10).
- Expanded candidate grid (MI-22).
- User lexicon (UL-1 to UL-14): learned picks, user phrases, English completion learned
  from typing, short words kept as English after a Tab flip (`ai`), plain-text files
  with corrupt-file recovery; engine and file tests pass.
- Evaluation: 4,940 Chinese phrases with 0 leaking into English (89.2% exactly
  correct); 2,858 Chinese-English-Chinese sentences with 0 errors; 21 of 10,000
  English words misjudged (0.2%). Evaluation runs without a user model.
- Password fields (PS-7): keys pass through under secure input; the keychain prompt
  accepted the password after the fix.
- Release: Developer ID signed, notarized, stapled (`dist/keyflip-0.1.1-macos-arm64.zip`).

## Not verified

- Hand test of the macOS checklist items added for the grid, caret position, CapsLock,
  MI-24, completion panel, and the user lexicon menu and learning.
- The input source switcher icon (macOS icon cache).

## Blockers

- None.

## Next action

1. Maintainer: walk the new checklist items in `specs/002-platform-shells/macos.md`.
3. Fix every failure: if it is a judgment problem, add an acceptance case first.

## Working notes

- The authoritative copy is the GitHub repository `Hsiang-Tang/keyflip`.
- Commit identity: the GitHub noreply address configured for the repository.
