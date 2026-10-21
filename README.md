# keyflip

A Zhuyin (Bopomofo) input method that decides by itself whether you are typing
Chinese or English. No Shift or CapsLock toggling.

```
My name is Hsiang in a Chinese sentence    hold Shift for H; the lowercase letters that follow join the English word
use github in a Chinese sentence           "github" cannot form valid Zhuyin syllables, so it is judged English
```

(The sentences above are shown in English; the real input is Zhuyin keystrokes
mixed with Latin letters.)

If a judgment is wrong, press Tab to flip between Chinese and English. Enter commits
exactly what the composition shows. Everything else behaves like Microsoft
New Zhuyin.

## Status

- Core engine: complete, tests pass (`crates/keyflip-core`).
- macOS shell: builds and installs on Apple Silicon and is in daily use by the
  maintainer; the full hand-test checklist is not yet complete (`macos/`,
  `crates/keyflip-ffi`).
- Learning: picks from the candidate window, your own phrases, and English words you
  type are remembered on this Mac (`specs/004-user-lexicon/`).
- macOS only. The Windows shell is frozen and unmaintained (ADR-0008).

Details: `docs/STATUS.md`.

## Try the core (no installation)

```
cargo run -p keyflip-cli -- play 'su3cl3Hsiang<E>'
cargo run --release -p keyflip-cli -- eval
```

Key notation: `specs/001-mixed-input/acceptance.md`.

## Build and install

```
cargo test --workspace
bash macos/build.sh --no-install   # build only
bash macos/build.sh                # build and install into ~/Library/Input Methods
```

Needs Rust and the Xcode command line tools. macOS needs no administrator rights.
Full guide: `specs/002-platform-shells/macos.md`. A signed, notarized release is built
with `macos/release.sh` (setup steps in its header).

## Install a release

1. Download `keyflip-<version>-macos-arm64.zip` from the GitHub releases page (Apple
   Silicon, macOS 12 or later) and unzip it.
2. Move `Keyflip.app` into `~/Library/Input Methods` (in Finder: Go > Go to Folder).
3. Log out and back in, then add it in System Settings > Keyboard > Text Input >
   Input Sources > Edit > + > Chinese, Traditional > keyflip 注音.
4. Turn off "Use the Caps Lock key to switch to and from ABC" in the same place, so
   CapsLock types capitals.

## Your dictionary

The input menu has "Open User Dictionary Folder"
(`~/Library/Application Support/keyflip/`):

- `phrases.txt`: your own words, one per line as `text<Tab>readings`, for example
  `鍵翻<Tab>ㄐㄧㄢˋ ㄈㄢ`. Picked up when you switch back to keyflip.
- `learned.tsv` and `english.tsv`: what keyflip learned. Delete a line to forget it,
  or use "Clear Learned Choices" (the old files are kept as `.bak`).

## Documentation map

- `.specify/memory/constitution.md`: project principles.
- `specs/`: specifications, plans, tasks, acceptance cases, research.
- `docs/`: product, architecture, roadmap, status, verification log, ADRs.

## License and data sources

The source code is MIT licensed (`LICENSE`). The data files derive from McBopomofo
(MIT), libtabe (BSD-style), and wordfreq (CC BY-SA 4.0). See `NOTICE` and
`THIRD_PARTY_LICENSES.md`.
