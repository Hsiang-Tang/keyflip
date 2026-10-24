# Plan: user lexicon and learning (004-user-lexicon)

## Layers

```
 Swift shell ── menu: clear, open folder; activate: reload
     │
 keyflip-ffi ── owns the files: load, save after a commit that learned, reload on mtime
     │
 keyflip-proto Shared ── one Arc<Mutex<UserModel>> for every session
     │
 keyflip-core UserModel ── parse, serialize, learn, rank; no I/O
```

## Core

- `user::UserModel` holds user phrases (from `phrases.txt`) and learned choices (from
  `learned.tsv`). `parse_*` return the model plus the number of malformed lines;
  `learned_tsv()` serializes. A change bumps a generation number so the file layer
  knows when to save.
- Learned item key: (readings, text, context reading). Value: count and a use sequence
  number (not a clock time; only the order matters for "most recent" and eviction).
- Engine: `Engine::with_user(Arc<Mutex<UserModel>>)`. `select_candidate` remembers the
  pick on the groups it covers; `commit_all` records the picks that are still intact.
  `would_handle` runs on a clone with learning switched off, so the pre-check never
  learns.
- Conversion: `UserModel::preferences` gives the preferred character at each syllable
  of a run (UL-3); the context of a syllable is the reading before it, known before the
  search, so the best-path search stays exact. An entry that agrees with every
  preference it covers scores as if its count were `USER_COUNT` (1,000,000, above 的 at
  615,175); each disagreeing syllable costs the same amount. Boosting only the learned
  entry was not enough: phrases such as 慢慢的 (84) and 你的 (2,285) contain the
  unwanted character and outscored a single boosted 地.
- English completion (UL-11): the engine asks the user model before the word list.
  Words are counted at commit; a word needs two uses before it is suggested.
- Short words (UL-13): Tab on pending keys marks the letters it produces as kept; a
  standalone all-kept run of 2 to 4 letters is recorded at commit. While the pending
  keys spell a recorded word, the snapshot shows the letters and a commit or Space
  emits them, but the keys stay pending so a tone key still completes the syllable.
- Candidates: learned texts for the exact span first (most recent first), then user
  phrases, then the lexicon order.

## Files

`~/Library/Application Support/keyflip/`:

- `phrases.txt`: written by the maintainer; created with a header on first run.
- `learned.tsv`: `readings<TAB>text<TAB>context<TAB>count<TAB>sequence`, and
  `english.tsv`: `word<TAB>count<TAB>sequence`, written by keyflip after each commit
  that learned something. `short-english.tsv`: `word<TAB>sequence` (UL-13, UL-14).

Saving writes a `.tmp` file, then renames it. Loading a written file with malformed
lines copies it to `.corrupt` first. Clearing copies both written files to `.bak`.
Reload compares modification times on activation.

## Shell

- `keyflip_user_open(dir)` once at startup (before sessions), `keyflip_user_reload()`
  on `activateServer`, `keyflip_user_clear()` from the menu.
- IMK `menu()` returns the two items (UL-6).
