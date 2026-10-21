# keyflip on macOS: build, install, and manual acceptance

The macOS version has two parts:

- Rust engine (`crates/keyflip-ffi`): the same judgment core as Windows, built as a static
  library that offers "one key in, full state out" through a C interface
  (`crates/keyflip-ffi/include/keyflip.h`).
- Swift shell (`macos/`): a thin shell over InputMethodKit (IMK, the system interface that
  lets an input method receive keys and show composition text). It makes no language
  judgment; it forwards keys and draws what the engine returns.

macOS input methods install in your own folder (`~/Library/Input Methods`) and need **no
administrator rights**.

## Verification status

- Rust engine and C interface: tests pass, and static libraries build for
  `aarch64-apple-darwin` and `x86_64-apple-darwin`.
- Swift shell: builds, signs ad hoc, and installs on Apple Silicon (2026-10-21).
- Real typing behavior: not yet verified; the checklist is at the end of this file.

## Requirements

- Xcode command line tools: `xcode-select --install` (provides `swiftc` and `codesign`).
- Rust, from rustup or Homebrew.

## Get the code

```
git clone https://github.com/Hsiang-Tang/keyflip.git
cd keyflip
```

## Build and install

```
bash macos/build.sh
```

The script builds the Rust engine, builds the Swift shell, signs ad hoc
(`codesign -s -`), and copies the app to `~/Library/Input Methods/Keyflip.app`. To build
without installing, use `bash macos/build.sh --no-install`; output goes to `macos/build/`.

After the first install:

1. Log out and back in once.
2. Open System Settings, Keyboard, Text Input, Input Sources, Edit, then `+`.
3. Choose Chinese (Traditional) and add "keyflip Zhuyin".
4. Switch to it from the menu bar input menu.

After a rebuild, the new version is used the next time you switch to it; to replace it
immediately run `killall Keyflip`.

## Uninstall

```
bash macos/uninstall.sh
```

Then remove "keyflip Zhuyin" in System Settings and log out and back in.

## Troubleshooting

- Is the input method process running: `pgrep -l Keyflip`.
- System log: `log stream --predicate 'process == "Keyflip"'`.
- "keyflip Zhuyin" is missing from the menu: confirm you logged out and back in, and that
  `~/Library/Input Methods/Keyflip.app/Contents/Info.plist` exists.
- No response while typing: run `killall Keyflip`, then switch input methods again.

## Shell behavior rules

- Only keyDown is received. Modifier-only events such as a Shift or CapsLock tap never
  arrive, so they cannot do anything.
- Key identity uses the physical key position (US-layout characters), not the active keyboard
  layout, so it matches Windows.
- CapsLock state does not affect `shift`; the light is sent as `caps_lock` and the engine
  makes letters uppercase English while it is on (ADR-0006). Turn off the system setting
  "Use Caps Lock to switch to and from ABC", or macOS takes the key first.
- While composing, a key the engine does not consume does not clear the composition (the
  engine swallows useless keys such as Up).
- When the input method is switched away or the application asks to commit, the shell sends
  `flush` to the engine and the composition text is committed.
- Password fields: in secure input mode macOS does not hand keys to third-party input
  methods, so the shell does nothing special.
- The candidate window is a small self-drawn floating window that does not steal focus. It
  sits below the cursor line, or above when there is no room.

## Manual acceptance checklist

Standard: each item must match `specs/001-mixed-input/acceptance.md`.

- [ ] Notes: type `su3cl3` then Enter; the Chinese for "hello" appears.
- [ ] `Hsiang` (hold Shift for H) then Enter; `Hsiang` appears.
- [ ] `github`, `docker`, `npm install` appear as English with no switching.
- [ ] `Hsiang`, Space, then `su3cl3` composes "Hsiang" followed by the Chinese for "hello".
- [ ] `j9s` (ㄨㄞㄋ) then Tab then Enter gives ㄨㄞㄋ.
- [ ] `docu` shows a small panel `document  Tab` below the caret; Tab completes
      `document`; Enter without Tab gives `docu`.
- [ ] `1` then Enter gives ㄅ; `ai` then Tab then Enter gives `ai`; `ai` then Space
      (Chinese appears) then Tab gives `ai `.
- [ ] Enter while composing only confirms the text, with no newline or send; a second Enter
      breaks the line or sends the message.
- [ ] Shift+Enter while composing behaves the same: the first confirms, the second breaks
      the line.
- [ ] `su3cl3`, Left, Down: the candidate window appears; a digit picks; Enter confirms.
- [ ] With the candidate window open, Right expands it into a grid of every candidate;
      arrows move, a digit picks from the highlighted row, Esc returns to the list.
- [ ] The candidate window appears next to the cursor without covering the text being
      typed; at the bottom of the screen it appears above.
- [ ] A Shift tap changes nothing. A CapsLock press only turns the light on or off and
      switches nothing.
- [ ] With the CapsLock light on, `docker` gives `DOCKER`; Shift+`a` gives `a`; digits
      stay digits. With the light off, Zhuyin works again.
- [ ] Shift+Space toggles full-width: punctuation becomes full-width; again returns to
      half-width.
- [ ] Test once each in Safari text fields, Chrome, VS Code (editor and terminal), a chat
      app, Notes, and Terminal.
- [ ] Switching to another input method or window commits the composing text with no loss.
- [ ] In a password field the input method does not intervene.
- [ ] Up while composing: the composition does not disappear.
- [ ] A shortcut such as Cmd+C while composing: the first press confirms the text, the
      second runs the shortcut.

## Known limits

- There is only one input mode. The menu icon is a vector template (`MenuIcon.pdf`,
  regenerate with `macos/make-icon.swift`); macOS caches input source icons, so a changed
  icon may only appear after logging out and back in.
- The candidate window is a simple floating window with no page indicator.
- Chinese right after an English word works on tone keys; a first-tone syllable there
  still needs a Space first.
- Only the standard Zhuyin layout is supported.
