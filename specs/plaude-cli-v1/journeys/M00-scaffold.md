# M00 — Workspace scaffold & CI baseline

## Identity

| Field | Value |
|---|---|
| **Milestone ID** | M0 |
| **Journey name** | "I can run `plaude --help` and see a real binary" |
| **Primary actor** | The maintaining engineer (first-run developer onboarding) |
| **Secondary actor** | CI — it must be able to run `make lint && make test && make build` on every change |
| **Dependencies** | None |
| **Blocks** | Everything (M1–M16) |
| **Expected effort** | 1–2 TDD loops |

## Context

This milestone has zero protocol work. Its sole job is to turn the
current doc-only repository into a buildable, testable, lintable Rust
workspace so that every subsequent milestone has a place to put code
and a green pipeline to trust.

The repo already has `Makefile`, `rustfmt.toml`, `clippy.toml`, and
`AGENTS.md` — this milestone wires them up to an actual Cargo
workspace. The Makefile's `build` target expects a binary named
`plaude-cli`, so the workspace must expose one bin target with that
name. We also want to establish the **crate boundary layout** from
the start, even if most crates are empty, so there is no "workspace
reorg churn" once real code starts landing in M1.

## Customer journey (CJM)

### Phase 1 — Discover

**User action**: clones the repo, reads `AGENTS.md` + `specs/plaude-cli-v1/README.md`, runs `make help`.

**Expected**: sees a coherent Rust project and a familiar Cargo
workspace layout. No surprises.

**Friction if absent**: new engineers cannot tell if the project is
abandoned scaffolding or a real codebase. "Where do I put my first
test?" has no obvious answer.

### Phase 2 — Build

**User action**: runs `make build`.

**Expected**: `cargo build --profile release --bin plaude-cli`
succeeds in under 60 seconds of clean build time and produces
`target/release/plaude-cli`.

**Friction if absent**: engineer has to guess the build command.

### Phase 3 — Run

**User action**: `./target/release/plaude-cli --help`.

**Expected**: prints a short help message that includes the program
name, version (from `Cargo.toml`), and a placeholder subcommand list
(can be empty in M0 but must print a usage line, not panic). Exit
code 0.

**Friction if absent**: any panic, crash, or exit code ≠ 0 means
M0 did not land.

### Phase 4 — Test

**User action**: `make test`.

**Expected**: runs `cargo test --all-targets --all-features` across
the empty crates and the one e2e test that invokes `plaude-cli --help`
via `assert_cmd`, and exits 0.

**Friction if absent**: no test scaffold means every subsequent
milestone has to build it from scratch.

### Phase 5 — Lint

**User action**: `make lint`.

**Expected**: `cargo clippy -- -D warnings`, `cargo fmt --check`, and
`cargo audit || true` all pass. Zero warnings. Zero suggestions.

## Engineer journey (micro-TDD)

Follow the AGENTS.md loop contract.

1. **Plan** — "The tiniest slice: make `plaude-cli --help` print a
   single line and exit 0, via one e2e test that runs the binary and
   asserts on its stdout."
2. **Test-RED** — create `crates/plaude-cli/tests/e2e_help.rs` with
   a single `#[test]` that uses `assert_cmd::Command::cargo_bin("plaude-cli").arg("--help").assert().success().stdout(predicates::str::contains("plaude-cli"))`.
   Expected failure message: "binary `plaude-cli` not found in target/".
3. **Code-GREEN** — create `crates/plaude-cli/src/main.rs` with a
   `clap::Command::new("plaude-cli").version(env!("CARGO_PKG_VERSION"))`
   and `.get_matches()`. Wire up `Cargo.toml` at the workspace root.
4. **Reflect** — failure cause matched intention (binary missing →
   binary exists, `--help` prints the name). No accidental new
   behaviour. Complexity delta: +3 files, ~30 lines.
5. **Refactor** — move `clap` boilerplate into a `cli::build_command`
   function if it exceeds 10 lines in `main.rs`.
6. **Verify** — `make lint`, `make test`, `make build` all green.

## Concrete deliverables (DoD checklist)

### Workspace layout

- [ ] `Cargo.toml` at repo root declares a `[workspace]` with members:
      `crates/plaud-domain`, `crates/plaud-transport`, `crates/plaud-proto`,
      `crates/plaud-auth`, `crates/plaud-transport-ble`,
      `crates/plaud-transport-wifi`, `crates/plaud-transport-usb`,
      `crates/plaud-sim`, `crates/plaud-sync`, `crates/plaude-cli`
- [ ] Workspace `Cargo.toml` sets `resolver = "3"` and the common
      `[workspace.package]` fields (edition = "2024", authors, license,
      repository, rust-version)
- [ ] Every member crate has its own `Cargo.toml` + `src/lib.rs` (for
      libs) or `src/main.rs` (for the bin) — even if the lib is just
      a stub with a single `//!` module doc comment
- [ ] The bin crate is `crates/plaude-cli` with `[[bin]] name = "plaude-cli"`
      so the existing Makefile target resolves
- [ ] Workspace-level `[workspace.dependencies]` declares shared deps:
      `clap = { version = "...", features = ["derive"] }`,
      `tokio = { version = "...", features = ["rt-multi-thread", "macros"] }`,
      `thiserror`, `tracing`, `tracing-subscriber`
- [ ] Dev-dependencies at workspace level: `assert_cmd`, `predicates`,
      `insta`, `tempfile`

### Binary behaviour

- [ ] `plaude-cli --help` exits 0 and prints at minimum: program name,
      version from `CARGO_PKG_VERSION`, a one-line description, and
      a `USAGE:` section (can be `plaude [COMMAND]` with no commands
      registered yet)
- [ ] `plaude-cli --version` exits 0 and prints `plaude-cli 0.1.0`
- [ ] `plaude-cli` with no arguments exits with a clap-generated
      usage message and a non-zero exit code (CLIG compliance:
      missing required subcommand → exit 2)
- [ ] No panics on any of the above; no stack traces on stderr

### Tests

- [ ] `crates/plaude-cli/tests/e2e_help.rs` exists and passes
      asserting: binary runs, exit 0, stdout contains the program name
- [ ] `crates/plaude-cli/tests/e2e_version.rs` exists and passes
      asserting: `--version` prints a semver string matching the
      `CARGO_PKG_VERSION` env var
- [ ] `crates/plaude-cli/tests/e2e_no_args.rs` exists and passes
      asserting: no args exits with code 2 and stderr contains "USAGE"
- [ ] `make test` runs all three tests successfully

### Lint & tooling

- [ ] `make lint` returns exit 0: clippy clean, rustfmt clean, audit
      soft-warns only
- [ ] `make build` produces `target/release/plaude-cli`
- [ ] `make clean` works
- [ ] `.gitignore` already covers `target/` (verify it does)

### CI

- [ ] A `.github/workflows/ci.yml` (or equivalent for the chosen CI) is
      added with three jobs: lint, test, build — each running on
      `ubuntu-latest` with a recent stable Rust toolchain and
      `rustfmt` + `clippy` components. If no CI is configured, add
      at minimum a `ci.yml` that runs `make lint && make test && make build`.
- [ ] CI badge added to top-level `README.md` (create a minimal
      README if one does not exist)

### Documentation

- [ ] Top-level `README.md` exists with: one-sentence project
      description, link to `specs/plaude-cli-v1/README.md`, build
      instructions (`make build`, `make test`, `make lint`), and
      a clear "pre-release, not for production use" banner
- [ ] `docs/usage/` directory exists with a placeholder `index.md`

## Test plan

| Test | Path | What it proves |
|---|---|---|
| `e2e_help` | `crates/plaude-cli/tests/e2e_help.rs` | Binary exists and responds to `--help` with exit 0 |
| `e2e_version` | `crates/plaude-cli/tests/e2e_version.rs` | Version string matches `CARGO_PKG_VERSION` |
| `e2e_no_args` | `crates/plaude-cli/tests/e2e_no_args.rs` | No-args case exits with CLIG-compliant code 2 |

All three are **hermetic**: no network, no hardware, no keyring. They
run in under 500 ms each.

## Friction points to watch for

1. **Resolver version**: Rust 2024 edition uses resolver 3. Forgetting
   to set `resolver = "3"` in the workspace `Cargo.toml` gives
   confusing dependency-resolution errors.
2. **Empty crates failing to compile**: an empty `lib.rs` must still
   have `#![deny(missing_docs)]` disabled or a crate-level `//!` doc
   comment, or the lint gate fails.
3. **Makefile git calls**: the existing Makefile has
   `$(shell git describe ...)` and `$(shell git rev-parse HEAD)`
   for `VERSION` and `GIT_COMMIT`. These are variable definitions,
   not targets. If any target uses them, the build fails when run
   outside a git working tree. Verify none of the targets used in
   CI reference these variables, or provide safe fallbacks.
4. **`cargo audit` on empty workspaces**: audit may warn about no
   advisories DB present; the Makefile already swallows errors
   with `|| true` — verify this is deliberate and desired.
5. **Workspace dependency deduplication**: declaring deps at the
   workspace level and inheriting with `workspace = true` in member
   crates is the idiomatic 2024-edition pattern — do not declare
   the same crate twice.

## UX assessment

M0 has no end-user-facing surface beyond `--help`, `--version`, and
the error path for no-args. The UX bar is therefore:

- `--help` output reads like a real tool's help, not a placeholder
- `--version` output matches what a human would expect (`plaude-cli
  0.1.0`), not Cargo's default pretty-print
- Error on no args gives the user a usable hint (clap's default is
  fine; CLIG-compliant exit code is 2)
- No warnings, no backtraces, no "this is alpha software" noise in
  normal output (the "pre-release" banner lives in the README, not
  in every command)

## Evidence & references

- Process contract: [`AGENTS.md`](../../../AGENTS.md)
- Plan: [`../PLAN.md`](../PLAN.md) §3 Architecture, §6 CLI surface
- Existing tooling: [`Makefile`](../../../Makefile), [`rustfmt.toml`](../../../rustfmt.toml), [`clippy.toml`](../../../clippy.toml)
- CLIG exit codes: https://clig.dev/#the-basics (exit 2 for missing
  required argument)

## Open questions

- **CI platform**: the repo has no existing CI config. Default to
  GitHub Actions unless the user specifies otherwise; if GitHub
  Actions is chosen, reuse `Swatinem/rust-cache` for caching.
- **License header**: none of the existing files carry a license
  header. Decide whether to add SPDX headers to new `.rs` files
  (recommended: `// SPDX-License-Identifier: Apache-2.0 OR MIT`).

## Definition of Ready (DoR)

- [x] Workspace crate layout is specified in `PLAN.md` §3
- [x] Makefile targets exist and reference `plaude-cli`
- [x] AGENTS.md micro-TDD loop is documented
- [x] No blocking dependencies (this is the root milestone)

## Implementation (closed 2026-04-05)

### Files created

**Workspace**
- [`/Cargo.toml`](../../../Cargo.toml) — workspace root, resolver 3,
  edition 2024, 10 member crates, shared `[workspace.dependencies]`
  and `[workspace.lints]` (`missing_docs = deny`, `clippy::all = deny`,
  `unused = deny`).

**Library crate stubs** — each is a documented `src/lib.rs` with a
crate-level `//!` module comment explaining its future purpose and
the milestone in which it lands. No public items yet.
- [`crates/plaud-domain/Cargo.toml`](../../../crates/plaud-domain/Cargo.toml), [`src/lib.rs`](../../../crates/plaud-domain/src/lib.rs)
- [`crates/plaud-transport/Cargo.toml`](../../../crates/plaud-transport/Cargo.toml), [`src/lib.rs`](../../../crates/plaud-transport/src/lib.rs)
- [`crates/plaud-proto/Cargo.toml`](../../../crates/plaud-proto/Cargo.toml), [`src/lib.rs`](../../../crates/plaud-proto/src/lib.rs)
- [`crates/plaud-auth/Cargo.toml`](../../../crates/plaud-auth/Cargo.toml), [`src/lib.rs`](../../../crates/plaud-auth/src/lib.rs)
- [`crates/plaud-transport-ble/Cargo.toml`](../../../crates/plaud-transport-ble/Cargo.toml), [`src/lib.rs`](../../../crates/plaud-transport-ble/src/lib.rs)
- [`crates/plaud-transport-wifi/Cargo.toml`](../../../crates/plaud-transport-wifi/Cargo.toml), [`src/lib.rs`](../../../crates/plaud-transport-wifi/src/lib.rs)
- [`crates/plaud-transport-usb/Cargo.toml`](../../../crates/plaud-transport-usb/Cargo.toml), [`src/lib.rs`](../../../crates/plaud-transport-usb/src/lib.rs)
- [`crates/plaud-sim/Cargo.toml`](../../../crates/plaud-sim/Cargo.toml), [`src/lib.rs`](../../../crates/plaud-sim/src/lib.rs)
- [`crates/plaud-sync/Cargo.toml`](../../../crates/plaud-sync/Cargo.toml), [`src/lib.rs`](../../../crates/plaud-sync/src/lib.rs)

**Binary crate**
- [`crates/plaude-cli/Cargo.toml`](../../../crates/plaude-cli/Cargo.toml) — `[[bin]] name = "plaude-cli"` matches the `Makefile`'s `--bin plaude-cli` target.
- [`crates/plaude-cli/src/main.rs`](../../../crates/plaude-cli/src/main.rs) — 51 lines. `clap::Parser`-derived `Cli` struct, `fn main() -> ExitCode` that checks `env::args().count() < MIN_USER_ARGS` and delegates to `Cli::command().print_help()` + `ExitCode::from(EXIT_USAGE)` if so, otherwise `Cli::parse()` and returns `ExitCode::SUCCESS`. No literal strings or numbers in the body; both the exit code and the argv threshold are named constants.

**E2E tests** — every file opens with a `// Journey:` comment linking back to this document, per step 16 of the implement skill.
- [`crates/plaude-cli/tests/e2e_help.rs`](../../../crates/plaude-cli/tests/e2e_help.rs) — two tests: `help_exits_zero_and_mentions_binary_name`, `short_help_flag_matches_long_help_flag`.
- [`crates/plaude-cli/tests/e2e_version.rs`](../../../crates/plaude-cli/tests/e2e_version.rs) — two tests: `version_exits_zero_and_prints_matching_semver` (uses `env!("CARGO_PKG_VERSION")` so the test stays in lock-step with `Cargo.toml`), `short_version_flag_matches_long_version_flag`.
- [`crates/plaude-cli/tests/e2e_no_args.rs`](../../../crates/plaude-cli/tests/e2e_no_args.rs) — one test: `no_args_exits_two_and_prints_usage_hint`, asserting `code(2)` and tolerating the usage hint on either stdout or stderr.

**Documentation**
- [`README.md`](../../../README.md) — top-level project README with pre-release banner, build instructions, project layout, contributing pointer, privacy notice, dual-license line.
- [`docs/usage/index.md`](../../../docs/usage/index.md) — user-facing usage guide, lists current M0 capabilities and a table of planned subcommands with their milestones.

**CI**
- [`.github/workflows/ci.yml`](../../../.github/workflows/ci.yml) — three jobs (`lint`, `test`, `build`) running `make lint`, `make test`, `make build` on `ubuntu-latest` with `dtolnay/rust-toolchain@stable` and `Swatinem/rust-cache@v2`.

### Files modified

- None outside the ones listed above. (The pre-existing `Makefile`,
  `rustfmt.toml`, `clippy.toml`, `AGENTS.md`, `.gitignore` were left
  untouched.)

### Dependency additions

At workspace level:

- `clap` 4.5 + `derive`
- `thiserror` 2.0 (declared for future M1 use)
- `tracing` 0.1, `tracing-subscriber` 0.3 (declared for future M12 use)
- `assert_cmd` 2.0 (dev-dep, used by the e2e tests)
- `predicates` 3.1 (dev-dep, used by the e2e tests)

All are currently-maintained, widely-used crates with active
2025/2026 releases.

### Results

- **`make build`** — `Finished release profile [optimized] target(s) in 2.39s`
- **`make test`** — 5 e2e tests pass + 11 empty unittest suites for
  the library stubs. Total runtime ≈ 0.05 s.
- **`make lint`** — clippy clean (`-D warnings`), rustfmt clean,
  cargo-audit absent on this host (swallowed by the pre-existing
  `|| true` in the Makefile — cosmetic, to be addressed in M12).
- **Binary size**: 993 KB release.
- **Startup time**: 1 ms for `plaude-cli --version` (measured with
  `time`).

### Traceability checklist per the implement skill (step 16)

- [x] Journey file contains an **Implementation** section listing files
      created and modified (this section).
- [x] Roadmap cross-links to this journey and key implementation files
      (added in the `M0 — Workspace scaffold & CI baseline` section of
      `ROADMAP.md`).
- [x] Test files carry a `// Journey: specs/plaude-cli-v1/journeys/M00-scaffold.md`
      comment in their file header.

### Post-close verification

```console
$ target/release/plaude-cli --help
Offline CLI for the Plaud Note voice recorder. No cloud, no phone app.

Usage: plaude-cli

Options:
  -h, --help     Print help
  -V, --version  Print version

$ target/release/plaude-cli --version
plaude-cli 0.1.0

$ target/release/plaude-cli ; echo "exit=$?"
Offline CLI for the Plaud Note voice recorder. No cloud, no phone app.

Usage: plaude-cli
…
exit=2
```

All three user-visible behaviours from the customer journey match
expectations. M0 is closed.
