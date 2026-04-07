"""Test 2b: post-fake-auth command probe.

Send a fabricated auth frame, then fire several harmless read-style
commands (GetDeviceName nullary, GetState nullary) and a standard SIG
battery level read. Observe which (if any) come back with real data
versus errors. If any command returns meaningful payload, device-level
token validation is weak and we can build an offline CLI against it.
"""
from __future__ import annotations

import asyncio
import sys
import time

from bleak import BleakClient, BleakScanner
from bleak.backends.scanner import AdvertisementData
from bleak.backends.device import BLEDevice

CHAR_WRITE = "00002bb1-0000-1000-8000-00805f9b34fb"
CHAR_NOTIFY = "00002bb0-0000-1000-8000-00805f9b34fb"
CHAR_BATTERY = "00002a19-0000-1000-8000-00805f9b34fb"  # SIG Battery Level

FAKE_AUTH = bytes([0x01, 0x01, 0x00, 0x02, 0x00, 0x00]) + b"0" * 32

# Candidate follow-up commands (nullary, should produce short responses)
PROBES = [
    ("GetDeviceName  0x006C", bytes([0x01, 0x6C, 0x00])),
    ("GetState       0x0003", bytes([0x01, 0x03, 0x00])),
    ("Opcode 0x0006          ", bytes([0x01, 0x06, 0x00])),
    ("Opcode 0x0009          ", bytes([0x01, 0x09, 0x00])),
]


async def find_plaud(timeout: float = 20.0) -> BLEDevice | None:
    print(f"[scan] looking for PLAUD_NOTE…", flush=True)
    found: dict[str, BLEDevice] = {}

    def cb(device: BLEDevice, adv: AdvertisementData) -> None:
        name = device.name or adv.local_name or ""
        if "PLAUD" in name.upper() and device.address not in found:
            print(f"[scan] {device.address} rssi={adv.rssi}", flush=True)
            found[device.address] = device

    async with BleakScanner(detection_callback=cb):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline and not found:
            await asyncio.sleep(0.5)
    return next(iter(found.values()), None)


async def main() -> int:
    device = await find_plaud()
    if device is None:
        print("[scan] no PLAUD_NOTE")
        return 2

    start = time.monotonic()
    notifs: list[tuple[float, bytes]] = []

    def on_notify(_s, data: bytearray) -> None:
        t = (time.monotonic() - start) * 1000
        notifs.append((t, bytes(data)))
        print(f"[rx +{t:7.1f}ms]  {bytes(data).hex()}", flush=True)

    try:
        async with BleakClient(device, timeout=15.0) as client:
            print(f"[connect] OK  mtu={client.mtu_size}", flush=True)
            await client.start_notify(CHAR_NOTIFY, on_notify)
            await asyncio.sleep(0.3)

            print(f"[tx      0ms ]  fake auth ({len(FAKE_AUTH)} B)", flush=True)
            await client.write_gatt_char(CHAR_WRITE, FAKE_AUTH, response=False)
            # Give the device ~2s to reply to the auth
            await asyncio.sleep(2.0)

            # Try a standard SIG battery read first (no vendor auth should be needed for it)
            try:
                batt = await client.read_gatt_char(CHAR_BATTERY)
                print(f"[read] battery level = {batt.hex()} ({int(batt[0]) if batt else '??'}%)", flush=True)
            except Exception as e:
                print(f"[read] battery read failed: {type(e).__name__}: {e}", flush=True)

            # Fire the probes one by one
            for label, frame in PROBES:
                if not client.is_connected:
                    print("[state] DISCONNECTED before probe")
                    break
                print(f"[tx  {(time.monotonic()-start)*1000:7.1f}ms]  {label}  bytes={frame.hex()}", flush=True)
                try:
                    await client.write_gatt_char(CHAR_WRITE, frame, response=False)
                except Exception as e:
                    print(f"[tx] write failed: {type(e).__name__}: {e}", flush=True)
                    break
                await asyncio.sleep(1.5)

            print(f"[result] still connected: {client.is_connected}")
            print(f"[result] total notifications: {len(notifs)}")
            return 0 if client.is_connected else 1
    except Exception as e:
        print(f"[error] {type(e).__name__}: {e}")
        return 3


if __name__ == "__main__":
    sys.exit(asyncio.run(main()))
