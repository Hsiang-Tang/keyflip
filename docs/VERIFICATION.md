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

### 2026-10-21 Zhuyin abbreviations (MI-24)

- Maintainer report: ㄨㄞㄋ came out as `j9s`. Automatic detection was rejected because
  identifiers such as `x86` share the shape; Tab converts instead. Cases R03 to R07
  written first; R03 and R05 failed before the change.
- `cargo test --workspace` passes (50 cases); evaluation output identical.

### 2026-10-21 English completion with Tab (003-tab-prediction)

- Four behavior tests written first (suggestion after the cursor, Tab accepts and Enter
  never commits it, the suggestion follows the prefix, none for short words, Chinese,
  paths, identifiers, capitals, or a cursor inside the text); they did not compile
  before the change. `cargo test --workspace` passes.
- Evaluation output is identical: the suggestion never reaches committed text.
- Samples: `conf` suggests `erence`, `envi` `ronment`, `impl` `ement`, `docu` `ment`
  (the shorter base wins over the more frequent `documents`).
- macOS build installed; dimmed drawing not yet seen in real hosts.

### 2026-10-21 macOS hand test feedback: suggestion display and panel position

- Hand test: the host drew the whole marked text reversed, so the dimmed suggestion
  looked typed. The suggestion now leaves the marked text and appears in the panel
  (`doc` dimmed, `tor` strong, "Tab"); rendered with the panel code on its own and
  inspected.
- Hand test: the candidate window opened far from the text. The shell passed the
  document offset of the marked range to `attributes(forCharacterIndex:)`, which IMK
  reads as an offset into the marked text. It now passes the cursor inside the
  composition, then 0. Waiting for the next hand test.
- Hand test: the menu icon had not changed. Text Input Sources reports
  `MenuIcon.pdf` for both the input method and its mode, so the bundle is right and the
  menu agent was showing a cached icon; `TextInputMenuAgent` was restarted.

### 2026-10-21 Menu icon and panel styling

- Maintainer feedback: the icon should be bare arrows like the system's menu glyphs,
  and the suggestion box looked off-center. The icon is now two opposite arrows with no
  frame (stroke outlines filled, 1.6 pt). The panel is an opaque rounded card with even
  padding; the completion is semibold, the typed part dimmed, and "Tab" is a smaller
  hint lifted to the word's center. Both rendered and inspected; the candidate list uses
  the same card.

### 2026-10-21 CapsLock light means uppercase (MI-25, ADR-0006)

- Maintainer request: all-caps words without holding Shift. ADR-0006 amends ADR-0002:
  the light means uppercase letters, never a language switch.
- Cases K01 to K04 were written with the change but not run before it, so they have no
  recorded failing run; without the engine branch the light is ignored and K01 would
  give `docker`.
- The key notation gained `<C>`. The Windows key map reads the light from the toggle
  bit; its existing "CapsLock never counts as Shift" test now also checks the light is
  reported, and a new test checks the toggle bit, not the down bit, is read.
- `cargo test --workspace` passes; evaluation output identical.

### 2026-10-21 Common characters before rare ones (MI-10)

- Maintainer report: very rare characters showed early in the candidate list. Cause:
  uncounted characters were ordered by code point, so extension-A characters (U+3400)
  came before common ones (U+4E00). 384 of 1,396 single-syllable readings had a rare
  character on page 1 while a common character was pushed to a later page.
- Fix: the builder keeps upstream's class column (`big5` or `utf8`) as a fourth
  `lexicon.tsv` column (1 = non-Big5); the engine breaks count ties with it.
- The behavior test failed before the change (㧊 at 6, 犮 at 14) and passes after. The
  first draft of the test used 坺, which upstream also marks `utf8`, so it still failed
  after the change; it was corrected to 犮, a Big5 character. The recorded failing run
  shows 犮 at 14 under the old order, so the corrected test also fails before the
  change and is not looser.
- Rebuilt from the same pinned sources (all four sha256 values match `SOURCES.txt`);
  the first three columns are byte-identical to the previous file. 12,149 entries are
  marked rare. After the change 1 reading remains (ㄎㄨㄤˋ: 鑛 has count 1, so frequency
  correctly wins).
- `cargo test --workspace` and the builder tests pass; evaluation output identical.

### 2026-10-21 Right arrow accepts a completion (TP-8) and switcher icon

- Maintainer report: in the YouTube Music search box, Tab after `complicat` committed
  `complicat` instead of completing. A shell log showed the engine accepted the
  suggestion, then the host asked for a commit 9 ms later. A browser test page logging
  key and composition events showed the page receives `keydown key=Tab` (code 229)
  before the composition update; a plain input and Notes complete correctly, so that
  page acts on Tab during composition. keyflip cannot hide the key from the page.
- Fix: Right arrow accepts a shown suggestion like Tab (TP-8). The behavior test failed
  before the change. Its first draft compared the whole preedit, which includes the
  suggestion segment; it was corrected to compare only typed text (a test bug, not a
  loosened expectation).
- The macOS shell now logs (event names only, no text) when the host asks for a commit:
  `log show --predicate 'subsystem == "com.hsiangtang.inputmethod.keyflip"'`.
- The input source switcher (Control+Space) showed the system's generic icon: it reads
  the app icon, which did not exist. `macos/make-icon.swift` now also draws
  `AppIcon.icns` (the same arrows on a light rounded square); `CFBundleIconFile` added.
  The menu bar icon is unchanged. Not yet seen in the switcher (icon cache).
- Hand test: Right does not help in the YouTube Music search box either (the log
  shows the same host commit after Right). Other web pages complete with Tab. Recorded
  as a known limit in TP-8; a shell repair was drafted, never tested, and removed.
- `cargo test --workspace` passes; evaluation output identical.

### 2026-10-21 User lexicon and learning (004-user-lexicon), macOS only, release script

- macOS only (ADR-0008): Windows code frozen, roadmap item 7 dropped.
- User model (ADR-0007): behavior tests in `crates/keyflip-core/tests/user.rs` were
  written before the code and failed first. The first conversion design boosted only
  the learned entry; three tests kept failing because phrases such as 慢慢的 and 你的
  outscored it, so the design moved to per-syllable preferences with a mismatch cost
  (recorded in the plan). Tests were not changed to pass.
- English completion learning (UL-11, maintainer request `doc` -> `docker`): the
  behavior test failed first (`tor` instead of `ker`).
- File layer: 5 tests in `keyflip-ffi` (template, save, corrupt copy and rewrite,
  reload of hand edits, clear with `.bak`). On this Mac the first launch created
  `~/Library/Application Support/keyflip/phrases.txt`.
- Release: `macos/build.sh --sign` signs with the hardened runtime and a timestamp;
  checked with the maintainer's Apple Development certificate, installed, and launched
  with no errors in the system log. Releases are Apple Silicon only: this machine's
  Rust comes from Homebrew without an x86_64 standard library. Notarization not run.
- `cargo test --workspace` passes; clippy clean; evaluation output identical.
- Hand test (maintainer): the input menu shows both items; picking 地 after 慢慢 is
  learned and comes back; after `docker` was typed twice, `doc` suggests `docker`. Not
  yet tried by hand: user phrases, clear.

### 2026-10-21 Password fields on macOS (PS-7) and the first notarized build

- Maintainer report: in the keychain password prompt, Shift letters typed but digits
  after them did not. The system log showed keyflip receiving the keys (marked text set
  and committed), so the guide's claim that macOS keeps input methods out of secure
  fields was wrong. The learned files were also written while the password was typed;
  every English word seen only once was deleted without reading it, keeping only the
  word the maintainer tested (`docker`).
- Fix: when `IsSecureEventInputEnabled()` is true the shell passes every key through,
  discards the composition without committing, and never reaches the engine.
  Shell-only; no engine test applies. After the fix the keychain prompt accepted the
  password and Developer ID signing succeeded.
- `macos/release.sh`: notarization accepted, ticket stapled, `spctl` reports
  "Notarized Developer ID". Output `dist/keyflip-0.1.0-macos-arm64.zip`
  (sha256 ca27e6b464dbc8f9ebec3de9779a5441dfc8848d1e64924a3e0e68560e2c6470). The
  notary profile uses the existing App Store Connect API key.
- Published as GitHub release `v0.1.0`; the downloaded asset has the same sha256.
- Hand test (maintainer): typing a password directly with keyflip active works.

### 2026-10-24 Short words kept as English (UL-13, UL-14)

- Maintainer report: `ai` then Enter sent ㄇㄛ (MI-7 as specified). Now a word of 2 to
  4 letters flipped with Tab and committed alone is recorded in `short-english.tsv`;
  afterwards its pending keys show as letters and commit as letters, while a tone key
  still converts (`ai3` gives 抹).
- `cargo test --workspace` passes (new: behavior test
  `short_words_flipped_with_tab_stay_english`, model test
  `short_words_round_trip_and_are_capped`, store test extended). `keyflip-cli eval`
  unchanged from the previous baseline. `cargo clippy --workspace` clean.
- Released as `v0.1.1` (build 2): notarization accepted, ticket stapled, `spctl` reports
  "Notarized Developer ID". `dist/keyflip-0.1.1-macos-arm64.zip`
  (sha256 008ef8b69859f474f76e40bc62419762f2ffa9300ab49645dceaab76b8c7881d).

### Not verified

- TSF registration (`regsvr32`) and all real typing behavior (composition, candidate
  window position, password fields, focus changes, sandboxed apps) on Windows:
  needs a human on a real machine; checklist in `specs/002-platform-shells/windows.md`.
- The install scripts have only had syntax and flow review.
- macOS typing behavior; checklist in `specs/002-platform-shells/macos.md`.
- Elevated (UAC) windows: no reference material; needs real testing.
