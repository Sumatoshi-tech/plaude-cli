# plaude-cli v1 specification

This folder is the canonical planning surface for **plaude-cli v1**, an
offline, device-direct CLI for the Plaud Note that talks to hardware
the user owns without any ongoing plaud.ai cloud contact or phone-app
dependency.

## Contents

| File | Purpose |
|---|---|
| [`PLAN.md`](PLAN.md) | The technical specification. Architecture, transports, security model, CLI surface, stressors, working agreements. Start here if you want to understand **what** we are building and **why**. |
| [`ROADMAP.md`](ROADMAP.md) | The implementation roadmap. Master milestone checklist with DoR/DoD per milestone, dependency graph, cross-cutting concerns, changelog. Start here if you want to know **what to build next**. |
| [`journeys/`](journeys/) | One CJM-style journey document per roadmap milestone. Each journey describes the shipped user experience, the TDD implementation flow, friction points, UX assessment, test plan, and evidence references. Open the relevant journey file when **starting work on a milestone**. |

## Reading order

1. `PLAN.md` §§ 1–4 (scope, device, transports, auth) — the problem statement.
2. `ROADMAP.md` — the ordered plan and current status.
3. The specific `journeys/Mxx-*.md` you are about to execute.
4. The supporting evidence under `../../specs/re/` that the journey cites.

## Working contract

- **AGENTS.md is the process contract.** Every milestone follows micro-TDD:
  one failing test → minimal code → reflect → refactor → verify → commit.
  Every claim in `docs/protocol/` cites evidence. Every public item has
  a doc comment. `make lint` is the gate.
- **No git operations are performed by the agent.** The human runs `git`.
- **Evidence is sacred.** The tree under `specs/re/captures/` (gitignored
  raw files and committable annotated analyses) is the provenance layer
  for the protocol spec. If a fact is not in evidence, it is not in the
  spec.
- **`plaud-sim` is the CI north star.** No milestone depends on physical
  hardware for its e2e tests; every transport has a simulated counterpart.
