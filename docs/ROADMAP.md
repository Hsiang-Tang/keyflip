# keyflip roadmap

Order is priority. Each finished item gets an entry in `docs/VERIFICATION.md`.
macOS is the only supported platform (ADR-0008).

## Now: make the macOS build usable

1. Build with `macos/build.sh` (done 2026-10-21).
2. Walk the manual checklist in `specs/002-platform-shells/macos.md`. A failure that
   is a judgment problem becomes a core acceptance case first, then gets fixed.
3. Use it for daily typing for a week, note misjudged words, and adjust rules or add
   cases.

## Next

4. Settings UI: dropped 2026-10-21 (see "Explicitly not doing").
5. Input method icon: done 2026-10-21 (menu bar template `MenuIcon.pdf` and app icon
   `AppIcon.icns`, both drawn by `macos/make-icon.swift`).
6. Identifier protection: joiners and punctuation width done 2026-10-21 (MI-20, MI-21).
   Remaining: output protected ranges in the snapshot once a shell needs them.
7. Windows install: dropped 2026-10-21 (ADR-0008).
8. Publish the repository (license work is done: `LICENSE`, `NOTICE`,
   `THIRD_PARTY_LICENSES.md`, English list from wordfreq). Done 2026-10-21: made
   public by the maintainer after a full-history scan for private data.

## Version 2

9. Tab prediction (accept a suggestion, like VS Code). Draft design:
   `specs/003-tab-prediction/`; English completion done 2026-10-21, Chinese associated
   phrases dropped.
10. User lexicon and learning: done 2026-10-21 (`specs/004-user-lexicon/`, ADR-0007).
    Learned picks, user phrases, and English completion learned from typing; hand test
    pending.
11. Signing and notarization: `macos/release.sh` done 2026-10-21 (Apple Silicon zip).
    Waiting for the maintainer's one-time setup: a Developer ID Application
    certificate and a `notarytool` keychain profile (steps in the script header).
12. First public release: run `macos/release.sh`, attach the zip to a GitHub release,
    and add install steps for people who do not build from source.

## Explicitly not doing

- Single-tap modifier gestures (ADR-0002).
- Fast-typing error correction.
- Settings UI (2026-10-21, maintainer). Space between Chinese and English: not wanted,
  a Space typed by hand is enough. Full-width punctuation: rarely used; Shift+Space
  per application is enough, so the state is not remembered either. Revisit when a
  real setting is needed.
- Windows (ADR-0008).
- Support for users who do not type tones.
