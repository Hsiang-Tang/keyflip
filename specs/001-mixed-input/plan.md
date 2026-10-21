# Plan and design: mixed input (001-mixed-input)

Overall architecture: `docs/ARCHITECTURE.md`. Decision rationale: `docs/adr/`.

## Architecture

```
 key ──► shell (Windows TSF / macOS IMK) ──► engine keyflip-core
              ▲                                   │
              └────────── full snapshot ◄─────────┘
```

- The engine is a pure library: `Engine::handle_key(KeyEvent) -> Snapshot`. No files,
  windows, or system APIs.
- A shell does two things: convert system keys into `KeyEvent`, and draw the `Snapshot` and
  send committed text. It keeps no judgment state. If the engine is broken or unavailable,
  the shell passes every key through.

### Snapshot

After each key the engine returns the complete state:

- `handled`: whether the input method consumed the key.
- `commit`: text to send to the application (possibly empty).
- `preedit`: list of composition segments, each with text and kind (Chinese, English,
  unfinished Zhuyin, selected).
- `cursor`: cursor position in the composition, in UTF-16 units (convenient for Windows).
- `candidates`: candidate window content (list, selected item, page); empty when closed.
- `full_width`: whether full-width mode is on.

## Key model

`KeyEvent` carries the physical key (a character of the standard Zhuyin layout) and whether
Shift, Ctrl, Alt, or Win (Meta) is down. Language judgment uses only "Shift plus letter";
no modifier-only event is used.

Standard Zhuyin layout:

```
1 ㄅ  2 ㄉ  3 ˇ  4 ˋ  5 ㄓ  6 ˊ  7 ˙  8 ㄚ  9 ㄞ  0 ㄢ  - ㄦ
q ㄆ  w ㄊ  e ㄍ  r ㄐ  t ㄔ  y ㄗ  u ㄧ  i ㄛ  o ㄟ  p ㄣ
a ㄇ  s ㄋ  d ㄎ  f ㄑ  g ㄕ  h ㄘ  j ㄨ  k ㄜ  l ㄠ  ; ㄤ
z ㄈ  x ㄌ  c ㄏ  v ㄒ  b ㄖ  n ㄙ  m ㄩ  , ㄝ  . ㄡ  / ㄥ
Space = first tone (no mark)
```

## Composition

The composition is a sequence of units:

- Zhuyin unit: one finished syllable (with tone, including Space for first tone), later
  converted to Chinese characters or words.
- English unit: one character. Each English letter is its own unit, so Left can move letter
  by letter.

There is also a short run of unfinished keys (pending) waiting for a tone or the next key to
decide whether it is Zhuyin or English. Whole Chinese sentences are segmented with the
McBopomofo lexicon by best path (Viterbi); characters the user picked by hand are pinned and
not overwritten by recomputation.

## Language judgment

For each incoming key, apply in order:

1. **Shift plus letter** is a definite English start. Lowercase letters in pending merge
   into the same English word (`iPhone`). Following lowercase letters, digits, `.`, `-`, `_`,
   and `'` stay in this word until Space or another symbol.
2. **Tone keys (Space, 3, 4, 6, 7)**:
   - Pending plus the tone forms a legal syllable: Chinese syllable.
   - Otherwise pending becomes English and the tone key is an English character (Space ends
     the English word and appends a space).
   - Pending is empty: `3`, `4`, `6`, `7` are digits; Space opens the candidate window.
3. **Other keys**: if pending plus the key is no longer the start of any legal syllable,
   pending becomes English and the English word continues. The basis is Zhuyin structure:
   the order of initial, medial, final, and tone, plus hard rules (ㄒ ㄐ ㄑ can only precede
   ㄧ or ㄩ; ㄦ cannot follow an initial; an initial cannot follow an initial).
4. **Short words legal both ways** (such as `ai`, `do`, `up`):
   - Default Chinese.
   - If the letters are a common English word (top 3,000 of the English list) and the
     Zhuyin reading maps to an uncommon character (low frequency), judge English.
   - With no tone key, this judgment happens when Space is pressed.

A judgment can always be changed:

- Enter: commit what is shown; unfinished Zhuyin leaves as its symbols (MI-7). For the
  raw letters, Tab first. Letters in a path or URL token never become pending Zhuyin, so
  `~/ai` commits as typed.
- Tab (while composing): cycle between raw, English, and Zhuyin.

## Recovery and pass-through keys

- With text in the composition, Enter only confirms the text and does not reach the
  application; with an empty composition, Enter passes through.
- Shift+Enter and Ctrl/Alt/Win plus any key: with text in the composition, confirm the text
  and swallow the key (a second press takes effect); with no composition, pass through
  untouched.
- Tab: while composing, the cycle above; with an empty composition and no prediction
  suggestion, pass through to the application (terminal completion).
- Esc: close the candidate window; do not clear the composition.

## Punctuation and full-width

- Half-width by default. Shift+Space toggles full-width mode; all punctuation is full-width
  in that mode.
- `, . ; / -` are also Zhuyin keys (ㄝ ㄡ ㄤ ㄥ ㄦ), so they are held first: if the next key
  is a tone key and forms a legal syllable they are Zhuyin, otherwise they are sent as
  punctuation.
- `.`, `-`, `_`, `'` inside an English word are always half-width.
- Identifier protection (MI-20, MI-21). Shift+`-` and Shift+`2` after an English word or
  number are stored as English characters, so the word keeps going (`my_ai` does not fall
  back to Zhuyin at `ai`). Other punctuation is stored half-width with a flag recording
  that it was typed in full-width mode; the full-width form is chosen when the text is
  rendered, not when the key is typed. Rendering checks the token around the mark: inside
  a code-like token (word characters after it, and word characters or only path leads
  `~ / . \` before it) the mark stays half-width. Deciding at render time means a mark
  typed before the rest of the URL is corrected as soon as the next letter arrives, and
  Backspace restores the full-width form. This follows ruhi's rule that identifiers are
  never split and KnowType's token-shape checks, without KnowType's regex and whitelist
  approach, which misread `user.name` as a URL. The snapshot does not yet expose protected
  ranges; no shell needs them.

## Splitting Chinese off an English word (MI-23)

The split is decided only when a tone key arrives, because that is the first moment
the tail is known to be a whole syllable. Candidates are tails of one to four keys,
including keys still pending (`, ; /` are pending Zhuyin keys that would otherwise break
the word). Two passes pick the tail: first the shortest tail whose head is a listed
English word, so `years` + 一 keeps `years` rather than `year` + 泥; then the longest tail
whose head starts with a Shift capital or cannot be Zhuyin, so `Hsiang` + 好 keeps
`Hsiang`. A run that is itself a listed word never splits, which protects `form4` and
`item3`. Measured by `keyflip-cli eval`: glued sentences and English-plus-digit tokens.

## Known limits (first version)

- Chinese right after an English word splits on the tone key (MI-23). A first-tone
  syllable there still needs a Space first, because Space also ends the English word.
  When the word plus an inflection is also a word (`list` + ㄋ… versus `lists`), the
  longer word wins; Tab undoes a wrong split.
- Two-letter abbreviations (such as `jp`, `ai`) default to Chinese; recover with Enter or Tab.
- Only the standard Zhuyin layout.

## Shell rules

- The pre-check (`OnTestKeyDown`) and real handling (`OnKeyDown`) must give the same answer.
- Password fields: send no keys, record nothing.
- Pass NULL for `hklsubstitute` when registering the input method.
- Never statically import `comctl32` or `TaskDialogIndirect`.
- Collapse the selection after committing text, or Notepad lets the next character overwrite
  the previous one.
