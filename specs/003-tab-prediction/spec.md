# Specification: Tab prediction (003-tab-prediction)

Status: decisions made 2026-10-21 (both kinds; Chinese continuation accepted with Right
at the end of the composition). Not implemented; paused by the maintainer before
implementation. Next: write acceptance cases, then plan and tasks. Roadmap item 9.

## Goal

While typing, show a likely continuation as dimmed text after the cursor and accept it
with one key, the way an editor accepts an inline suggestion. The input method stays
offline and keeps no history (constitution principle 5).

## What Tab does today

- Idle (empty composition): Tab passes through to the application, so terminal and
  editor completion keep working (MI-13).
- Composing: Tab flips the most recent automatic judgment (MI-8), turns pending keys
  into letters, or turns a run of Zhuyin-key letters into symbols (MI-24).

Every finished Chinese syllable can be flipped back to letters with Tab, so a Chinese
suggestion accepted with Tab would hide that recovery on every syllable. English words
of three or more letters have no such conflict: they are judged English by structure,
and the short ambiguous words that Tab exists for (`ai`, `up`) are too short to
suggest anything.

## Kinds of prediction

- English word completion: after three or more letters of an English word, suggest the
  most common listed word (rank under 10,000) that extends the prefix by at least two
  letters. Source: `data/english.txt`, already bundled.
- Chinese continuation (associated phrases): after a Chinese character, suggest the
  most frequent lexicon phrase that starts with the last one or two characters (after
  我愛, suggest 你 from 我愛你 if such a phrase exists; after 電, suggest 腦). Source:
  `data/lexicon.tsv`, already bundled.

## Decisions

- **Scope**: English completion only first (recommended: no key conflict, smallest
  change, and real use shows whether Chinese continuation is wanted), or both kinds
  together.
- **Chinese accept key** (only if Chinese continuation is in scope): Right at the end
  of the composition (recommended: the key is free there, and shells such as fish use
  it for suggestions), or Tab, accepting that a syllable can then only be flipped back
  when no suggestion is showing.

## Draft requirements

- TP-1 A suggestion is shown only when the cursor is at the end of the composition and
  no candidate window is open. It is drawn as a separate preedit segment of kind
  `suggestion` after the cursor.
- TP-2 Tab while a suggestion is shown appends it as typed text and the word continues.
  Without a suggestion, Tab keeps every current meaning (MI-8, MI-13, MI-24).
- TP-3 A suggestion is never committed by Enter, Space, focus loss, or any other commit;
  only accepting it adds its text.
- TP-4 Any other key hides the current suggestion before it is handled, so typing
  through a suggestion behaves as if it were not there.
- TP-5 English completion: prefix of three or more lowercase letters inside an English
  word, the word itself not finished by Space; the suggestion is the remainder of the
  best-ranked listed word with the prefix, at least two letters long.
- TP-6 No suggestion inside a path, URL, or identifier token (MI-20, MI-21).
- TP-7 No learning, no history, no disk or network access. A user lexicon remains a
  separate roadmap item (10) with its own decision record.

## Draft acceptance cases

- `docu` shows suggestion `ment`; Tab, Enter commits `document`.
- `docu`, Enter commits `docu` (the suggestion is not committed).
- `docum` then `e`: the suggestion follows the new prefix.
- `ai`: no suggestion (too short); Tab still flips.
- `su3cl3`: no English suggestion; Tab still flips the last syllable.
- `~/doc`: no suggestion inside a path.
- Idle Tab still passes through to the application.

## Shell work

- macOS: draw the `suggestion` segment dimmed. Marked text styling is decided by the
  host application and some hosts ignore colors, so the panel may need to show the
  suggestion instead; to be checked in the hand test.
- Windows: the TSF display attribute for the segment; unverified until a machine with
  administrator rights exists.
- Older shells that do not know the segment kind would show it as normal text; the
  engine and shells ship together, so this is acceptable.

## Prior art

- Code editors accept inline suggestions with Tab only while one is visible; otherwise
  Tab keeps its normal meaning. keyflip adopts that rule (TP-2).
- Command shells with autosuggestions accept with Right at the end of the line, which
  is why Right is the recommended key for Chinese continuation.
- Microsoft New Zhuyin can offer associated phrases after a commit. keyflip keeps them
  inside the composition instead, so committing never pops up a second list.
