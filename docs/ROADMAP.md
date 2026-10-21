# keyflip roadmap

Order is priority. Each finished item gets an entry in `docs/VERIFICATION.md`.

## Now: make the macOS build usable

1. Build with `macos/build.sh` (done 2026-10-21).
2. Walk the manual checklist in `specs/002-platform-shells/macos.md`. A failure that
   is a judgment problem becomes a core acceptance case first, then gets fixed.
3. Use it for daily typing for a week, note misjudged words, and adjust rules or add
   cases.

## Next

4. Settings UI: dropped 2026-10-21 (see "Explicitly not doing").
5. Input method icon: done 2026-10-21 on macOS (vector template `MenuIcon.pdf`, drawn
   by `macos/make-icon.swift`). Windows still has no icon.
6. Identifier protection: joiners and punctuation width done 2026-10-21 (MI-20, MI-21).
   Remaining: output protected ranges in the snapshot once a shell needs them.
7. Windows: run `installer/install.ps1` on a machine with administrator rights and
   verify with `specs/002-platform-shells/windows.md`. Rebuild `dist/`; the current
   binaries predate the engine's Up-arrow change and are stale.
8. Publish the repository (license work is done: `LICENSE`, `NOTICE`,
   `THIRD_PARTY_LICENSES.md`, English list from wordfreq). Done 2026-10-21: made
   public by the maintainer after a full-history scan for private data.

## Version 2

9. Tab prediction (accept a suggestion, like VS Code). Draft design:
   `specs/003-tab-prediction/`; English completion done 2026-10-21, Chinese associated
   phrases dropped.
10. User lexicon and learning (candidate memory); must be inspectable, deletable, and
    recoverable after corruption (TronIn's approach).
11. Signing and notarization (macOS) and installer signing (Windows), only needed for
    distribution to others.

## Explicitly not doing

- Single-tap modifier gestures (ADR-0002).
- Fast-typing error correction.
- Settings UI (2026-10-21, maintainer). Space between Chinese and English: not wanted,
  a Space typed by hand is enough. Full-width punctuation: rarely used; Shift+Space
  per application is enough, so the state is not remembered either. Windows keeps its
  `config.json` for spacing. Revisit when a real setting is needed (for example the
  user lexicon, item 10).
- Support for users who do not type tones.
