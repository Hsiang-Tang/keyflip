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
- macOS shell: builds and installs on Apple Silicon; typing behavior not yet
  verified by hand (`macos/`, `crates/keyflip-ffi`).
- Windows shell: loads and runs on real Windows, but registration needs
  administrator rights, so typing behavior is not yet verified
  (`crates/keyflip-tsf`, `crates/keyflip-server`).

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
Full guides: `specs/002-platform-shells/macos.md` and `windows.md`.

## Documentation map

- `.specify/memory/constitution.md`: project principles.
- `specs/`: specifications, plans, tasks, acceptance cases, research.
- `docs/`: product, architecture, roadmap, status, verification log, ADRs.

## License and data sources

The source code is MIT licensed (`LICENSE`). The data files derive from McBopomofo
(MIT), libtabe (BSD-style), and wordfreq (CC BY-SA 4.0). See `NOTICE` and
`THIRD_PARTY_LICENSES.md`.
