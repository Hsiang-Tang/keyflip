# keyflip architecture

```
 key ──► shell (Windows TSF / macOS IMK) ──► engine keyflip-core
              ▲                                   │
              └────────── full snapshot ◄─────────┘
```

## Components

- `crates/keyflip-core`: pure Rust engine. `Engine::handle_key(KeyEvent) -> Snapshot`.
  No files, windows, or system APIs. Contains the key table (`zhuyin.rs`), lexicon and
  Viterbi segmentation (`lexicon.rs`, `convert.rs`), English word list (`english.rs`),
  and the state machine (`engine.rs`).
- `crates/keyflip-proto`: JSON protocol types between shell and engine. The `engine`
  feature is optional and on by default; the TSF DLL turns it off so the DLL carries
  no lexicon.
- `crates/keyflip-server`: Windows engine service over a named pipe.
- `crates/keyflip-tsf`: Windows text service DLL (thin).
- `crates/keyflip-ffi`: C interface static library for macOS.
- `macos/`: Swift IMK shell (thin).
- `crates/keyflip-cli`: `play` simulator and `eval` evaluator.
- `data/`: `lexicon.tsv` (5.8 MB, derived from McBopomofo), `english.txt`,
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
- Enter commits the raw letters; Tab flips the last automatic judgment.

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
