# keyflip acceptance cases

Machine-readable cases are in `crates/keyflip-core/tests/cases.txt` (key sequence plus
expected committed text). Behavior details (which keys are swallowed) are in
`crates/keyflip-core/tests/behavior.rs`. This file explains the categories and rules.
Never delete or loosen a case to make an implementation pass.

## Key notation

- `a-z 0-9 , . ; / - '` are physical keys, `A-Z` is Shift plus a letter, `_` is Space.
- `~ ! @ # $ % ^ & * ( ) + { } | : " ?` are Shift plus the key that types them; `<S-k>`
  is Shift plus the plain key `k` (`<S-->` types `_`, `<S-,>` and `<S-.>` type `<` `>`).
- `<E>` Enter, `<S-E>` Shift+Enter, `<T>` Tab, `<L>` `<R>` `<D>` `<U>` arrow keys,
  `<B>` Backspace, `<X>` Esc, `<S-_>` Shift+Space.

## Categories

Positive cases (English where it should be English, Chinese where it should stay Chinese)

- Chinese stays Chinese: `su3cl3` is the Chinese for "hello".
- English starting with Shift-typed capitals: `Hsiang`, `API`, `iPhone`.
- All-lowercase English judged by Zhuyin structure: `github`, `the`, `z-index`, `docker`,
  `npm install`.
- Digits: `192.168.1.1`, `2026`, `user2024` stay as typed; `3` followed by Chinese splits
  into "3" plus the Chinese word; `2026` followed by a Chinese unit splits.
- Mixed text: English then Chinese, and Chinese then English.

Identifier protection (MI-20, MI-21)

- After an English word, `_` and `@` keep the word going: `my_ai` stays English, and so
  does `a@su3.com`.
- In full-width mode, punctuation inside URLs, paths, and identifiers stays half-width:
  `https://ai.com`, `src/ai/main.rs`, `~/ai`, `FOO_BAR`.
- Sentence punctuation still follows the setting: `Hsiang，`, `Note：`, `（API）`, and
  `API／` before Chinese stay full-width.

Negative cases (must not be misjudged)

- `keyflip-cli eval` types every Chinese phrase in the lexicon; none may be converted to
  English letters. Currently 0 misjudgments across 4,940 phrases.

Boundary cases

- With an empty composition, Enter, Tab, Space, arrows, Backspace, and Esc all pass through
  to the application.
- English letters are always consumed (the shell never types them twice).
- Cursor position is counted in UTF-16 units.

Recovery cases

- Enter mid-typing commits the raw letters: `ai<E>` is `ai`.
- Tab flips the judgment back and forth between Chinese and English.
- Shift+Enter and Ctrl/Alt/Win combos: the first press only confirms the text while
  composing; the second passes through to the application.

## Known misjudgments that need recovery

English misjudgments measured by the evaluator concentrate on common one- and two-letter
words (`i`, `up`, `ai`, `jp`) because they are also legal Zhuyin syllables. They default to
Chinese and are recovered with Enter or Tab. Only 0.2% of the top 10,000 English words are
misjudged.

## Not yet accepted (needs a human on real machines)

See the manual checklists in `specs/002-platform-shells/windows.md` and `macos.md`.
