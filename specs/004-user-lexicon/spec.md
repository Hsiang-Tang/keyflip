# Specification: user lexicon and learning (004-user-lexicon)

Status: implemented 2026-10-21; macOS hand test pending. Roadmap item 10.
Decision record: `docs/adr/0007-user-data-lives-beside-the-engine.md`.

## Goal

When the default conversion is wrong and the maintainer picks the right word from the
candidate window, the next time the same syllables are typed the right word comes out
by itself. Words the lexicon lacks (names, project terms) can be added by hand.
Microsoft New Zhuyin learns the same way (constitution principle 7).

## Requirements

- UL-1 Learning. When text is committed, every candidate picked by hand in it is
  recorded: the readings it covers, its text, and the reading of the syllable just
  before it (its context; empty at the start of a run). A pick that is never committed
  (Esc, Backspace over it) is not recorded.
- UL-2 Candidate order. Learned choices for exactly the syllables under the cursor
  come first in the candidate window, most recent first; the usual order (MI-10)
  follows.
- UL-3 Default conversion. A learned word of two or more characters is preferred
  wherever its syllables appear. A learned single character is preferred after the
  same preceding syllable, and anywhere once it has been picked three times. One pick
  of 地 in 慢慢地 must not turn every ㄉㄜ˙ into 地.
- UL-4 User phrases. A hand-edited file adds words, one per line as
  `text<TAB>readings`, readings separated by spaces (`鍵翻\tㄐㄧㄢˋ ㄈㄢ`). They convert
  and list like lexicon words, ranked above the most frequent lexicon word.
- UL-5 Inspectable. Both files are UTF-8 text with a header that explains the format.
  Deleting a line forgets that item; deleting a file forgets everything in it.
- UL-6 Deletable from the input menu: "Clear learned choices" and "Open user
  dictionary folder".
- UL-7 Robust. Malformed lines are skipped, never fatal. When a file had malformed
  lines, a copy is kept as `<name>.corrupt` before the learned file is rewritten. Writes
  go to a temporary file renamed into place, so a crash never leaves half a file. The
  learned file keeps at most 5,000 items; the least recently used are dropped.
- UL-8 Private. Files stay on this Mac under
  `~/Library/Application Support/keyflip/`; nothing is sent anywhere. Password fields
  give the input method no keys (macOS secure input), so nothing typed there is
  learned.
- UL-9 Hand edits are picked up the next time keyflip is activated (switching to it,
  or to another text field).
- UL-10 The engine stays pure: it holds the user model in memory and never touches
  the disk (constitution principle 5); the C interface layer loads and saves the files.
- UL-11 English habits (maintainer request: `doc` suggests `doctor`, but `docker` is
  the word the maintainer uses). When text is committed, each English word in it of 5 to 20
  lowercase letters that stands alone (not glued to a digit, `_`, `@`, `.`, `/`, `-`,
  `'`, or a capital) is counted. A word counted at least twice is suggested for its
  prefix (TP-5 rules: three or more letters typed, at least two letters added) before
  the word list's choice: most used first, then most recent. Words outside the bundled
  list (`docker`, `kubectl`) qualify. Once is not enough, so a typo or a one-off string
  never becomes a suggestion.
- UL-12 Learned English words live in `english.tsv`
  (`word<TAB>count<TAB>last use`), at most 2,000 words, least recently used dropped,
  with the same robustness rules as UL-7. "Clear learned choices" clears it too. The
  file lists every qualifying word typed at least once outside password fields,
  including in terminals.

## Acceptance cases

Behavior tests in `crates/keyflip-core/tests/user.rs`:

- ㄉㄜ˙ after ㄇㄢˋ: pick 地 and commit; typing 慢慢 + ㄉㄜ˙ again gives 地, while
  ㄉㄜ˙ after another syllable still gives 的.
- A single character picked three times is preferred everywhere.
- A picked word is preferred wherever its syllables appear and is listed first.
- A pick abandoned with Esc or never committed is not learned.
- A user phrase converts by default and is listed.
- Malformed lines are skipped and reported; the serialized file parses back to the same
  model; the learned list is capped and drops the least recently used.
- Without a user model the engine behaves exactly as before (evaluation unchanged).
- `doc` suggests `tor`; after `docker` is typed and committed twice it suggests `ker`;
  once is not enough; `my_docker` and `docker2` are not counted.

## Not doing

- Learning Chinese from words typed without picking (frequency counting of everything
  typed): it records far more of what is typed, and the lexicon order is already right
  for those. English words are counted for completion only (UL-11).
- Syncing between machines. The files can be copied by hand.
- Learning for English judgment (Chinese or English). It keeps using key structure and
  the word list only; learning affects completion (UL-11), not the language decision.
