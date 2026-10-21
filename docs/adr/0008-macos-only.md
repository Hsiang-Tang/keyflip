# ADR-0008 macOS only; the Windows shell is frozen

- Status: accepted (2026-10-21).

## Context

keyflip exists because CapsLock and Shift switching on macOS input methods is
unreliable for mixed Chinese and English typing. The maintainer types only on macOS.
The Windows shell (`keyflip-tsf`, `keyflip-server`, `installer/`) loads on real
Windows but has never been registered, because no machine with administrator rights
is available (ADR-0005).

## Decision

- macOS is the only supported platform. New features target the engine and the macOS
  shell only.
- The Windows code stays in the repository, frozen: it keeps compiling and its tests
  keep running in `cargo test --workspace`, but it gets no new features, no hand
  testing, and no releases. Its documents are marked unmaintained.
- Roadmap item 7 (Windows install) is dropped.

## Consequences

- Protocol changes still need the Windows crates to compile, which costs little and
  keeps the option open.
- If the Windows shell is ever revived, it starts from its checklist in
  `specs/002-platform-shells/windows.md` and the features added since this record
  (completion display, user files) need porting.
