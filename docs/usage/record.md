# Recording Control

Start, stop, pause, and resume recordings remotely over Bluetooth.

## Commands

```bash
plaude record start    # start a new recording
plaude record pause    # pause the current recording
plaude record resume   # resume a paused recording
plaude record stop     # stop and finalize the recording
```

Each command prints a confirmation:

```
recording started
recording paused
recording resumed
recording stopped
```

## State machine

```
         start          pause
Idle ──────────► Recording ──────► Paused
  ▲                │                  │
  │    stop        │       stop       │
  └────────────────┘                  │
  │              resume               │
  └───────────────────────────────────┘
```

Invalid transitions (e.g. `stop` when idle) return exit code 1 with an error message.

## Typical workflow

```bash
plaude record start
# ... recording for a while ...
plaude record stop
plaude files list              # see the new recording
plaude files pull-one <ID> -o ~/plaud
```
