# keyflip verification log

## Required commands

In WSL, run `source ~/.cargo/env` first and keep build output outside the repo:

```
export CARGO_TARGET_DIR=/root/keyflip-target
cargo test --workspace                       # acceptance, behavior, protocol tests
cargo run --release -p keyflip-cli -- eval   # Chinese/English misjudgment rates
python3 -I -m unittest discover -s tools -p 'test_build*.py'   # lexicon builder
```

## Log

### 2026-10-21 Core engine

- All 21 acceptance cases pass; 17 behavior tests and 5 protocol tests pass.
- Evaluation (`keyflip-cli eval`):
  - 4,940 Chinese phrases: 0 converted to English; 89.2% exactly correct.
  - Chinese phrases, English words (rank above 100, three letters or more), and
    2,778 sentences built from phrases: every English word preserved, 0 sentences wrong.
  - English list of 9,884 words (lowercase plus Space): 46 misjudged (0.5%), all
    one- or two-letter ambiguous words.
- Two acceptance cases failed on the first run (trailing whitespace in the test file
  was lost while editing); the `{sp}` notation fixed them. This shows the tests can
  catch errors.

### 2026-10-21 Windows shell and engine service

- `tools/build-windows.sh` produces `keyflip_tsf.dll` (1.36 MB), `keyflip-server.exe`,
  and `keyflip-pipe-client.exe`. The DLL has no lexicon (the protocol library's engine
  feature is optional), no `comctl32` import, and exports four COM functions.
- Engine service verified on real Windows: ping; `su3cl3<E>` commits the Chinese for
  "hello"; `Hsiang<E>` commits `Hsiang`; two clients at once; a second instance exits
  immediately; the process is gone after `--stop`.
- `keyflip-tsf-smoke.exe` loads the DLL on real Windows (no registration, no
  administrator rights): all dependencies resolve, the class factory is obtainable, a
  foreign CLSID is rejected, the text service object can be created, and
  `DllCanUnloadNow` reports unloadable after release.
- Endpoint security on that machine blocked calling `cmd.exe` and `powershell.exe`
  from WSL, so the install scripts have never been run.

### 2026-10-21 C interface and macOS shell

- `keyflip-ffi` tests pass (2); the engine gained "Up arrow is swallowed while
  composing" with a behavior test; the whole workspace passes.
- Static libraries for `aarch64-apple-darwin` and `x86_64-apple-darwin` (about 25 MB
  each, need only `-lSystem -lc -lm`).

### 2026-10-21 First macOS build (Apple Silicon)

- `cargo test --workspace` passes.
- `bash macos/build.sh --no-install` succeeds: the Swift shell compiled on the first
  attempt and produced `macos/build/Keyflip.app`, signed ad hoc, signature verified.
- Link warnings: the Rust static library targets macOS 26 while the shell deployment
  target is macOS 12. Warnings only; no effect on local use.
- `macos/build.sh` fix: this Mac's Rust comes from Homebrew without rustup, so the
  script now adds the target only when rustup exists.
- `bash macos/build.sh` installed the app to `~/Library/Input Methods`. Not yet added
  as an input source and not yet used for typing.

### 2026-10-21 English word list replaced (licensing)

- `data/english.txt` now comes from wordfreq 3.1.1 (CC BY-SA 4.0) via
  `tools/build_english.py`, replacing google-10000-english (redistribution not allowed).
  Single letters other than `a` and `i`, corpus artifacts, and a short profanity blocklist
  are removed.
- `data/lexicon.tsv` rebuilt from the pinned McBopomofo commit: byte-identical.
- `cargo test --workspace` passes (all 21 acceptance cases unchanged); tools tests pass (7).
- Evaluation: 4,940 phrases, 0 leaked into English (89.2% exact); 2,858 Chinese-English-
  Chinese sentences, 0 wrong; English ranks 0-10,000: 21 of 10,000 misjudged (0.2%).
  The old list measured 46 of 9,884 (0.5%) but included all single letters, so the two
  numbers are not directly comparable.

### 2026-10-21 Identifier protection (MI-20, MI-21)

- New acceptance cases I01 to I11 written first; six failed before the change (`my_ai`
  fell back to Zhuyin, `a@su3.com` became Chinese, and URLs and paths got full-width
  `：` and `／`). All 32 cases pass after it; the four negative cases (sentence
  punctuation stays full-width) passed before and after.
- The key notation gained shifted symbols (`@`, `:`, `<S-->` for `_`) with a behavior
  test; a second behavior test checks the preedit of a URL in full-width mode.
- `cargo test --workspace` passes; tools tests pass.
- Evaluation sampling was not deterministic: phrases with the same text but different
  readings kept hash-map order, so the exact count moved between 4,404 and 4,408 across
  runs. The sample is now sorted by text and readings. With that fix, the engine before
  and after this change gives identical evaluation output: 4,940 phrases, 0 leaked,
  4,405 exact (89.2%); 2,858 mixed sentences, 0 wrong; 21 of 10,000 English words
  misjudged.

### 2026-10-21 Expanded candidate grid (MI-22)

- Requested from the first macOS hand test: Left and Down were not enough to browse
  candidates. Right now expands the window into a grid.
- Five behavior tests (expand, arrows by item and row, digit picks from the selected
  row, Esc returns to the list, Left in the compact list keeps MI-11) and a protocol test
  (a reply without `expanded` reads as not expanded). `cargo test --workspace` passes.
- `macos/build.sh --no-install` succeeds. The panel code was run on its own with 36
  candidates and screenshotted: four rows of nine in aligned columns, the selected row at
  full strength, the others dimmed, the selection highlighted.
- Not verified: the grid during real typing; Windows draws no grid.

### 2026-10-21 Enter commits what is shown (MI-7 changed)

- Maintainer report from the macOS hand test: a shown ㄅ committed as `1`. The maintainer
  chose "commit what is shown"; raw letters now need Tab first.
- Case B02 changed from `ai<E>` = `ai` to ㄇㄛ by that decision, not to make code pass;
  new cases B04 to B06. The behavior test for raw letters became a test for the new rule
  plus the Tab path, and a second test covers Shift+Enter and Ctrl combos.
- The change broke I06 (`~/ai` committed as `~／ㄇㄛ`), which exposed that letters in a
  path were still judged as Zhuyin. MI-20 now covers path and URL tokens; cases I12 to
  I14 added. `API/你好` (I11) still splits.
- `cargo test --workspace` passes (38 cases). Evaluation output is identical to the
  previous run.

### 2026-10-21 Chinese right after an English word (MI-23)

- Maintainer report: `enter` followed by Zhuyin with no Space became one long English
  word. Cases G01 to G07 written first; G01 to G03 failed.
- The evaluator gained two lines. "Glued zh+en+zh" types phrase, English word, phrase
  with no Space (first-tone tails skipped): 1,682 of 1,682 wrong before, 17 (1.0%) after.
  The remaining failures are inflection ambiguity (`lists`, `aged`, `sends`). "English +
  digit" types 2,925 words and identifiers followed by a tone digit: 7 changed before
  (`mp3`, `x86` and other short pending runs, unrelated to this change), 8 after; the new
  one is lowercase `gpt4`, accepted because Tab recovers it.
- All other evaluation lines are unchanged. `cargo test --workspace` passes (45 cases).

### Not verified

- TSF registration (`regsvr32`) and all real typing behavior (composition, candidate
  window position, password fields, focus changes, sandboxed apps) on Windows:
  needs a human on a real machine; checklist in `specs/002-platform-shells/windows.md`.
- The install scripts have only had syntax and flow review.
- macOS typing behavior; checklist in `specs/002-platform-shells/macos.md`.
- Elevated (UAC) windows: no reference material; needs real testing.
