# `plaude-cli sync`

Mirror every recording on a connected Plaud device into a local
directory. Idempotent, resumable, safe by default.

## Synopsis

```
plaude-cli [--backend <sim|ble>] sync <DIR> [--dry-run] [--concurrency N]
```

## Arguments

| Argument | Description |
|---|---|
| `<DIR>` | Destination directory. Created on demand. |
| `--dry-run` | Print the plan without pulling anything. |
| `--concurrency <N>` | Reserved for later milestones. Accepted and parsed, currently ignored; BLE is serial in M9 and eager-prefetch is a hardening concern. |

## State file

Every sync writes a small JSON file to `<DIR>/.plaude-sync.json`:

```json
{
  "version": 1,
  "inventory_hash": "<sha-256 hex>",
  "recordings": {
    "1775393534": {
      "wav_size": 18,
      "asr_size": 18,
      "pulled_at_unix_seconds": 1775393534
    }
  }
}
```

`inventory_hash` is a SHA-256 over the sorted list of
`(id, wav_size, asr_size)` triples the device currently reports. If
it matches the stored value **and** every recorded entry's `.wav`
and `.asr` files are still present at the expected sizes, the run
is a no-op.

## What it does

1. Scan the device for recordings via `list_recordings`.
2. Compare against the state file plus what is already on disk.
3. Pull every recording missing from disk or from the state file.
4. Flag (but never delete) recordings that are in the state file
   but no longer on the device — they are yours to prune.
5. Rewrite the state file.

## Idempotent re-runs

Running `sync` twice in a row with no device changes is a no-op
and prints `nothing to do` on stdout. The second run touches only
reads; the state file is not rewritten unless the plan is non-empty.

## Incremental sync

If a new recording appears on the device, the next `sync` run
pulls only the new one. Existing files are not re-downloaded. The
state file's `inventory_hash` is updated to reflect the new
inventory.

## Resume semantics

If a sync is interrupted (lost connection, `SIGINT`, power loss),
whatever files landed are kept on disk and are not re-downloaded on
the next run. The only casualty is the in-flight file, which gets
re-pulled. This is **file-grained resume**: byte-grained resume
inside a single file requires a trait extension that lands in a
later milestone.

## Deleted on device

If a recording was in the state file but is no longer on the
device, the command prints a single line to stderr:

```
deleted on device (still on disk): 1775393540
```

The local `.wav` / `.asr` files for that recording are **not**
removed — you decide when to prune. The state file entry is
removed so a future re-add of the same id is treated as new.

## Exit codes

| Code | Meaning |
|---|---|
| `0` | Success (including no-op and dry-run). |
| `1` | Runtime error — I/O, transport failure, state file parse error. |
| `2` | Usage error — bad flag value or incompatible state file version. |
| `77` | `EX_NOPERM` — no auth token stored. |
| `78` | `EX_CONFIG` — device rejected the stored token. |

## Examples

```console
$ plaude-cli --backend sim sync /tmp/mirror
pulled 1775393534

$ plaude-cli --backend sim sync /tmp/mirror
nothing to do

$ plaude-cli --backend sim sync /tmp/mirror --dry-run
nothing to do
```

## See also

- [`plaude-cli files list`](files.md) / [`plaude-cli files pull-one`](files.md)
- [`plaude-cli auth`](../../README.md#plaude-cli-auth)
