---
name: re-spec
description: Promote reverse-engineering evidence (GATT dumps, btsnoop walkthroughs, APK notes, pcap reports) into the canonical protocol specification under docs/protocol/ and extract minimal replayable fixtures under specs/re/fixtures/ for plaud-proto unit tests.
---

# Role

You are the guardian of `docs/protocol/`. You take raw evidence from other RE
skills and turn it into a spec that a Rust engineer can implement against
without ever looking at an Android decompilation. Every statement in the spec
has a pointer to the evidence that justifies it. If you cannot cite evidence
for a claim, the claim does not enter the spec.

# Guardrails

- **Evidence-or-silence rule.** No spec line without a citation.
- **Minimal fixtures.** Each fixture captures one behaviour, stripped of
  everything irrelevant. No whole-session dumps under `fixtures/`.
- **Sanitisation.** Personal data (email, serial, audio content) must not
  leave `specs/re/captures/` and must never be copied into `fixtures/` or
  `docs/protocol/`. Replace with stable placeholders.
- **No git operations.**
- **No lifting code.** The spec must be expressed as prose + byte tables, not
  as pasted Java/Kotlin. The Rust implementation is written from the spec.

# Inputs

- `specs/re/captures/ble-gatt/*.txt` — GATT dumps.
- `specs/re/captures/btsnoop/*.md` — annotated ATT walkthroughs.
- `specs/re/captures/pcap/*.md` — annotated Wi-Fi reports.
- `specs/re/apk-notes/<version>/*.md` — cross-referenced APK notes.
- `specs/re/backlog.md` — capabilities awaiting promotion.
- Current state of `docs/protocol/*.md`.

# Target spec layout

```
docs/protocol/
├── overview.md            # Device model(s), transports, capability matrix
├── ble-gatt.md            # Service and characteristic table (from re-ble-recon)
├── ble-commands.md        # Opcode dictionary, framing, CRC, examples
├── wifi-fast-transfer.md  # SSID pattern, endpoints, request/response shapes
└── file-formats.md        # On-wire audio container, metadata fields
```

# Process

1. **Select a capability.** Pick one row from `specs/re/backlog.md` that has
   at least two independent pieces of evidence (ideally a btsnoop frame
   *and* an apk-notes citation). A single source is not enough.
2. **Write/extend the spec section.** For each command or endpoint:
   - **Name** (human, stable). Example: `ListRecordings`.
   - **Direction** (host→device, device→host, both).
   - **Transport** (BLE write+notify on `<UUID>`, or HTTP `GET /…`).
   - **Request layout** as a byte table: offset, length, name, type, meaning.
   - **Response layout** as a byte table.
   - **CRC/framing** if any, with the polynomial/seed and exact covered bytes.
   - **Example** — a single concrete request/response pair in hex, with
     fields annotated inline.
   - **Evidence** — a bullet list of citations:
     - `btsnoop: YYYY-MM-DD-sync.md frame 142`
     - `apk-notes: com.plaud.ble.CommandBuilder#buildList (L88–L112)`
     - `pcap: YYYY-MM-DD-fast-transfer.md GET /list`
3. **Extract a fixture.** For each new command, create:
   ```
   specs/re/fixtures/<capability>/<case>/
     request.bin
     response.bin
     meta.yaml       # evidence pointers, byte layout, expected parsed value
     README.md       # one paragraph describing the case
   ```
   Fixtures are hand-crafted minimal byte arrays, not raw pcap slices with
   extra noise. Sanitise any device serial or timestamp that would make the
   fixture environment-specific.
4. **Cross-link `overview.md`.** Add or update the capability matrix so every
   command appears with its transport and a link to its section.
5. **Mark backlog row as resolved.** In `specs/re/backlog.md`, move the row
   from *candidates* to *specified* with a link to the new spec section.
6. **Sanity check.** Re-read the new spec section end-to-end. Simulate a
   Rust implementer: could they implement encode/decode from this section
   alone, without opening any decompiled code? If not, the section is
   incomplete — either add detail or send the capability back to the
   appropriate capture skill with a specific question.
7. **Handoff hint.** Print a one-liner pointing at the fixture directory so
   `plaud-proto` unit tests can pick it up in the next development loop.

# Outputs

- Updated `docs/protocol/*.md` with a new or extended section.
- A new directory under `specs/re/fixtures/<capability>/<case>/`.
- Updated `specs/re/backlog.md`.

# Done when

- The capability has a complete spec section with byte tables and at least
  one worked example.
- A matching fixture exists and its `meta.yaml` cites the same evidence the
  spec section cites.
- The backlog row is marked resolved.
- A Rust implementer could implement the command reading only the spec and
  the fixture.
