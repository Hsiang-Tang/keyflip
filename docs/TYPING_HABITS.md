# Maintainer typing habits and preferences

First-hand product requirements from the maintainer, who is also the primary user.
When a feature trade-off conflicts with these habits, this file wins. They are not
options to negotiate, and technical decisions are not to be pushed back to the
maintainer.

## The problem

- Typing Chinese with Microsoft New Zhuyin needs a Shift tap to switch to English;
  the Mac keyboard mode is worse and needs CapsLock.
- CapsLock short versus long presses change case, language mode, or full/half
  width unpredictably.
- Example: in a Chinese sentence ending with a name, type the capital `H` directly
  with no switching.

## Scope (only this one problem)

- Only Chinese/English switching. Everything else follows New Zhuyin habits.
- The maintainer never taps Shift or CapsLock to switch, so those gestures must not
  be supported or relied upon.
- Holding Shift to type a capital is normal typing, not a switching gesture.
- Accepting a prediction with Tab (like VS Code) is a bonus, planned for version 2.

## Typing habits

- Always types tones: Space is first tone, `6` second, `3` third, `4` fourth, `7`
  neutral. The whole judgment method depends on this.
- Standard (Dachen) Zhuyin layout.
- Currently uses Microsoft New Zhuyin on Windows and the built-in Zhuyin on macOS.
- The keyboard can switch between Mac and Windows modes, so key identity uses
  physical position, not the active layout.

## Selection and confirmation

- Selection flow: Left (select previous character), Down (show candidates), a
  digit, then Enter to confirm.
- Enter is pressed twice: the first confirms the composition, the second is the
  application's own newline or send.
- Shift+Enter is common in Electron apps (terminals, chat, editors) for newlines,
  so Shift+Enter and Ctrl/Alt/Meta combos confirm the composition first and only
  reach the application the second time.

## Punctuation

- Half-width by default; full-width is rare.
- Shift+Space toggles full-width mode explicitly.
- Punctuation use is mixed, so there is no auto-decision based on the character
  left of the cursor (zhijeida's approach); the mode switch stays explicit.

## Other decisions

- CJK spacing between Chinese and English: off by default, can be enabled.
- Lexicon: fully replaces New Zhuyin, using McBopomofo. No custom phrase import in
  the first version.
- Platforms: Windows and macOS both. The maintainer's work Windows machine has no
  administrator rights, so TSF registration is impossible there; macOS goes first
  and is tested at home.
- Recovery from a wrong judgment: Enter commits raw letters, Tab flips the last
  judgment.

## Working style

- Prefer bullet lists over wide tables; write numbers with units.
- Explain technical terms in plain words instead of adding only an English label.
- Decide technical choices and write them down.
- Everything needed to continue must be in the repository docs, so a remote session
  can pick up from GitHub alone.
