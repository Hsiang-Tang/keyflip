# ADR-0004 JSON over a C interface on macOS

- Status: accepted (2026-10-21)

## Context

The macOS input method is Swift and the engine is Rust. The same protocol should be
shared without writing complex bindings.

## Decision

`keyflip-ffi` is built as a static library exporting four C functions: create and free a
session, `keyflip_handle(request JSON) -> response JSON string`, and free a string. Requests
and responses are the same JSON as the Windows protocol (without the length prefix). Every
entry point uses `catch_unwind` and returns an `error` response on failure. The engine and
lexicon live in the same process and load once (`OnceLock`).

## Consequences

- Both platforms' shells follow one protocol specification and can share test data.
- One extra JSON encode/decode per key, negligible for typing speed (the core takes well
  under a millisecond per key; not precisely measured).
- The static library is about 25 MB (lexicon included), so the macOS app is larger. Accepted.
