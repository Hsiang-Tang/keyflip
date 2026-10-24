# ADR-0007 User data lives beside the engine, in plain text

- Status: accepted (2026-10-21).

## Context

Learning from candidate picks and English words (`specs/004-user-lexicon/`) means
keeping what the maintainer typed across sessions. Constitution principle 5 says the
engine never touches the disk, and that password fields record nothing. TronIn's
lesson for user dictionaries: they must be inspectable, deletable, and survive
corruption.

## Decision

- `keyflip-core` holds the user model in memory only (`user::UserModel`): parsing,
  serializing, learning, and ranking, with no I/O. Every session shares one model.
- `keyflip-ffi` owns the files in `~/Library/Application Support/keyflip/`:
  `phrases.txt` (written by the user, never rewritten), `learned.tsv`,
  `english.tsv`, and `short-english.tsv` (written by keyflip after a commit that learned something).
- Files are UTF-8 text with explanatory headers. Writes go through a temporary file
  and a rename. Malformed lines are skipped; a written file that had any is copied to
  `.corrupt` and rewritten clean. Clearing keeps `.bak` copies.
- Only committed hand picks and stand-alone lowercase English words are learned.
  macOS gives input methods no keys in password fields, so nothing typed there reaches
  the model.

## Consequences

- Principle 5 holds as written: the engine stays free of I/O.
- The files reveal some of what was typed; they never leave the Mac and can be cleared
  from the input menu.
- Learning affects ranking and completion only, never the Chinese/English decision, so
  `keyflip-cli eval` (which runs without a user model) still measures the rules alone.
