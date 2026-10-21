# Current status

Updated: 2026-10-21.

## In one sentence

The core engine is complete and passes acceptance. The macOS shell builds and
installs (first successful build 2026-10-21) but has not been tested by hand. The
Windows shell cannot be registered on a machine without administrator rights, so
Windows typing behavior is unverified.

## Verified

- Core: 54 acceptance cases, 31 behavior tests, 6 protocol tests, 2 C-interface
  tests; `cargo test --workspace` passes.
- Identifier protection (MI-20, MI-21): `_` and `@` keep an English word going, and
  URLs, paths, and identifiers keep half-width punctuation in full-width mode.
- Enter commits what is shown (MI-7, changed by the maintainer); Chinese may follow an
  English word with no Space (MI-23): glued sentences 1.0% wrong, down from 100%.
- Zhuyin abbreviations with Tab (MI-24) and English completion with Tab (TP-1 to TP-7);
  Chinese associated phrases were considered and dropped.
- CapsLock light means uppercase English letters (MI-25, ADR-0006).
- Candidate order: common (Big5) characters come before rare ones on a frequency tie
  (MI-10); rare characters no longer push common ones off page 1.
- Expanded candidate grid (MI-22): Right opens a grid of every candidate; the macOS
  grid was rendered and inspected, not yet used by hand. Windows shows only the page
  holding the selection.
- Evaluation: 4,940 Chinese phrases with 0 leaking into English (89.2% exactly
  correct); 2,858 Chinese-English-Chinese sentences with 0 errors; 21 of 10,000
  English words misjudged (0.2%).
- Windows: engine service and DLL loading verified on real Windows (no administrator
  rights needed).
- Rust static libraries build for `aarch64-apple-darwin` and `x86_64-apple-darwin`.
- macOS shell compiles, signs ad hoc, and installs on Apple Silicon.

## Not verified

- All hand typing behavior on both platforms.
- Windows TSF registration (needs administrator rights).

## Blockers

- Windows: no machine with administrator rights yet.

## Next action

1. On the Mac: log out and back in, add "keyflip Zhuyin" in System Settings
   (Keyboard, Text Input, Input Sources), then walk the checklist in
   `specs/002-platform-shells/macos.md`.
2. Fix every failure: if it is a judgment problem, add an acceptance case first.
3. Then pick the next item from `docs/ROADMAP.md`.

## Working notes

- The authoritative copy is the GitHub repository `Hsiang-Tang/keyflip`.
- Commit identity: the GitHub noreply address configured for the repository.
