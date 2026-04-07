# Wi-Fi Fast Transfer — Plaud Note

*Empty.* To be populated by `re-wifi-probe` (dynamic) and `re-apk-dive`
(static). No public documentation of the hotspot-side protocol has been
found.

Expected content once populated:

- SSID pattern and visibility window.
- DHCP lease, gateway IP, any IPv6 details.
- Open TCP/UDP ports on the gateway.
- mDNS service advertisements.
- HTTP (or custom TCP) endpoint list with request/response shapes.
- Bulk transfer framing, chunking, resume semantics.
- Trigger sequence from BLE (cross-reference into `ble-commands.md`).
