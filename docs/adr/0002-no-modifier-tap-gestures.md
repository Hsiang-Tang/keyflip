# ADR-0002 No modifier-tap gestures

- Status: accepted (2026-10-21)

## Context

The original pain point is Shift and CapsLock taps. Studying vChewing, UnifyIME, and
0C showed these gestures have a long list of hard-to-fix problems: CapsLock switching
delay, Electron splitting one release into two events, modifier events lost after an
app switch, Shift taps undetectable while the CapsLock light is on, and no consensus on
the tap-versus-hold threshold (0.2 s, 0.3 s, or none).

## Decision

- Language switching is decided entirely from the key sequence, never from modifier-only
  events.
- On macOS the shell requests keyDown only, so the system never sends flagsChanged.
- Shift matters only combined with a letter (an English word starting in capitals),
  which is normal typing.
- CapsLock state does not affect judgment.

## Consequences

- Without a "force English" gesture, recovery relies on Enter (commit raw letters) and
  Tab (flip); misjudgments concentrate on one- and two-letter ambiguous words.
- This promise is part of the constitution and must not be broken.
