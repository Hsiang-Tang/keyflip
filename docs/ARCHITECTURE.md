# keyflip architecture

```
 key ──► shell (macOS IMK) ──► engine keyflip-core ◄── user model (in memory)
              ▲                      │                     ▲
              └──── full snapshot ◄──┘          keyflip-ffi loads and saves the files
```

macOS is the only supported platform (ADR-0008); the Windows crates are frozen.

## Components

- `crates/keyflip-core`: pure Rust engine. `Engine::handle_key(KeyEvent) -> Snapshot`.
  No files, windows, or system APIs. Contains the key table (`zhuyin.rs`), lexicon and
  Viterbi segmentation (`lexicon.rs`, `convert.rs`), English word list (`english.rs`),
  the state machine (`engine.rs`), and the user model (`user.rs`: user phrases,
  learned choices, learned English words; no I/O).
- `crates/keyflip-proto`: JSON protocol types between shell and engine. The `engine`
  feature is optional and on by default; the TSF DLL turns it off so the DLL carries
  no lexicon.
- `crates/keyflip-server`, `crates/keyflip-tsf`, `installer/`: Windows engine service
  and text service DLL; frozen (ADR-0008).
- `crates/keyflip-ffi`: C interface static library for macOS. Owns the user files in
  `~/Library/Application Support/keyflip/` (ADR-0007).
- `macos/`: Swift IMK shell (thin); `build.sh` builds and installs, `release.sh` signs,
  notarizes, and packages.
- `crates/keyflip-cli`: `play` simulator and `eval` evaluator.
- `data/`: `lexicon.tsv` (6.1 MB, derived from McBopomofo), `english.txt`,
  `SOURCES.txt` and `ENGLISH_SOURCE.txt` (pinned upstream versions).
- `tools/`: `build_lexicon.py`, `build_english.py`, `build-windows.sh`.

## Snapshot

After every key the engine returns the complete state: `handled`, `commit` (text to
send to the application), `preedit` (segments with text and kind), `cursor` (UTF-16),
`candidates` (list, selected item, page size), and `full_width`. Shells draw only from
the snapshot.

## Judgment flow (summary)

Details are in `specs/001-mixed-input/plan.md`. In short:

- Letters typed without Shift are stored as pending Zhuyin; when the sequence is no
  longer the prefix of any legal syllable it becomes English; a tone key ends a syllable.
- Shift plus letter starts an English segment.
- Enter commits the composition as shown; Tab flips the last automatic judgment.

## Data and licensing

- The lexicon derives from McBopomofo (MIT) and libtabe (BSD-style). The English word
  list derives from wordfreq (CC BY-SA 4.0). See `NOTICE` and `THIRD_PARTY_LICENSES.md`.
- The lexicon is reproducible with `tools/build_lexicon.py`; the upstream commit is
  pinned in `data/SOURCES.txt`. The English list is reproducible with
  `tools/build_english.py`; the package version is in `data/ENGLISH_SOURCE.txt`.

## Decision records

- `docs/adr/0001-engine-shell-split.md`
- `docs/adr/0002-no-modifier-tap-gestures.md`
- `docs/adr/0003-proto-engine-optional-feature.md`
- `docs/adr/0004-macos-c-ffi-json.md`
- `docs/adr/0005-windows-registration-needs-admin.md`
- `docs/adr/0006-capslock-is-an-uppercase-lock.md`
- `docs/adr/0007-user-data-lives-beside-the-engine.md`
- `docs/adr/0008-macos-only.md`
