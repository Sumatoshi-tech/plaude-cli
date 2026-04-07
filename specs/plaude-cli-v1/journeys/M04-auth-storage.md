# M04 — `plaud-auth` storage, import, show, clear

## Identity

| Field | Value |
|---|---|
| **Milestone ID** | M4 |
| **Journey name** | "I can run `plaude auth import ~/my.btsnoop.log` once and then `plaude auth show` confirms my device credential is cached, with zero cloud and zero leakage of the raw token" |
| **Primary actor** | End user doing one-time CLI onboarding |
| **Dependencies** | M1 (AuthStore trait), M3 (test sim not strictly needed but referenced) |
| **Blocks** | M5 (BLE transport needs credential store), M8 (bootstrap needs `ChainStore`) |
| **DoD source** | `specs/plaude-cli-v1/ROADMAP.md#m4` |

## Context

M1 defined the `AuthStore` trait. M4 provides three concrete
implementations — `KeyringStore`, `FileStore`, `ChainStore` — plus a
pure-Rust btsnoop parser that extracts a V0095-style auth token from
an Android HCI snoop log, plus the CLI subcommand tree (`plaude auth
set-token|import|show|clear`) that wires those pieces to user-visible
UX.

The live-tested evidence that **replaying a captured token is
sufficient for full BLE access** is the whole justification for this
milestone — see
[`specs/re/captures/ble-live-tests/2026-04-05-token-validation.md`](../../specs/re/captures/ble-live-tests/2026-04-05-token-validation.md).
The Python reference implementation of the btsnoop-to-token extractor
lives at
[`specs/re/captures/ble-live-tests/scripts/plaud-test2c.py`](../../specs/re/captures/ble-live-tests/scripts/plaud-test2c.py);
the Rust parser in this milestone matches its byte-extraction logic.

## Customer journey (CJM)

### Phase 1 — User onboarding

**User action**:

```
$ plaude auth import ./my-btsnoop.log
Token imported. Fingerprint: a82dcb11ff56d11d
```

**Expected**: the CLI parses the btsnoop log, extracts the first auth
frame written to the vendor write characteristic (`handle=0x000D`,
opcode `0x52`, value starting with `01 01 00 02 00 00`), constructs
an `AuthToken` via `plaud_domain::AuthToken::new`, stores it via
`ChainStore::put_token`, and prints the SHA-256-truncated-to-16-char
fingerprint. **The raw token is never printed or written to a log.**

### Phase 2 — User verification

**User action**:

```
$ plaude auth show
Token stored.
Fingerprint: a82dcb11ff56d11d
Backend: file (~/.config/plaude/token)
```

**Expected**: the CLI reads from the `ChainStore` (keyring first,
file fallback), computes the fingerprint, and tells the user which
backend the token came from. Absent → exit 2 with a "no token
stored" message pointing at `plaude auth import`.

### Phase 3 — User manual paste

**User action**:

```
$ plaude auth set-token b4b48c21074f89d287c01e9f4b1ffab7
Token stored. Fingerprint: a82dcb11ff56d11d
```

**Expected**: validates the input is 16 or 32 ASCII hex chars via
`AuthToken::new`, stores it, prints the fingerprint.

### Phase 4 — User cleanup

**User action**:

```
$ plaude auth clear
Token removed.
```

**Expected**: removes from **both** backends. Idempotent — a second
call returns success even if nothing was present.

### Phase 5 — Engineer runs e2e tests against a tempdir-sandboxed HOME

**Action**: the test passes `--config-dir <tempdir>` to the CLI and
runs `auth set-token / show / clear` against that sandbox, never
touching the real `~/.config/plaude/token`.

**Expected**: hermetic, deterministic, zero filesystem collision
with the user's real config.

## Scope

**In scope:**

- `plaud_auth::FileStore` — file-backed `AuthStore` at
  `~/.config/plaude/token` with mode `0600` on Unix (default perms on
  Windows).
- `plaud_auth::KeyringStore` — wraps the `keyring` crate; uses
  `tokio::task::spawn_blocking` because keyring operations are
  synchronous.
- `plaud_auth::ChainStore` — primary + secondary `AuthStore` pair.
  `get_token` tries primary first, falls back to secondary. `put_token`
  writes to primary, falls back to secondary on error.
  `remove_token` tries both and returns success if either succeeded.
- `plaud_auth::fingerprint::token_fingerprint` — SHA-256 of the
  token's UTF-8 bytes, first 16 hex chars.
- `plaud_auth::btsnoop::extract_auth_token` — pure-Rust btsnoop log
  parser that walks records looking for the first ATT Write Command
  to handle `0x000D` whose value starts with the V0095 auth prefix.
- `plaud_auth::default_store` — convenience constructor returning a
  `ChainStore` with keyring primary + file secondary, using
  `~/.config/plaude/token` for the file path.
- `plaude-cli` binary: new `auth` subcommand tree with four
  subcommands and a `--config-dir` global override for tests.

**Out of scope (deferred):**

- ACL fragmentation in the btsnoop parser. Our V0095 captures come
  from Samsung Android phones which negotiate MTU ≥ 247, so the
  38-byte auth frame fits in a single ACL packet. A capture from an
  MTU-23 phone would fragment and our parser will return
  `NoAuthFrameFound` rather than reassembling. This is acceptable
  for M4 and can be revisited if evidence demands it.
- Multi-device keying. The `AuthStore` trait takes a `device_id`
  parameter, but M4 ignores it in favour of a single constant key
  (`"default"`). Multi-device support will arrive when M7 adds
  per-device connect flows.

## Test plan

| Path | Scope | Coverage |
|---|---|---|
| `tests/fingerprint.rs` | unit | SHA-256 → 16-char fingerprint for a known token string; two different tokens produce different fingerprints |
| `tests/file_store.rs` | integration | put → get round-trip in a tempdir; get on missing file returns `None`; remove on missing is idempotent; put creates parent dir with 0700; put writes file with 0600 (Unix); put overwrites an existing file |
| `tests/chain_store.rs` | integration | primary Ok(Some) returns without touching secondary; primary Ok(None) falls through to secondary; primary error falls through; put fallback on primary error; remove succeeds if either backend succeeds |
| `tests/btsnoop_parser.rs` | integration | synthetic log with a crafted ATT write to handle `0x000D` returns the embedded token; log without any `0x52 handle 0x000D` write returns `NoAuthFrameFound`; invalid header → `InvalidMagic`; a truncated log returns a specific error |
| `tests/cli_auth.rs` | e2e (new) | `plaude auth set-token <hex> --config-dir <tmp>`; `plaude auth show --config-dir <tmp>` outputs the fingerprint; `plaude auth clear --config-dir <tmp>`; invalid token rejected with exit code 2 |

Target coverage for `plaud-auth`: ≥ 90 %.

## Definition of Ready

- [x] M1 closed; `AuthStore` trait exists
- [x] `plaud_domain::AuthToken` with hex-length validation
- [x] Evidence for the V0095 auth-frame byte layout in
      `specs/re/captures/btsnoop/2026-04-05-plaud-0day-pair.md`

## Definition of Done

Mirror of the M4 DoD in `specs/plaude-cli-v1/ROADMAP.md`. Updated at
milestone close with evidence links.

## Implementation (closed 2026-04-06)

### Files created — `plaud-auth` library crate

- [`Cargo.toml`](../../../crates/plaud-auth/Cargo.toml) — deps: `plaud-domain`, `plaud-transport`, `thiserror`, `async-trait`, `tokio`, `keyring`, `sha2`, `dirs`. `keyring-tests` feature gates the keyring integration tests.
- [`src/lib.rs`](../../../crates/plaud-auth/src/lib.rs) — module tree, re-exports, `default_store` convenience constructor.
- [`src/constants.rs`](../../../crates/plaud-auth/src/constants.rs) — every magic value (btsnoop format, HCI/L2CAP/ATT offsets, auth frame prefix, file permission modes) as a named `const`.
- [`src/fingerprint.rs`](../../../crates/plaud-auth/src/fingerprint.rs) — `token_fingerprint(&AuthToken)` returning `sha256(token)[:16]` hex.
- [`src/btsnoop.rs`](../../../crates/plaud-auth/src/btsnoop.rs) — pure-Rust parser that walks a btsnoop v1 log, finds the first ATT Write Command to handle `0x000D` with the V0095 auth prefix, and returns the embedded token.
- [`src/file_store.rs`](../../../crates/plaud-auth/src/file_store.rs) — `FileStore` implementing `AuthStore` against a caller-chosen path. Unix permissions `0600` on the file and `0700` on the parent directory via `tokio::task::spawn_blocking`.
- [`src/keyring_store.rs`](../../../crates/plaud-auth/src/keyring_store.rs) — `KeyringStore` wrapping the `keyring` crate with `spawn_blocking` for async safety.
- [`src/chain_store.rs`](../../../crates/plaud-auth/src/chain_store.rs) — `ChainStore` composing two `AuthStore`s in a primary-then-secondary fallback arrangement.

### Files created — `plaude-cli` binary

- [`src/main.rs`](../../../crates/plaude-cli/src/main.rs) — reworked to own the top-level `Cli` struct with `--config-dir` global and `DispatchError` mapping Usage vs Runtime exit codes.
- [`src/commands/mod.rs`](../../../crates/plaude-cli/src/commands/mod.rs) — subcommand module root.
- [`src/commands/auth.rs`](../../../crates/plaude-cli/src/commands/auth.rs) — `auth set-token / import / show / clear` handlers with sandbox-vs-production store construction via `build_store(config_dir)`.

### Test files created — 5 integration test files, 28 new tests

Every file opens with `// Journey: specs/plaude-cli-v1/journeys/M04-auth-storage.md`.

- [`plaud-auth/tests/fingerprint.rs`](../../../crates/plaud-auth/tests/fingerprint.rs) — 4 tests: length, determinism, inequality for different tokens, non-leakage of token substring
- [`plaud-auth/tests/file_store.rs`](../../../crates/plaud-auth/tests/file_store.rs) — 8 tests: get-empty returns None, put/get round-trip, parent dir creation, overwrite, remove idempotency, remove after put, `0600`/`0700` Unix permissions
- [`plaud-auth/tests/chain_store.rs`](../../../crates/plaud-auth/tests/chain_store.rs) — 4 tests: put/get via primary, fallback to secondary on primary miss, remove clears both, remove idempotency
- [`plaud-auth/tests/btsnoop_parser.rs`](../../../crates/plaud-auth/tests/btsnoop_parser.rs) — 6 tests: happy path, invalid magic, unsupported version, truncated header, no auth frame present, truncated record
- [`plaude-cli/tests/e2e_auth.rs`](../../../crates/plaude-cli/tests/e2e_auth.rs) — 6 e2e tests against the real binary in a sandboxed tempdir: set-token happy path, no-raw-token-in-output, show on empty errors with code 2, clear idempotent, invalid-token rejected with code 2, set→clear→show round-trip

### Files modified

- [`Cargo.toml`](../../../Cargo.toml) (workspace root) — added `keyring`, `sha2`, `dirs`, `tempfile` to `[workspace.dependencies]`; added `rt-multi-thread` and `fs` features to `tokio`.
- [`crates/plaude-cli/Cargo.toml`](../../../crates/plaude-cli/Cargo.toml) — pulled in `plaud-auth`, `plaud-domain`, `plaud-transport`, `tokio`, `tempfile`.
- [`docs/usage/index.md`](../../../docs/usage/index.md) — updated the current-status and planned-subcommand tables to reflect the shipped `auth` subcommands, plus a new "Token storage model" and "Privacy guarantees of `auth show`" section.

### Results

- **`make lint`** — clippy `-D warnings` clean, rustfmt clean.
- **`make test`** — **198 tests passed, 0 failed** (170 from M0–M3 + 28 new from M4).
- **`make build`** — release binary rebuilt in 2.82 s.
- **Live integration**: `target/release/plaude-cli auth show` against the real pre-seeded `~/.config/plaude/token` from the research phase returns the exact fingerprint `a82dcb11ff56d11d` documented in `specs/re/captures/ble-live-tests/2026-04-05-token-validation.md`. Full production round-trip verified.

### Cross-cutting concerns

- [x] Every public item has a doc comment.
- [x] No `#[allow(...)]` attributes anywhere.
- [x] No `unwrap()`/`expect()` in production code paths. The `write!`-into-`String` in `fingerprint.rs` uses `let _ = ...` because `fmt::Write for String` is infallible.
- [x] No bare literals in business logic.
- [x] Keyring tests gated behind the `keyring-tests` cargo feature to keep CI hermetic.
- [x] Raw token never appears in stdout/stderr from any `auth` subcommand — asserted by `auth_set_token_does_not_print_the_raw_token`.

### Traceability

- [x] This journey has an Implementation section (you're reading it).
- [x] [`ROADMAP.md`](../ROADMAP.md) M4 row is ✅ CLOSED with file links.
- [x] Every test file header begins with `// Journey: specs/plaude-cli-v1/journeys/M04-auth-storage.md`.
