# ADR-0003 Optional engine feature in the protocol library

- Status: accepted (2026-10-21)

## Context

`keyflip-proto` is used by both the server (which needs the engine and lexicon) and the
TSF DLL (which needs only the types). Linking the engine by default embeds the 5.8 MB
lexicon in the DLL, making it 7.3 MB, and every program that loads the input method pays
for it.

## Decision

`engine` is an optional feature of proto, on by default. The TSF crate depends on it with
`default-features = false`. `tools/build-windows.sh` builds the DLL and the server in two
separate cargo invocations so feature unification does not pull the engine back into the DLL.

## Consequences

The DLL is 1.36 MB and carries no lexicon. The two build invocations must not be merged.
