# keyflip on Windows: build, install, and manual acceptance

On Windows keyflip consists of three files:

- `keyflip_tsf.dll`: the input method shell. The Windows Text Services Framework (TSF, the
  system interface between applications and input methods) loads it into every program with
  a text box. It only forwards keys and draws results; it makes no language judgment.
- `keyflip-server.exe`: the engine server, one per user. All judgment happens here.
- `keyflip-pipe-client.exe`: a diagnostic tool. It sends one request to the server and
  prints the response, to confirm the server is alive.

Shell and server talk over a named pipe (a local inter-process channel on Windows) named
`\\.\pipe\keyflip-v1-<user SID>`. Each user has their own pipe, so no one reaches another
user's engine.

## Shell behavior rules

- The pre-check (`OnTestKeyDown`) sends a `Test` request and returns `handled` as is; real
  handling (`OnKeyDown`) sends a `Key` request and acts on the response. Both give the same
  answer.
- If the pipe cannot connect, does not answer within 1 second, or errors, the shell passes
  every key through (returns FALSE). It reconnects automatically and does not retry for 2
  seconds after a failure, so it never hangs the application.
- If the pipe does not exist, the shell tries to start `keyflip-server.exe` next to the DLL.
  Sandboxed programs (such as the Windows search box) cannot start processes, so the
  installer also registers autostart at login.
- Password and read-only fields get no keys: when the input scope is password or PIN, or the
  content is read-only or transient, the shell passes the key through and sends nothing to
  the server.
- A single Shift or CapsLock tap does nothing. Shift state comes from the Shift key only;
  CapsLock does not affect `shift`.
- When focus moves to another document the shell sends `Reset` to the server and ends the
  composition without committing any text.
- After committing text the cursor collapses to the end of the text; otherwise Notepad lets
  the next character overwrite the previous one.

## Identifiers

- CLSID (COM class): `{EE404856-D919-44A1-9E03-44A3A9687088}`
- Language profile GUID: `{7A5720D5-9112-4CEB-AAD3-D59AB153FBE8}`
- Display attribute, composing (underline): `{A486B672-F12C-4286-BB0D-92FA6A8E7CDE}`
- Display attribute, selected (highlight): `{9A5BFB0C-829B-442C-9D86-32383E39963E}`
- Language: zh-TW (0x0404)
- Display name: keyflip Zhuyin

These values are defined only in `crates/keyflip-tsf/src/consts.rs`.

## Build

Cross-compile on Linux or WSL (no Windows compiler needed):

```
rustup target add x86_64-pc-windows-gnu
sudo apt install mingw-w64
tools/build-windows.sh
```

The script keeps output outside the repository (default `~/keyflip-target-win`), copies the
three files into `dist/`, and prints each file's SHA-256. Set `CARGO_TARGET_DIR` to change
the output location.

Platform-independent unit tests (key mapping, composition and candidate display plan) run on
Linux:

```
cargo test -p keyflip-tsf --lib
```

Known limit: only the 64-bit build exists, so 32-bit programs will not show this input
method.

## Install

1. Bring `dist/` to the Windows machine.
2. Open PowerShell as administrator.
3. Preview first to see what it will do:

   ```
   .\installer\install.ps1 -DryRun
   ```

4. Install for real:

   ```
   .\installer\install.ps1
   ```

   Use `-BinDir <folder>` to choose the source; the default is `dist\` under the repository.

The installer does these steps in order:

- Copies the three files to `%ProgramFiles%\keyflip\`.
- Uses `icacls` to give "ALL APPLICATION PACKAGES" read and execute on that folder so
  sandboxed programs can load the DLL.
- Runs `regsvr32 /s keyflip_tsf.dll` to register the COM class and the input method profile.
- Registers server autostart under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`. It
  does not start the server during install (the installer window runs as administrator, and
  the engine should not run elevated); the server starts at the next login, or when the shell
  starts it with normal privileges the first time you type with keyflip. This is a
  current-user setting, so open the administrator window with the same account you normally
  log in with.

Finally open Settings, Time & Language, Language & Region, Chinese (Traditional, Taiwan),
Language options, Keyboards, add "keyflip Zhuyin", and switch to it with Win+Space.

## Uninstall

Open PowerShell as administrator:

```
.\installer\uninstall.ps1 -DryRun
.\installer\uninstall.ps1
```

It stops the server with `keyflip-server.exe --stop`, runs `regsvr32 /u /s`, deletes the
autostart value, and deletes the install folder. If a program that was open while the input
method was active still holds the DLL, deleting the folder fails with a message; log out or
restart and delete again.

## Manual acceptance checklist

TSF behavior can only be confirmed by actually typing. Tick each item after typing it; on
failure note the application, the keys you pressed, and what you saw.

Preparation:

- [ ] Confirm the server is alive with `keyflip-pipe-client.exe`; it should print
      `{"op":"pong"}`.
- [ ] Switch the language bar to "keyflip Zhuyin".

### Basic scenarios (in every program)

In an ordinary text field of each program below, type:

1. Chinese: `su3cl3` (the Chinese for "hello") then Enter. The composition shows an
   underline; after Enter the text is committed with no line break.
2. English: `github` then Space. It is judged English and not turned into Zhuyin.
3. Capital start: type Chinese `su3cl3`, press Space to finish, then hold Shift for `H` and
   type `siang`. You get `Hsiang`, not split into Zhuyin.
4. Two-step Enter: Enter while composing only confirms the text; a second Enter breaks the
   line or sends.
5. Two-step Shift+Enter: Shift+Enter while composing confirms the text and swallows the key;
   a second press takes effect (for example a chat app's line break).
6. Tab flip: Tab while composing cycles between raw, English, and Zhuyin.
7. Full-width toggle: Shift+Space switches to full-width; `,` should give a full-width
   punctuation mark; press again to return to half-width.
8. Shift tap: pressing and releasing Shift alone must do nothing; a CapsLock tap must not
   affect judgment either.
9. Candidate window position: after one character press Down; the window appears just
   below the composition, or above near the bottom of the screen; 1 to 9 pick a candidate;
   it disappears when you switch windows.

### Programs

- [ ] Notepad: all basic scenarios pass, and typing after a commit does not overwrite the
      previous character.
- [ ] VS Code editor: all basic scenarios pass.
- [ ] VS Code terminal: type a command such as `git status`; English is not split into
      Zhuyin; with an empty composition Tab is still terminal completion.
- [ ] Chrome or Edge text fields (address bar and web inputs): all basic scenarios pass.
- [ ] A chat app such as Discord: all basic scenarios pass; the two-step Shift+Enter must
      match the description above.
- [ ] Windows search box (sandboxed): typing works. If it fails or the server was not
      started automatically, check the permissions on `%ProgramFiles%\keyflip` and whether
      `keyflip-server.exe` is already running after login.
- [ ] Password fields (for example a browser password box, or a UWP password box other than
      the Windows login): the input method must intercept no keys; whatever you type appears
      as is, with no composition and no candidate window.

### Failure and recovery

- [ ] End `keyflip-server.exe` in Task Manager: the typing program must not hang; keys pass
      through (English letters appear as typed); within a few seconds, or after a few more
      key presses, the server restarts and the input method recovers.
- [ ] Click elsewhere or switch windows halfway through composing: the composition
      disappears with no leftover text.

### Recording results

Record acceptance results and failure details in `docs/VERIFICATION.md`.

## Troubleshooting

- "keyflip Zhuyin" is not in the input method list: confirm `install.ps1` ran as
  administrator with no errors, then add the keyboard in the language options.
- Pipe problems: run `keyflip-pipe-client.exe`; if it cannot connect, the server is not
  running. Run `%ProgramFiles%\keyflip\keyflip-server.exe` by hand to see the error.
- To start over completely: run `uninstall.ps1`, log out and back in, then run `install.ps1`.
