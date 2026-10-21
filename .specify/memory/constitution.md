# keyflip constitution

Principles that every spec, plan, and change must respect. Changing one requires an
ADR in `docs/adr/`.

1. **No mode toggle gestures.** Language is decided from the key sequence alone.
   No single-tap Shift, CapsLock, or other modifier-only gesture, ever (ADR-0002). The
   CapsLock light only means uppercase letters (ADR-0006).
2. **One brain.** All language judgment lives in `keyflip-core`. Platform shells
   only forward keys and draw the snapshot they are given (ADR-0001).
3. **Snapshot protocol.** One key in, one complete state out. Shells keep no
   judgment state.
4. **Fail open.** If the engine is unavailable or slow, shells let every key
   through. The input method must never trap the user's typing.
5. **Privacy.** Password and read-only fields receive no keys and nothing is
   recorded. No telemetry. The engine never touches the network or the disk.
6. **Test-first behavior.** Rules become acceptance cases before code. Cases are
   never deleted or loosened to make code pass. Measured error rates are tracked
   by `keyflip-cli eval`.
7. **Stay in scope.** Only the Chinese/English switching problem is solved. Every
   other behavior follows the familiar Microsoft New Zhuyin habits.
8. **Honest verification.** Unverified behavior is recorded as unverified in
   `docs/VERIFICATION.md` and `docs/STATUS.md`.
