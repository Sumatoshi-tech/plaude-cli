# `plaude-cli record` — remote recording pipeline control

## Overview

The `record` subcommand tree lets you start, stop, pause, and resume
a recording on the connected Plaud device from the command line. Each
action maps directly to the corresponding vendor BLE opcode.

All recording commands require an authenticated transport (a stored
token). If no token is stored, the command exits with code 77
(`EX_NOPERM`). If the device rejects the token, exit code 78
(`EX_CONFIG`).

## Commands

### `record start`

Start a new recording. Fails if a recording is already in progress.

```
$ plaude-cli --backend sim record start
recording started
```

### `record stop`

Stop the current recording and finalise the `.WAV`/`.ASR` pair. Fails
if no recording is in progress.

```
$ plaude-cli --backend sim record stop
recording stopped
```

### `record pause`

Pause the current recording. Fails if not currently recording.

```
$ plaude-cli --backend sim record pause
recording paused
```

### `record resume`

Resume a paused recording. Fails if the recording is not paused.

```
$ plaude-cli --backend sim record resume
recording resumed
```

## State machine

The device enforces a simple state machine:

```
         start          pause
Idle ──────────► Recording ──────► Paused
  ▲                │                  │
  │    stop        │       stop       │
  └────────────────┘                  │
  │                                   │
  └───────────────────────────────────┘
                resume (back to Recording)
```

Invalid transitions (e.g. `stop` when idle, `pause` when paused)
return a protocol error and exit with code 1.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Runtime/protocol error (invalid state transition) |
| 77 | No auth token stored |
| 78 | Device rejected the stored token |
