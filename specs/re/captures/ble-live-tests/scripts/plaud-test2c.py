"""Test 2c: replay the real captured token from a non-phone BLE central.

Reads the real token out of our gitignored btsnoop log, connects to
the Plaud device from this laptop (not the user's phone), sends the
token, then fires GetDeviceName to confirm the device is speaking to us.
"""
from __future__ import annotations

import asyncio
import subprocess
import sys
import time

from bleak import BleakClient, BleakScanner
from bleak.backends.scanner import AdvertisementData
from bleak.backends.device import BLEDevice

CHAR_WRITE = "00002bb1-0000-1000-8000-00805f9b34fb"
CHAR_NOTIFY = "00002bb0-0000-1000-8000-00805f9b34fb"

BTSNOOP_LOG = "/home/dmitriy/sources/plaude-cli/specs/re/captures/btsnoop/2026-04-05-plaud-0day-pair.log"


def extract_real_token() -> bytes:
    """Pull the first observed 0x0001 auth frame out of our raw btsnoop log."""
    out = subprocess.run(
        [
            "tshark",
            "-r",
            BTSNOOP_LOG,
            "-Y",
            "btatt.opcode == 0x52 and btatt.handle == 0x000d",
            "-T",
            "fields",
            "-e",
            "btatt.value",
        ],
        capture_output=True,
        text=True,
        check=True,
    )
    for line in out.stdout.splitlines():
        line = line.strip()
        if line.startswith("010100"):
            frame = bytes.fromhex(line)
            # Sanity: header 01 01 00, then the auth body
            assert frame[0] == 0x01 and frame[1] == 0x01 and frame[2] == 0x00
            return frame
    raise RuntimeError("no 0x0001 auth frame found in btsnoop")


async def find_plaud(timeout: float = 20.0) -> BLEDevice | None:
    print("[scan] looking for PLAUD_NOTE…", flush=True)
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
    real_auth = extract_real_token()
    print(f"[prep] extracted real auth frame: {len(real_auth)} B (token redacted from log)")

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
            print(f"[connect] OK mtu={client.mtu_size}", flush=True)
            await client.start_notify(CHAR_NOTIFY, on_notify)
            await asyncio.sleep(0.3)

            print(f"[tx      0ms]  REAL auth frame", flush=True)
            await client.write_gatt_char(CHAR_WRITE, real_auth, response=False)
            await asyncio.sleep(2.0)

            # Probe GetDeviceName; the real response is ASCII "PLAUD_NOTE" padded
            print(f"[tx  {(time.monotonic()-start)*1000:7.1f}ms]  GetDeviceName 0x006C", flush=True)
            await client.write_gatt_char(CHAR_WRITE, bytes([0x01, 0x6C, 0x00]), response=False)
            await asyncio.sleep(2.0)

            # Probe GetState
            print(f"[tx  {(time.monotonic()-start)*1000:7.1f}ms]  GetState 0x0003", flush=True)
            await client.write_gatt_char(CHAR_WRITE, bytes([0x01, 0x03, 0x00]), response=False)
            await asyncio.sleep(2.0)

            # Probe Opcode 0x06 (storage/counter stats)
            print(f"[tx  {(time.monotonic()-start)*1000:7.1f}ms]  Opcode 0x0006", flush=True)
            await client.write_gatt_char(CHAR_WRITE, bytes([0x01, 0x06, 0x00]), response=False)
            await asyncio.sleep(2.0)

            print(f"[result] still connected: {client.is_connected}")
            print(f"[result] notifications received: {len(notifs)}")
            # Summarise: did we see a GetDeviceName response (ASCII PLAUD bytes)?
            for t, payload in notifs:
                if len(payload) >= 7 and payload[:3] == bytes([0x01, 0x6C, 0x00]):
                    ascii_tail = payload[3:].rstrip(b"\x00").decode("ascii", errors="replace")
                    print(f"[WIN] GetDeviceName response at +{t:.0f}ms = {ascii_tail!r}")
            return 0 if client.is_connected else 1
    except Exception as e:
        print(f"[error] {type(e).__name__}: {e}")
        return 3


if __name__ == "__main__":
    sys.exit(asyncio.run(main()))
