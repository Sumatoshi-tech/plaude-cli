# Exit codes

plaude-cli follows the `sysexits(3)` convention. Every command maps
to one of the codes below. Wrapper scripts can branch on the numeric
value without parsing stderr.

| Code | Name | Meaning | Common causes |
|------|------|---------|---------------|
| `0` | Success | The command completed normally. | — |
| `1` | Runtime error | An I/O, parse, protocol, or internal failure. | Corrupt state file, bad JSON, unknown recording id. |
| `2` | Usage error | The CLI was invoked incorrectly. | Missing required argument, unknown setting name, invalid value. |
| `69` | `EX_UNAVAILABLE` | The transport layer is unreachable. | BLE backend not wired, device not found, connection dropped, timeout. |
| `77` | `EX_NOPERM` | A command requires an auth token but none is stored. | Run `plaude-cli auth bootstrap` or `plaude-cli auth set-token`. |
| `78` | `EX_CONFIG` | The device rejected the stored auth token. | Token is stale or wrong device. Re-run `plaude-cli auth bootstrap`. |

## Which commands use which codes

| Command | 0 | 1 | 2 | 69 | 77 | 78 |
|---------|---|---|---|----|----|-----|
| `auth set-token` | on success | bad keyring | bad hex input | — | — | — |
| `auth import` | on success | parse failure | missing path | — | — | — |
| `auth show` | on success | — | no token stored | — | — | — |
| `auth clear` | always | — | — | — | — | — |
| `auth bootstrap` | on success | — | wrong backend | BLE not wired | — | — |
| `battery` | on success | — | — | BLE not wired | — | — |
| `device info` | on success | — | — | BLE not wired | no token | rejected |
| `device privacy` | on success | — | bad arg | BLE not wired | no token | rejected |
| `device name` | on success | — | — | BLE not wired | no token | rejected |
| `files list` | on success | — | — | BLE not wired | no token | rejected |
| `files pull-one` | on success | unknown id | — | BLE not wired | no token | rejected |
| `record *` | on success | invalid transition | — | BLE not wired | no token | rejected |
| `settings list` | on success | — | — | BLE not wired | no token | rejected |
| `settings get` | on success | key not found | unknown name | BLE not wired | no token | rejected |
| `settings set` | on success | — | bad name/value | BLE not wired | no token | rejected |
| `sync` | on success | I/O error | — | BLE not wired | no token | rejected |

## Shell integration

```bash
plaude-cli --backend sim sync ~/plaud
case $? in
  0)  echo "sync complete" ;;
  69) echo "device unreachable — is BLE on?" ;;
  77) echo "no token — run: plaude-cli auth bootstrap" ;;
  78) echo "bad token — re-run: plaude-cli auth bootstrap" ;;
  *)  echo "unexpected error ($?)" ;;
esac
```
