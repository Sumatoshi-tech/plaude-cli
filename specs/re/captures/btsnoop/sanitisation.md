# Sanitisation log — btsnoop captures

This file documents substitutions applied to btsnoop-derived markdown
walkthroughs before they are committed. The raw `.log` and `.att.csv`
files remain unredacted on disk and are kept out of git by the root
`.gitignore`.

## 2026-04-05-plaud-sync-session

| Original | Substitute | Rationale |
|---|---|---|
| First control-write payload: 32 ASCII-hex chars (128-bit auth token) tied to the current phone↔device pairing | `<AUTH_TOKEN_32HEX>` | Leaking a session/auth fingerprint across commits is unnecessary. The **structure** of the field (length, encoding) is preserved; only the **value** is redacted. |
| Phone Bluetooth address `8C:C5:D0:9C:1F:AB` | kept verbatim | Phone BT addresses are not sensitive for the purposes of this project; keeping them helps future analysts cross-reference the trace. If this project ever moves toward a public repository, re-evaluate. |
| Device serial `888…7881` | n/a | Not present in any ATT payload. Verified by grep against the full ATT value stream. |
| Plaud RPA `D1:A6:DE:62:DF:14` | kept verbatim | RPA is an ephemeral random address, not a stable identifier. |

## Global policy

- Never paste the 32-hex auth token into any committable file.
- If a future capture contains the device serial in any payload, redact
  with `<SERIAL>`.
- If a future capture contains an account email or phone IMEI, redact
  with `<EMAIL>` / `<IMEI>`.
- Every substitution gets a row in this file.
