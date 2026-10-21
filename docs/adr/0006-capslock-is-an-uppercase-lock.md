# ADR-0006 CapsLock is an uppercase lock, never a language switch

- Status: accepted (2026-10-21). Amends one line of ADR-0002.

## Context

ADR-0002 bans modifier-only gestures and says CapsLock state does not affect judgment.
In the first macOS hand test the maintainer asked for the habit every other input
method keeps: with the CapsLock light on, letters come out uppercase, so `DOCKER` does
not need Shift held for six keys. The problems ADR-0002 lists come from using CapsLock
to switch languages (light out of sync with the mode, taps lost while the light is on),
not from CapsLock's ordinary meaning.

## Decision

- The shells report the CapsLock light as `caps_lock` with every key. They never send
  the CapsLock key itself, so a tap still does nothing on its own.
- In `keyflip-core`, a letter typed with the light on is an uppercase English letter,
  as if Shift were held; Shift with the light on gives a lowercase English letter.
  Digits, punctuation, tones, and every other key are unaffected.
- With the light on, Zhuyin cannot be typed, as in Microsoft New Zhuyin.
- CapsLock never switches language and never toggles anything inside keyflip.

## Consequences

- ADR-0002's line "CapsLock state does not affect judgment" is replaced by this record;
  the rest of ADR-0002 stands.
- On macOS, the system setting "Use Caps Lock to switch to and from ABC" takes the key
  before keyflip sees it; with that setting on, CapsLock switches input sources instead.
- Windows reads the light from the keyboard state's toggle bit; unverified until a
  machine with administrator rights exists.
