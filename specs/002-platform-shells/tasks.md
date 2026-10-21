# Tasks: platform shells

- T-PS-1 Protocol library and tests. Requirement PS-1. Verification:
  `cargo test -p keyflip-proto`. Status: done.
- T-PS-2 Windows engine service. Requirement PS-5. Verification: on real Windows, ping,
  hello, Hsiang, concurrent clients, single instance, `--stop` (see
  `docs/VERIFICATION.md`). Status: done.
- T-PS-3 Windows TSF DLL loading. Requirements PS-4, PS-11. Verification:
  `keyflip-tsf-smoke.exe` loads on real Windows, obtains the factory, rejects a foreign
  CLSID, unloads. Status: done.
- T-PS-4 Windows TSF registration and real typing. Requirements PS-2 to PS-9. Verification:
  manual checklist in `windows.md`. Status: dropped (ADR-0008, macOS only).
- T-PS-5 C interface (keyflip-ffi). Requirement PS-1. Verification:
  `cargo test -p keyflip-ffi` (5 tests, including the user files). Status: done.
- T-PS-6 macOS Swift shell builds. Verification: `macos/build.sh` succeeds. Status: done
  (2026-10-21, Apple Silicon).
- T-PS-7 macOS real typing. Requirements PS-2 to PS-9. Verification: manual checklist in
  `macos.md`. Status: in progress; installed, waiting for hand testing.
- T-PS-8 macOS icon and multiple modes. Status: icon done (2026-10-21, vector template
  PDF, rendered and inspected; menu bar appearance waits for the hand test); multiple
  modes not started.
- T-PS-9 Settings UI (Chinese/English spacing, full-width default). Status: dropped
  (2026-10-21, see `docs/ROADMAP.md`).
- T-PS-10 Signed and notarized macOS release. Verification: `macos/release.sh` produces
  a stapled zip that `spctl` accepts. Status: done (2026-10-21); first notarized build
  `keyflip-0.1.0-macos-arm64.zip`.
