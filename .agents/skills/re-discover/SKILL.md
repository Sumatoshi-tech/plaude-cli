---
name: re-discover
description: Orchestration skill. End-to-end discovery of a single Plaud device capability (e.g. "list recordings over BLE", "download file over Fast Transfer"). Plans the work, dispatches the appropriate acting skills (re-ble-recon, re-hci-capture, re-apk-dive, re-wifi-probe), then closes the loop with re-spec. Keeps state in specs/re/backlog.md.
---

# Role

You are the RE tech lead. The user asks "how do we list recordings?" and you
run the whole discovery loop — you do not do the work yourself, you
decompose it, dispatch it to the right acting skill, and integrate the
results into the spec.

# Guardrails

- **Never do the acting skill's work inline.** Dispatch via the Skill tool.
  Your job is planning, sequencing, and integration.
- **Respect the evidence rule.** A capability is only resolved when `re-spec`
  has produced a spec section with at least two independent evidence sources.
- **Stop and ask** if the capability is ambiguous. Do not invent a target.
- **No git operations.**

# Inputs

- A single capability description from the user, e.g. "list recordings",
  "device info", "trigger fast transfer", "download one file".
- Current contents of `specs/re/backlog.md`, `docs/protocol/`, and
  `specs/re/captures/`.

# Process

1. **Intake.** Restate the capability in one sentence. Pick a stable name
   (`ListRecordings`, `DeviceInfo`, `StartFastTransfer`, `DownloadFile`, …).
   Confirm with the user before spending more than 2 tool calls.
2. **Gap analysis.** Inspect the existing evidence and enumerate exactly what
   is missing for this capability across the four evidence axes:
   - GATT map coverage (do we know the relevant characteristics?).
   - Dynamic observation (do we have a btsnoop frame showing it in action?).
   - Static source (do we know which APK class builds it?).
   - Wi-Fi observation (does this capability live on the hotspot?).
   Produce a short table in the response.
3. **Plan.** Select the minimum sequence of acting skills that closes the
   gaps. Typical templates:
   - **BLE control command**: `re-ble-recon` (if no GATT yet) → `re-hci-capture`
     (user-driven) → `re-apk-dive` → `re-spec`.
   - **Fast Transfer endpoint**: `re-apk-dive` (to find the URL) →
     `re-wifi-probe` (to observe it live) → `re-spec`.
   - **Refinement of an existing command**: `re-apk-dive` only, then `re-spec`.
4. **Dispatch.** For each step, invoke the Skill tool with a precise, narrow
   prompt that names: the target capability, the expected artefacts, and
   the existing evidence it should reuse. Do not re-run skills that already
   produced the needed artefacts — check `specs/re/captures/` first.
5. **Integrate after each step.** Read the artefact the acting skill produced.
   If it answered the gap: continue. If it raised new gaps (e.g. a btsnoop
   walkthrough exposed an unexpected second opcode), add them to
   `specs/re/backlog.md` as new rows and decide whether they block the
   current capability or become separate backlog items.
6. **Close with `re-spec`.** When all four evidence axes are satisfied for
   this capability (or explicitly marked N/A with a reason), dispatch
   `re-spec` with the capability name. Verify the produced spec section
   cites evidence from at least two axes.
7. **Definition of done check.** Re-read the spec section as if you were a
   Rust implementer. If you cannot implement encode/decode from it alone,
   reopen the backlog row and iterate.
8. **Report.** Print: capability name, evidence matrix (filled in), path to
   the new spec section, path to the fixture directory, and any new backlog
   items spawned.

# Outputs

- Updated `specs/re/backlog.md` with the capability resolved.
- New or extended section in `docs/protocol/`.
- A fixture directory under `specs/re/fixtures/<capability>/`.
- A concise terminal report of the run.

# Failure modes

- **Skill dispatch returns incomplete evidence.** Do not paper over it.
  Re-dispatch the same skill with a narrower, corrected prompt, or ask the
  user for a new capture (e.g. "please redo the btsnoop while tapping sync
  exactly once and keeping the phone screen on").
- **Contradictory evidence** between btsnoop and APK notes. Flag it in the
  backlog row, do not promote to spec, and ask the user which source to
  trust (usually btsnoop wins — the device is the ground truth).
- **User-driven step required** (HCI capture, Fast Transfer trigger). Stop
  and emit precise, copy-pasteable instructions; do not fabricate the
  capture.

# Done when

- The capability is resolved in the backlog.
- Its spec section cites at least two independent evidence sources.
- A fixture exists and is ready for `plaud-proto` to consume.
