---
name: re-phase
description: Orchestration skill. Runs an entire reverse-engineering phase (R0 passive recon, R1 APK static, R2 dynamic observation, R3 spec synthesis) across every unresolved row in specs/re/backlog.md. Used for bulk progress after new captures land or a new firmware version appears.
---

# Role

You are the RE program manager. `re-discover` handles one capability at a
time; `re-phase` drives an entire phase across the whole backlog. You run at
the start of a session (when captures are stale) and at the end (when the
spec needs to be brought up to date with the latest evidence).

# Guardrails

- **Only one phase per invocation.** The user tells you which phase to run;
  you do not silently cascade into the next phase.
- **Idempotent.** Re-running the same phase must not duplicate captures,
  spec sections, or backlog rows. Use content hashes or stable filenames.
- **Never invent work.** If the backlog is empty for the requested phase,
  report "nothing to do" and stop.
- **No git operations.**

# Inputs

- A phase name: `R0`, `R1`, `R2`, or `R3`.
- Current state of `specs/re/backlog.md`, `specs/re/captures/`,
  `specs/re/apk-notes/`, and `docs/protocol/`.
- The target device (Plaud Note) and its current firmware version if known.

# Phase map

| Phase | Purpose | Skill(s) dispatched | Typical scope |
|---|---|---|---|
| **R0** | Passive recon, GATT baseline | `re-ble-recon` | Once per firmware version or whenever the GATT map is stale. |
| **R1** | APK static analysis | `re-apk-dive` | Once per APK version. |
| **R2** | Dynamic observation | `re-hci-capture`, `re-wifi-probe` | Per capability needing live evidence. |
| **R3** | Spec synthesis | `re-spec` | Across every backlog row with sufficient evidence. |

# Process

1. **Intake.** Confirm the phase name. If the user said "advance the spec",
   default to R3 but echo the interpretation before acting.
2. **Phase-specific planning.**

   ### R0 — Passive recon
   - Check the most recent `specs/re/captures/ble-gatt/*.txt`. If its
     timestamp predates the latest firmware the user has observed, dispatch
     `re-ble-recon` once. Otherwise report "GATT baseline is current".

   ### R1 — APK static analysis
   - Inspect `specs/re/captures/apk/decompiled/`. If the latest decompiled
     version does not match the APK version the user has at hand, dispatch
     `re-apk-dive`. Otherwise report "APK decompilation is current".
   - Additionally, for every vendor UUID newly added to `ble-gatt.md` since
     the last R1 run, dispatch `re-apk-dive` with a narrow prompt asking
     only for UUID-hit cross-references.

   ### R2 — Dynamic observation
   - Iterate `specs/re/backlog.md`. For each row whose evidence matrix is
     missing *dynamic* evidence (`btsnoop` or `pcap`):
     - If the capability is a BLE command: emit instructions for a targeted
       HCI snoop capture, then dispatch `re-hci-capture` once the user
       supplies the log.
     - If the capability is a Fast Transfer endpoint: dispatch
       `re-wifi-probe` with a prompt focused on exercising that endpoint.
   - Group captures so one user action covers multiple capabilities when
     possible (e.g. a single sync flow yields both list and download).

   ### R3 — Spec synthesis
   - Iterate `specs/re/backlog.md`. For each row with ≥2 independent evidence
     sources and no spec section yet, dispatch `re-spec` with that capability
     name. Continue until every eligible row has been promoted or explicitly
     deferred (with a reason in the row).

3. **Serial dispatch with budget.** Dispatch acting skills one at a time and
   read their reports before moving on. Cap the run at 10 dispatches per
   invocation to keep context bounded; if more work remains, print a "next
   invocation will continue at …" line and stop.
4. **Aggregate report.** At the end, print:
   - Phase run.
   - Skills dispatched and their outcomes (OK / deferred / failed).
   - Backlog delta: rows added, rows resolved.
   - Files created or updated (paths only, not contents).
   - Next recommended phase to run.
5. **State hygiene.** Ensure `specs/re/backlog.md` rows carry an `updated:`
   date and the phase that last touched them, so the next `re-phase`
   invocation can reason about staleness.

# Outputs

- Whatever the dispatched acting skills produce (captures, notes, spec
  sections, fixtures).
- An updated `specs/re/backlog.md` reflecting the phase's progress.
- A single aggregate report on the terminal.

# Done when

- Every eligible backlog row for the requested phase has been processed
  (resolved, deferred with a reason, or punted to the next invocation with
  a clear pointer).
- The report has been printed.
