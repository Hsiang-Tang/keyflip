# keyflip product definition

## Problem

When typing Chinese with Zhuyin, every English word needs a mode switch first
(a Shift tap, and on macOS a CapsLock short or long press), then another switch back.
Switching is frequent and error-prone, and often produces wrong case or
full-width/half-width mixups.

## Outcome

Typing needs no switching gesture:

- In `My name is Hsiang` (inside a Chinese sentence), hold Shift for `H`; the
  lowercase letters that follow join the same English word.
- All-lowercase words such as `github` or `docker` are judged English at the point
  where the keys can no longer form a valid Zhuyin syllable.
- A wrong judgment is fixed with one key: Enter commits the raw letters; Tab cycles
  between raw, English, and Zhuyin.

Everything else keeps Microsoft New Zhuyin habits:

- In the composition area, Left moves back; Down on the character left of the
  cursor opens candidates; a digit picks one.
- Enter while composing only confirms the text; a second Enter sends or breaks the
  line.
- Punctuation is half-width by default; Shift+Space toggles full-width mode.
- Tones: Space is first tone, `6` second, `3` third, `4` fourth, `7` neutral.

## Non-goals (first version)

- Single-tap Shift, CapsLock, or other modifier gestures. This is the core promise.
- Fast-typing error correction (early tone, swapped medial/final). Similar projects
  repeatedly regress here, so the first version does not touch it.
- Habit learning, AI candidate ranking, Tab prediction bar (prediction is version 2).
- Importing custom phrases from New Zhuyin.
- Keyboard layouts other than standard (Dachen) Zhuyin.

## Platforms

- Windows (replacement for Microsoft New Zhuyin): first target.
- macOS (replacement for the built-in Zhuyin): second stage, same core engine.

## Prior art

Surveyed: UnifyIME, TronIn, zhijeida, BearSpark, shanjie, vChewing, ruhi, 0C,
KnowType, ZingIME. Full notes: `specs/001-mixed-input/research.md`.

Worth borrowing:

- Treat every key as both Zhuyin and English; judge by whether the Zhuyin structure
  is legal (BearSpark, zhijeida, vChewing).
- Hard structural rules, for example the initials ㄒ ㄐ ㄑ can only precede ㄧ or ㄩ,
  and ㄦ never follows an initial (TronIn).
- Shift plus letter is a definite English signal (vChewing, shanjie, TronIn, BearSpark).
- "Can form a character" is not enough; it must form a common character (TronIn's
  `do` versus 剋).
- The 0C architecture: thin shell plus separate engine, key event in, full snapshot
  out, shell keeps no state.
- Protection rules for English identifiers (`camelCase`, `z-index`, `.json`, URLs).
- Measure error rates with a simulator instead of guessing (shanjie).
- Windows pitfalls: pass NULL for `hklsubstitute`, never statically import
  `comctl32`, keep the pre-check and real handling answers identical (ruhi, 0C).

Why build another one:

- No project combines Windows, standard Zhuyin, no modifier gestures, and New Zhuyin
  habits.
- Most similar projects are macOS-only, or ship mixed typing off by default and need
  gestures for recovery.
- TronIn is closed source; the others make trade-offs that differ from these habits.
