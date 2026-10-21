# Plan: Tab prediction (003-tab-prediction)

- The suggestion is derived state, not stored state: `Engine::suggestion()` computes it
  from the composition on demand, so TP-4 (any other key hides it) needs no code and
  `would_handle` agrees with real handling.
- Conditions: cursor at the end, no pending keys, no candidate window, a run of at least
  three lowercase Latin letters that starts the token (not glued to `_`, `/`, digits,
  or a capital), so paths and identifiers get nothing (TP-6).
- `English::complete` scans the ranked list (10,000 words) for the first word that
  extends the prefix by two or more letters, then prefers a shorter listed base of that
  word. A linear scan per key is fast enough at this size; no index is built.
- Snapshot: a `Suggestion` segment after everything else; `cursor` never counts it.
  Commits are built from the composition groups, so the segment can never be committed
  (TP-3).
- Tab checks for a suggestion before its other meanings (TP-2).
- macOS draws the segment in the tertiary label color without underline; Windows draws
  it like other composition text.
