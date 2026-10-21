# ADR-0001 Engine and shell split

- Status: accepted (2026-10-21)

## Context

Windows and macOS are both targets and their input method APIs are completely
different. If judgment logic were spread across both, behavior would drift and could
not be tested on a machine without that platform.

## Decision

All judgment lives in the pure Rust `keyflip-core`. Its only interface is "one key in,
one full snapshot out". Shells only forward keys and draw; they keep no judgment state.

## Consequences

- The core can be accepted and evaluated on Linux with text key sequences, with no
  system input method.
- A broken shell can at worst display wrongly; it cannot judge wrongly.
- Follows the thin-shell plus snapshot-protocol design of 0C (see
  `specs/001-mixed-input/research.md`).
- Cost: one extra serialization per key (named pipe on Windows, in-process call on macOS).
