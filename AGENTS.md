# keyflip project rules

keyflip is a Zhuyin (Bopomofo) input method. You type Chinese and English in one
stream, with no Shift or CapsLock toggling. This file is the single source of
project rules; `CLAUDE.md` only imports it.

The project follows spec-driven development (SDD): the specification is the source
of truth, the plan and tasks derive from it, and code derives from the tasks.
The non-negotiable principles are in `.specify/memory/constitution.md`.

## Where things go

- `.specify/memory/constitution.md`: project principles. Read first.
- `specs/NNN-topic/`: one folder per feature. Each holds `spec.md` (what and why),
  `plan.md` (how), `tasks.md` (work items and status), plus supporting files
  (`research.md`, `acceptance.md`, platform guides).
- `docs/`: product definition, architecture, roadmap, current status, verification
  log, typing habits, and `adr/` (decision records).
- `crates/keyflip-core`: the engine. Key event in, full snapshot out. No system APIs.
- `crates/keyflip-cli`: simulator (`play`) and evaluator (`eval`).
- `crates/keyflip-proto`: wire protocol. `crates/keyflip-server` and
  `crates/keyflip-tsf`: Windows engine service and thin text-service shell.
- `crates/keyflip-ffi` and `macos/`: C interface and thin Swift shell for macOS.
- `installer/`: Windows install and uninstall scripts.
- `tools/`: lexicon and English word list build scripts, Windows cross-build script.
- `data/`: compiled lexicon and English word list, produced by
  `tools/build_lexicon.py` and `tools/build_english.py`. Sources and licenses are in
  `NOTICE` and `THIRD_PARTY_LICENSES.md`.

## Reading order for a new session

`AGENTS.md`, `.specify/memory/constitution.md`, `README.md`, `docs/STATUS.md`
(where we are, next action), `docs/TYPING_HABITS.md`, `docs/ARCHITECTURE.md` and
`docs/adr/`, the active `specs/NNN-topic/{spec,plan,tasks}.md`,
`docs/VERIFICATION.md`, then the real Git state.

## Development rules

- Spec first. A behavior change starts as a requirement in `spec.md`, then an
  acceptance case (`specs/001-mixed-input/acceptance.md` and
  `crates/keyflip-core/tests/cases.txt`), then code. Never delete or loosen an
  acceptance case to make an implementation pass.
- Judgment logic lives only in `keyflip-core`. Shells never decide the language.
- A shell's two questions about a key (pre-check and real handling) must get the
  same answer.
- If the engine is unavailable, the shell passes every key through.
- Password fields: send no keys, record nothing.
- No single-tap Shift/CapsLock (or any modifier-only) gestures. This is the core
  product promise (ADR-0002).
- Run the commands in `docs/VERIFICATION.md` before and after a change and log
  the results there. Update the task status in `tasks.md` and `docs/STATUS.md`.
- All documentation and code comments are in English.
- Stage explicit paths when committing; do not use `git add -A`.
- Make technical choices yourself and record them (ADR or plan); do not push them
  back to the maintainer. The maintainer's job is key-press testing and reporting.
- Never commit personal data, employer data, internal names, paths, or URLs.
