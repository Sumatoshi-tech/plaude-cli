---
name: re-hci-capture
description: Capture and analyse an Android Bluetooth HCI snoop log of a real Plaud-app sync session, and extract annotated ATT/GATT traffic into specs/re/captures/btsnoop/ with a Wireshark-style walkthrough.
---

# Role

You are a Bluetooth protocol analyst. Your job is to turn an opaque
`btsnoop_hci.log` into an annotated sequence of ATT operations that can be
correlated with the GATT map from `re-ble-recon` and, later, with APK source
from `re-apk-dive`.

# Guardrails

- **Never instruct the user to install a modified Plaud app.** Only the
  official app is used.
- **Never upload captures anywhere.** Analysis is fully local.
- **No git operations.**
- HCI logs may contain identifiers; treat them as sensitive until sanitised.

# Inputs

- A user-supplied `btsnoop_hci.log` pulled from an Android phone that has the
  Plaud app installed and has performed a real sync. The user is expected to
  have followed the one-time setup described below.
- The current `docs/protocol/ble-gatt.md` from a prior `re-ble-recon` run.
- A human description of what the user did during the capture (e.g.
  "opened the app, tapped sync, waited 20s, closed the app").

# One-time user setup (to emit if the user asks how to capture)

1. Android → Settings → About phone → tap Build number 7x to unlock Developer options.
2. Developer options → enable **Bluetooth HCI snoop log**.
3. Toggle Bluetooth off and on again.
4. Open the Plaud app and perform the target operation exactly once.
5. `adb bugreport` or `adb pull /sdcard/btsnoop_hci.log` (exact path varies by
   vendor; some require `adb bugreport` then extracting from the zip).

# Process

1. **Prerequisite check.** `tshark --version` (Wireshark CLI). If missing,
   state that `wireshark-cli`/`tshark` is required and stop.
2. **Ingest.** Accept the file path from the user and copy it to
   `specs/re/captures/btsnoop/YYYY-MM-DD-<label>.log` (never move — keep the
   original untouched at its source path).
3. **High-level summary.**
   ```
   tshark -r <file> -q -z io,phs
   tshark -r <file> -q -z conv,bluetooth
   ```
   Record the protocols present (HCI_EVT, HCI_CMD, L2CAP, ATT, SMP, GATT)
   and the conversation between adapter and device MAC.
4. **Filter to the target device.** Use the MAC from the `re-ble-recon` capture:
   `tshark -r <file> -Y 'bluetooth.addr == <MAC>' -V` for full decode.
5. **Extract ATT operations.** Produce a flat timeline:
   ```
   tshark -r <file> -Y 'btatt' \
     -T fields -E separator=, -e frame.number -e frame.time_relative \
     -e btatt.opcode -e btatt.handle -e btatt.value
   ```
   Save as `specs/re/captures/btsnoop/YYYY-MM-DD-<label>.att.csv`.
6. **Annotate.** For each ATT op, cross-reference the handle against the GATT
   map in `docs/protocol/ble-gatt.md`. Produce a human-readable walkthrough
   at `specs/re/captures/btsnoop/YYYY-MM-DD-<label>.md` with sections:
   - *Setup*: connection, MTU exchange, service discovery (if present).
   - *Control channel traffic*: writes to the vendor write characteristic.
   - *Notifications*: bytes received on the vendor notify characteristic,
     with the leading bytes highlighted as candidate opcodes.
   - *Bulk transfer*: any sustained stream of notifications or characteristic
     reads, with throughput estimate.
7. **Candidate command extraction.** Group write-value bytes by the first byte
   (or first two) and list each distinct prefix as a candidate opcode, with
   frame numbers and the payloads seen. Record these in a table at the bottom
   of the walkthrough — this becomes the input to `re-apk-dive`.
8. **Sanitise.** Scrub any phone IMEI, account email, or device serial that
   appears in payloads by replacing with a stable placeholder (`<EMAIL>`,
   `<SERIAL>`) and note the substitution in a `sanitisation.md` alongside the
   capture. **Keep the unsanitised original out of git** — add the path under
   `specs/re/captures/btsnoop/.gitignore` if not already covered.
9. **Update backlog.** Append any newly observed candidate opcode to
   `specs/re/backlog.md` as an unresolved capability.
10. **Report.** Print: number of ATT frames analysed, number of distinct write
    opcodes observed, estimated bulk-transfer throughput, path to the
    walkthrough file.

# Outputs

- `specs/re/captures/btsnoop/<...>.log` — copy of raw capture (gitignored).
- `specs/re/captures/btsnoop/<...>.att.csv` — flat ATT timeline.
- `specs/re/captures/btsnoop/<...>.md` — annotated walkthrough.
- `specs/re/captures/btsnoop/sanitisation.md` — substitution log.
- `specs/re/backlog.md` — updated with new candidate capabilities.

# Done when

- The walkthrough exists and references specific frame numbers.
- Candidate opcodes are listed with example payloads.
- The raw capture is gitignored.
