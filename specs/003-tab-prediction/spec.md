# Specification: Tab prediction (003-tab-prediction)

Status: implemented 2026-10-21 (English word completion with Tab only); macOS hand
test pending. Chinese
continuation (associated phrases) is out of scope; see "Not doing". Roadmap item 9.

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

## Scope

English word completion: after three or more letters of an English word, suggest the
most common listed word (rank under 10,000) that extends the prefix by at least two
letters. Source: `data/english.txt`, already bundled.

## Not doing

Chinese continuation (associated phrases, such as 腦 after 電) was considered and
dropped on 2026-10-21. Reasons: it is outside the Chinese/English switching problem
(constitution principle 7); without typing history the lexicon can only guess by
frequency, so it guesses wrong often; a wrong guess after every character is visual
noise; and Tab cannot accept it without hiding the flip of the last syllable. Reopen
only if daily use shows a real need.

## Requirements

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
  best-ranked listed word with the prefix, at least two letters long. When a shorter
  listed word with the prefix is a prefix of that word, the shorter one is used
  (`document`, not the more frequent `documents`); the word continues after Tab, so a
  plural is one more key.
- TP-6 No suggestion inside a path, URL, or identifier token (MI-20, MI-21).
- TP-7 No learning, no history, no disk or network access. A user lexicon remains a
  separate roadmap item (10) with its own decision record.
- TP-8 Right arrow while a suggestion is shown accepts it exactly like Tab. A suggestion
  is shown only with the cursor at the end, where Right otherwise does nothing, so no
  existing meaning changes; with a candidate window open, Right keeps MI-22. Reason:
  some web pages act on the Tab key themselves even while text is being composed. The
  browser delivers the key to the page before the completed text, and the page commits
  the old composition (seen in the YouTube Music search box, 2026-10-21; a plain input
  and Notes work). Pages rarely act on Right, and shells accept suggestions with it
  (fish, zsh-autosuggestions).
- Known limit: the YouTube Music search box acts on Right as well (Google-style search
  boxes accept their own inline completion with Tab and Right), so completion does not
  work there. Repairing it in the shell after the page's commit was considered and
  dropped by the maintainer (2026-10-21): one site, and other pages work.

## Acceptance cases

- `docu` shows suggestion `ment`; Tab, Enter commits `document`.
- `docu`, Enter commits `docu` (the suggestion is not committed).
- `docum` then `e`: the suggestion follows the new prefix.
- `ai`: no suggestion (too short); Tab still flips.
- `su3cl3`: no English suggestion; Tab still flips the last syllable.
- `~/doc`: no suggestion inside a path.
- Idle Tab still passes through to the application.
- `docu`, Right, Enter commits `document`; `docu`, Left, Right moves the cursor only.

## Shell work

- macOS: the suggestion is shown in the floating panel below the caret (typed part
  dimmed, completion in full strength, then "Tab"), never in the marked text. The first
  hand test showed a host drawing the whole marked text reversed, so a dimmed segment
  there looked typed.
- Windows: the segment is drawn like other composition text (underlined, not dimmed);
  unverified until a machine with administrator rights exists.
- Older shells that do not know the segment kind would show it as normal text; the
  engine and shells ship together, so this is acceptable.

## Prior art

- Code editors accept inline suggestions with Tab only while one is visible; otherwise
  Tab keeps its normal meaning. keyflip adopts that rule (TP-2).
