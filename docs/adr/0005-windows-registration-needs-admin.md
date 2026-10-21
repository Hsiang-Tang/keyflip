# ADR-0005 Windows registration needs administrator rights; no workaround

- Status: accepted (2026-10-21)

## Context

Registering a TSF text service writes to HKLM. Research (Microsoft documentation,
Praetorian research, Mozc) found no official per-user registration. The Windows machine
available at the time had a standard user account (Medium integrity, not in
Administrators), so `install.ps1` refuses to run.

## Decision

- Do not try to bypass the administrator requirement, and do not use a global keyboard
  hook (security software treats it as a keylogger).
- Hand-typing verification on Windows waits for a machine with administrator rights;
  macOS goes first.
- The parts verifiable without administrator rights (engine service, DLL loading) are
  verified on real Windows.

## Consequences

- See `docs/STATUS.md`: Windows typing behavior is unverified.
