# Specification: mixed Chinese/English input (001-mixed-input)

Status: core implemented and accepted (2026-10-21). Scope: behavior of
`crates/keyflip-core`, platform independent.

## Goal

Type with the standard Zhuyin layout and no switching gesture; the input method decides
whether each segment is Zhuyin or English. Background and habits: `docs/TYPING_HABITS.md`.
Survey of similar input methods: `research.md`.

## Non-goals

- No single-tap Shift, CapsLock, or any modifier-only gesture.
- No fast-typing error correction (early tone, swapped medial/final). TronIn needed five
  releases to fix its version.
- No support for users who do not type tones.
- First version: no layout other than Dachen, no custom phrase import, no learning.

## Requirements

Each requirement maps to an acceptance case or test; see `tasks.md`.

### Language judgment

- MI-1 Shift plus a letter starts an English segment; following letters, `.`, `-`, `_`,
  `'`, and digits keep joining the same English segment.
- MI-2 Keys typed without Shift are pending Zhuyin first. As soon as the pending sequence
  can no longer be the prefix of any legal Zhuyin syllable, the whole sequence becomes
  English. Legality comes from the lexicon's syllable-prefix set, which also encodes the
  hard structural rules of Zhuyin.
- MI-3 A tone key (Space, 3, 4, 6, 7) ends the syllable if the pending sequence forms a
  legal syllable; otherwise it is treated as English.
- MI-4 With nothing pending, the tone keys `3`, `4`, `6`, `7` are digits. A digit run may
  continue with digits and `.`.
- MI-5 Space after a pending sequence is a first-tone syllable by default. Exception: when
  the raw letters are a common English word (rank under 3,000) and the best Chinese
  candidate's frequency is under 50, treat it as English and append a space.
- MI-6 No automatic space between adjacent English and Chinese by default; a setting adds
  one.

### Recovery

- MI-7 Enter while composing commits the raw letters (not Chinese) and does nothing else.
- MI-8 Tab flips the most recent automatic judgment: Zhuyin to English or English to Zhuyin.
- MI-9 Backspace deletes one unit; after deleting an English letter the remaining key
  sequence is judged again.

### Candidate selection

- MI-10 The candidate list shows words first, then single characters, nine per page,
  selected by digit.
- MI-11 Left selects the previous character and Down opens candidates, as in New Zhuyin.
- MI-12 Esc closes the candidate window first; only without one does it clear the
  composition.

### Shell interaction

- MI-13 With an empty composition, Enter, Tab, Space, arrows, Backspace, and Esc all pass
  through to the application.
- MI-14 While composing, Shift+Enter and Ctrl/Alt/Meta combos: the first press only
  confirms the composition and swallows the key; the second (composition now empty) passes
  through.
- MI-15 Up arrow is swallowed while composing so the host does not assume the input method
  gave up and clear the composition.
- MI-16 English letters are always consumed (the shell never types them twice).
- MI-17 Cursor position is reported in UTF-16 units.
- MI-18 Punctuation is half-width by default; Shift+Space toggles full-width mode, and
  pressing again returns.

### Identifier protection

- MI-20 After an English word or a number, Shift+`-` (`_`) and Shift+`2` (`@`) join the
  word: they are English characters and the letters and digits after them stay English
  with no Zhuyin judgment (`my_ai`, `name@site`). With nothing English before them they
  are ordinary punctuation.
- MI-21 Punctuation inside a code-like token stays half-width even in full-width mode. A
  token is a run of English letters, digits, and punctuation with no space and no Chinese.
  A punctuation mark is inside the token when a letter or digit follows it in the token
  and either a letter or digit precedes it, or it and everything before it in the token
  are path leads `~`, `/`, `.`, `\` (`https://site.com`, `src/app/main.rs`, `~/src`).
  Punctuation that ends a token, or that opens one in front of a word (`(` in `(API)`),
  follows the full-width setting.

### Quality

- MI-19 Evaluation: no lexicon phrase is converted to English; the misjudgment rate on the
  top 10,000 English words is at most 1%. Current: 0 of 4,940 phrases leak; 21 of 10,000
  English words misjudged (0.2%).

## Open questions

- Two-letter abbreviations (`ai`, `jp`, `up`, `i`) are judged Chinese by default and
  recovered with Enter or Tab. Whether to improve this is left to real usage data.
- Identifier protection covers joiners and punctuation width (MI-20, MI-21). Splitting a
  token into protected ranges for the shell (KnowType's range list) waits for a shell
  that needs it.
