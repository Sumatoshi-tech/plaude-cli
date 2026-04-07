---
name: re-wifi-probe
description: Probe the Plaud "Fast Transfer" Wi-Fi hotspot from a Linux laptop acting as the client. Captures the DHCP lease, the gateway IP, open TCP/UDP ports, mDNS advertisements, and any HTTP endpoints, and produces a pcap plus an annotated report for cross-reference with re-apk-dive.
---

# Role

You are a network protocol analyst. The Plaud device briefly opens an open
Wi-Fi access point to let a phone pull recordings. Your job is to put a laptop
on that hotspot instead of the phone, capture everything that moves, and
document the minimum client implementation required.

# Guardrails

- **Fast Transfer must be triggered from the real Plaud app** on the user's
  phone. Do not attempt to trigger it via BLE writes from the CLI until a
  separate skill has validated the BLE command that does so.
- **Only connect the laptop when the user has explicitly authorised it.**
  Joining the hotspot temporarily takes the laptop off its normal Wi-Fi.
- **Read-only probing.** No brute-forcing, no exploit payloads, no writes to
  endpoints we have not understood.
- **No network egress to the public internet from the laptop while joined.**
  The hotspot is isolated; keep it that way (disable default route if needed).
- **No git operations.**

# Inputs

- Physical access to the Plaud Note and the user's phone with the Plaud app.
- A second Wi-Fi radio is preferred but not required. A single radio is
  acceptable if the user accepts temporary disconnection.
- `specs/re/apk-notes/<version>/wifi-fast-transfer.md` from a prior
  `re-apk-dive` run (optional but strongly recommended).

# Process

1. **Prerequisites.** `nmcli --version`, `dhclient` or `dhcpcd`, `tcpdump`,
   `nmap`, `avahi-browse`, `curl`, `jq`. Report missing ones and stop.
2. **Baseline.** Record the current active Wi-Fi connection name so it can be
   restored: `nmcli -t -f NAME,DEVICE,STATE con show --active`.
3. **Trigger Fast Transfer.** Ask the user to open the Plaud app on their
   phone and start a Fast Transfer. The Plaud hotspot SSID should appear
   (pattern to confirm from apk notes; historically looks like `PLAUD-*`).
4. **Join without default route.** Create a temporary NM profile that does not
   take the default route, to keep the laptop's internet traffic off this
   network:
   ```
   nmcli dev wifi connect '<SSID>' ifname <iface>
   nmcli con mod '<SSID>' ipv4.never-default yes ipv6.never-default yes
   nmcli con up   '<SSID>'
   ```
   Record the DHCP lease (`ip addr`, `ip route`) — the gateway IP is the
   device. Typical vendor patterns: `192.168.4.1`, `192.168.49.1`, `10.10.10.1`.
5. **Start packet capture** on the hotspot interface before any active probing:
   ```
   tcpdump -i <iface> -w specs/re/captures/pcap/YYYY-MM-DD-fast-transfer.pcap \
           -s 0 not port 22
   ```
   Run in the background and record the PID.
6. **mDNS sweep.** `avahi-browse -a -r -t` for 10 seconds. Record any services
   the device advertises (`_http._tcp`, `_plaud._tcp`, etc.) with their port
   and TXT records.
7. **Port scan.** Targeted, polite:
   ```
   nmap -sT -p 1-1024,8000-8100,8080,8443,8888 -Pn -T3 <gateway>
   nmap -sU -p 53,67,68,5353,1900      -Pn -T3 <gateway>
   ```
   Record open ports and service banners. If a web server responds, fetch
   `/`, `/index.html`, `/api`, `/files`, `/list` with `curl -sS -D-` and save
   the full response headers + body.
8. **Let the phone finish its transfer** with capture still running. The
   phone's traffic will be in the pcap alongside anything the laptop did —
   this gives a direct comparison between the real client and our probe.
9. **Stop capture** cleanly (`kill -INT $PID`). Verify the pcap with
   `tcpdump -r … | head`.
10. **Restore networking.** `nmcli con down '<SSID>'; nmcli con up '<baseline>'`.
    Confirm internet reachability.
11. **Analyse.**
    ```
    tshark -r <pcap> -q -z io,phs
    tshark -r <pcap> -q -z conv,tcp
    tshark -r <pcap> -Y 'http' -V > specs/re/captures/pcap/<...>.http.txt
    ```
    Identify every TCP flow between phone (DHCP client) and device (gateway).
12. **Document.** Write `specs/re/captures/pcap/YYYY-MM-DD-fast-transfer.md`:
    - SSID pattern, DHCP lease, gateway IP.
    - mDNS advertisements.
    - Open ports and their protocols.
    - HTTP endpoints observed from the phone, with request/response excerpts.
    - Bulk-transfer shape (one big download? chunked? per-file HTTP GET?).
    - Cross-references to apk-notes classes that build the URLs or parse
      the responses.
13. **Update backlog.** Add Wi-Fi-side capabilities (`list files`, `download
    file`, `device info over wifi`) to `specs/re/backlog.md` with pointers to
    the pcap frames that demonstrate them.
14. **Report.** SSID, gateway, open ports, list of HTTP endpoints, path to
    the pcap and the annotated markdown.

# Outputs

- `specs/re/captures/pcap/<...>.pcap` (gitignored; may contain device serial).
- `specs/re/captures/pcap/<...>.md` — annotated report (committable).
- `specs/re/captures/pcap/<...>.http.txt` — tshark HTTP decode.
- `specs/re/backlog.md` — updated with Wi-Fi capabilities.

# Failure modes and recovery

- **Hotspot does not appear**: confirm the phone actually triggered Fast
  Transfer; try `iwlist <iface> scan` to verify SSID visibility.
- **DHCP hangs**: some vendors hand out leases slowly after association;
  wait up to 30 s before retrying.
- **Laptop loses all Wi-Fi**: the baseline restore step must always run, even
  on failure. Wrap the capture in a shell function so cleanup is guaranteed.

# Done when

- A pcap exists and contains at least one complete phone↔device TCP flow.
- The annotated markdown lists gateway IP, open ports, and every HTTP endpoint
  observed on the phone side.
- The laptop has been restored to its baseline Wi-Fi connection.
