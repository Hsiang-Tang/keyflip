# Plan: platform shells

## Windows

- Shell `crates/keyflip-tsf`: COM text service DLL (Rust `windows` crate pinned to
  `=0.58.0`), no lexicon.
- Engine service `crates/keyflip-server`: separate process that holds the engine and
  lexicon and serves a named pipe. The DLL starts it lazily when it cannot connect; it also
  starts at the next login after installation.
- Cross-compiled in WSL with `x86_64-pc-windows-gnu` (`tools/build-windows.sh`, mingw-w64).
  The DLL and server are built in two separate cargo invocations so the protocol library's
  `engine` feature does not bloat the DLL (7.3 MB down to 1.36 MB).
- Install: `installer/install.ps1` (administrator PowerShell, registered with regsvr32).
- Known pitfalls and fixes: `research.md` sections 9 and 10 and `windows.md`.

## macOS

- The Rust engine `crates/keyflip-ffi` is built as a static library; the C interface is in
  `crates/keyflip-ffi/include/keyflip.h`: `keyflip_session_new`, `keyflip_session_free`,
  `keyflip_handle(session, request JSON)` returning a JSON string, and `keyflip_string_free`.
- Swift shell `macos/`: an `IMKInputController` subclass that requests keyDown only and
  overrides `handle(_:client:)`. It uses `insertText` and `setMarkedText` to present results;
  the candidate window is its own non-activating `NSPanel`.
- Build: `macos/build.sh` (cargo plus swiftc, ad hoc signature, installs into
  `~/Library/Input Methods`).
- IMK pitfalls and handling:
  - After overriding `handle`, IMK no longer calls `inputText`.
  - Returning false for an unhandled key while composing may make the host clear the
    composition, so the engine swallows useless keys (Up) and the shell returns true for
    unrecognized keys while composing.
  - The IMK process never receives modifier-only events, which matches the design of using
    no modifier gestures.
  - An empty composition is cleared only if there was one before, to avoid flicker in some
    hosts.

## Decisions

See `docs/adr/`: engine/shell split (0001), no modifier gestures (0002), optional engine
feature in the protocol (0003), JSON over a C interface on macOS (0004), Windows needs
administrator rights (0005).
