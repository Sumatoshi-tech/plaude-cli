# Authentication

Every command that talks to the device (except `battery`) needs an auth token.
The token is a 32-character hex string, unique to your device, captured once and stored permanently.

## Setting a token manually

If you already have your token:

```bash
plaude auth set-token abcdef0123456789abcdef0123456789
```

## Importing from a Bluetooth capture

If you have an Android HCI snoop log containing a Plaud sync session:

```bash
plaude auth import /path/to/btsnoop_hci.log
```

## Checking your token

```bash
plaude auth show
# Token stored.
# Fingerprint: a82dcb11ff56d11d
```

The fingerprint is a SHA-256 hash — the raw token is never printed.

## Clearing the token

```bash
plaude auth clear
```

## Bootstrap — automatic token capture {#bootstrap}

The bootstrap command makes your computer pretend to be a Plaud device.
When the phone app connects and sends the token, we capture it.

### Prerequisites

- Your **real** Plaud device must be powered off or out of range
- The Plaud phone app must be installed on your phone
- Bluetooth must be enabled on both devices

### Steps

1. Turn off your real Plaud device (or move it far away)
2. Run the bootstrap:
   ```bash
   plaude auth bootstrap
   ```
3. Open the Plaud app on your phone
4. The app will find "PLAUD_NOTE" (your computer) and connect
5. The token is captured and stored automatically:
   ```
   Advertising as PLAUD_NOTE — open the Plaud app and pair with this device...
   Token captured. Fingerprint: a82dcb11ff56d11d
   ```
6. Turn your real device back on

### Timeout

Default wait is 120 seconds. Override with:

```bash
plaude auth bootstrap --timeout 60
```

## Token storage

Tokens are stored in two places (first available wins):

1. **OS keyring** — Linux Secret Service, macOS Keychain, Windows Credential Manager
2. **File fallback** — `~/.config/plaude/token` (mode 0600)

Override the file location with `--config-dir`:

```bash
plaude --config-dir /tmp/test auth show
```
