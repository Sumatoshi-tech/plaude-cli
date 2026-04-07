# plaud-proto test fixtures

Each `.bin` file in this directory is a tiny, self-contained wire
example the codec's round-trip tests exercise. Every fixture cites
its provenance below so future engineers can trace a byte back to
the btsnoop capture it came from.

Fixtures are built at test runtime by the helpers in the individual
test files (via `include_bytes!` and/or literal `[u8; N]`), so this
directory intentionally ships no raw `.bin` files. The rationale:

* keeping the test data inline with the test keeps the relationship
  between **"bytes → assertion"** grep-visible,
* it avoids the forensic risks of committing any raw capture bytes
  that could contain residual device identifiers (serial numbers,
  ChaCha nonces) we have not sanitised,
* every constant matches its `const` in `src/constants.rs`, so a
  fixture renaming / reformat is a single-edit-site change.

Every fixture in this table is reconstructed from a **committable**,
annotated walkthrough document — never from a gitignored raw log.

| Fixture | Source walkthrough | Evidence pointer |
|---|---|---|
| V0095 auth request layout | `specs/re/captures/btsnoop/2026-04-05-plaud-sync-session.md` §"Authenticate" + live-tested via `specs/re/captures/ble-live-tests/2026-04-05-token-validation.md` | wire prefix `01 01 00 02 00 00` then 32 ASCII hex chars (token redacted to a deterministic placeholder) |
| Auth response (accepted) | `specs/re/captures/btsnoop/2026-04-05-plaud-0day-pair.md` §3 | `01 01 00 00 0a 00 03 00 01 01 00 00 56 5f 00 00` |
| Auth response (rejected) | `specs/re/captures/ble-live-tests/2026-04-05-token-validation.md` Test 2a | `01 01 00 01 0a 00 03 00 01 01 00 00 56 5f 00 00` |
| GetDeviceName request | `specs/re/captures/btsnoop/2026-04-05-plaud-sync-session.md` §"GetDeviceName" | `01 6c 00` |
| GetDeviceName response | same | `01 6c 00 50 4c 41 55 44 5f 4e 4f 54 45 …` (ASCII "PLAUD_NOTE" + padding zeros) |
| ReadFileChunk request | `specs/re/captures/btsnoop/2026-04-05-plaud-0day-pair.md` §"Session C" | `01 1c 00 58 66 d2 69 80 4c 01 00 c0 57 01 00` |
| Bulk frame (data chunk) | `specs/re/captures/btsnoop/2026-04-05-plaud-sync-session.md` §"Bulk frame" | synthetic: fixed file id, offset 0, 80 bytes of constant payload |
| Bulk frame (end-of-stream) | `specs/re/captures/btsnoop/2026-04-05-plaud-0day-pair.md` §7 | synthetic: `offset = 0xFFFFFFFF` |
| Handshake `0xFE12` preamble | `specs/re/apk-notes/3.14.0-620/architecture.md` §"Mode B" | `12 fe` (the handshake type read as a `u16` LE at offset 0) |
