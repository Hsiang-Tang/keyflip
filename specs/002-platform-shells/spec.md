# Specification: platform shells (002-platform-shells)

Scope: two thin shells, Windows (TSF) and macOS (InputMethodKit), and the protocol between
engine and shell.

## Requirements

- PS-1 A shell only converts system keys into `KeyEvent`, draws the `Snapshot`, and sends
  committed text. It keeps no judgment state and never judges language.
- PS-2 Key identity uses physical position (US-layout characters), not the active keyboard
  layout, identical on Windows and macOS.
- PS-12 Every key carries the CapsLock light as `caps_lock`; the CapsLock key itself is
  never sent (ADR-0006).
- PS-3 Only keyDown is handled. Single-tap Shift, CapsLock, or any modifier-only event must
  have no effect.
- PS-4 The two questions about whether to consume a key (Windows `OnTestKeyDown` and
  `OnKeyDown`) must get the same answer.
- PS-5 If the engine is unavailable or times out, the shell passes every key through.
- PS-6 When the input method is switched away, loses focus, or the application asks to
  commit, the shell sends `flush` to the engine and the composition text is committed, not
  lost.
- PS-7 Password fields: Windows sends no keys and records nothing. On macOS, whenever
  secure event input is on (password fields, the keychain prompt, Terminal's Secure
  Keyboard Entry), the shell passes every key through untouched, discards any
  composition without committing it, and never reaches the engine, so nothing is
  learned. The system does not do this by itself: on 2026-10-21 the keychain password
  prompt delivered keys to keyflip, digits after a Shift word stayed in the
  composition, and the field refused them.
- PS-8 The candidate window is a self-drawn floating window that does not steal focus. It sits
  below the cursor line, or above when there is no room; there is a fallback when the cursor
  coordinates are unavailable.
- PS-9 Collapse the selection after committing (Windows Notepad would overwrite committed
  text).
- PS-10 Install and uninstall ask for no unnecessary privilege: macOS needs no administrator
  rights; registering a TSF on Windows does (ADR-0005).
- PS-11 The Windows shell does not import `comctl32` and passes NULL for `hklsubstitute` at
  registration (pitfalls found in ruhi).

## Protocol (keyflip-proto)

- Transport: 4-byte little-endian length plus JSON. Windows uses the named pipe
  `\\.\pipe\keyflip-v1-<user SID>`; macOS calls the in-process library through the C
  interface with the same JSON.
- Requests: `key`, `test`, `reset`, `flush`, `ping`. Responses: `state`, `test`, `pong`,
  `error`.
- A response is always the complete snapshot; see `docs/ARCHITECTURE.md`.

## Acceptance

- Windows: manual checklist in `windows.md`.
- macOS: manual checklist in `macos.md`.
- Both must match the behavior in `specs/001-mixed-input/acceptance.md`.
