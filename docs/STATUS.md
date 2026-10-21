# Current status

Updated: 2026-10-21.

## In one sentence

The engine and the macOS shell are complete and in daily use by the maintainer; the
repository is public. macOS is the only supported platform (ADR-0008). Learning
(004-user-lexicon) and the release script are new and wait for hand testing and the
maintainer's Apple Developer setup.

## Verified

- Core: 54 acceptance cases, 32 behavior tests, 8 user-model behavior tests, 5 user
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
- User lexicon (UL-1 to UL-12): learned picks, user phrases, English completion learned
  from typing, plain-text files with corrupt-file recovery; engine and file tests pass.
- Evaluation: 4,940 Chinese phrases with 0 leaking into English (89.2% exactly
  correct); 2,858 Chinese-English-Chinese sentences with 0 errors; 21 of 10,000
  English words misjudged (0.2%). Evaluation runs without a user model.
- macOS release build signs with the hardened runtime and a timestamp, and runs
  (checked with a development certificate).

## Not verified

- Hand test of the macOS checklist items added for the grid, caret position, CapsLock,
  MI-24, completion panel, and the user lexicon menu and learning.
- Notarization (needs a Developer ID certificate).
- The input source switcher icon (macOS icon cache).

## Blockers

- Release: the Developer ID Application certificate and `notarytool` profile can only
  be created by the maintainer (steps in `macos/release.sh`).

## Next action

1. Maintainer: walk the new checklist items in `specs/002-platform-shells/macos.md`.
2. Maintainer: create the Developer ID certificate and notary profile; then run
   `macos/release.sh` and publish the zip (roadmap item 12).
3. Fix every failure: if it is a judgment problem, add an acceptance case first.

## Working notes

- The authoritative copy is the GitHub repository `Hsiang-Tang/keyflip`.
- Commit identity: the GitHub noreply address configured for the repository.
