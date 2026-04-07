"""Test 2: does the Plaud Note validate the auth token?

Connect to the PLAUD_NOTE, enable notifications on the vendor notify
characteristic, send a fabricated 0x0001 auth frame carrying a token of
all ASCII '0's, and observe for 6 seconds whether the device accepts it
(stays connected, possibly responds) or drops us with a remote disconnect.
"""
from __future__ import annotations

import asyncio
import sys
import time

from bleak import BleakClient, BleakScanner
from bleak.backends.scanner import AdvertisementData
from bleak.backends.device import BLEDevice

CHAR_WRITE = "00002bb1-0000-1000-8000-00805f9b34fb"   # vendor control-in
CHAR_NOTIFY = "00002bb0-0000-1000-8000-00805f9b34fb"  # vendor notify-out

# Fake auth frame: copy the observed wire structure, replace token with zeros.
#   01 01 00      type=1, opcode=0x0001 LE
#   02 00 00      length field + version (matches wire bytes)
#   30 * 32       ASCII '0' x 32 = the token
FAKE_AUTH = bytes([0x01, 0x01, 0x00, 0x02, 0x00, 0x00]) + b"0" * 32


async def find_plaud(timeout: float = 20.0) -> BLEDevice | None:
    print(f"[scan] looking for PLAUD_NOTE for up to {timeout:.0f}s…", flush=True)
    found: dict[str, BLEDevice] = {}

    def cb(device: BLEDevice, adv: AdvertisementData) -> None:
        name = device.name or adv.local_name or ""
        if "PLAUD" in name.upper():
            if device.address not in found:
                print(
                    f"[scan]   {device.address}  rssi={adv.rssi}  name={name!r}",
                    flush=True,
                )
            found[device.address] = device

    async with BleakScanner(detection_callback=cb) as _scanner:
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline and not found:
            await asyncio.sleep(0.5)
    return next(iter(found.values()), None)


async def main() -> int:
    device = await find_plaud()
    if device is None:
        print("[scan] no PLAUD_NOTE found — is it awake and not connected to the phone?")
        return 2

    print(f"[connect] {device.address}", flush=True)
    start = time.monotonic()
    notifs: list[bytes] = []

    def on_notify(_sender, data: bytearray) -> None:
        t = time.monotonic() - start
        notifs.append(bytes(data))
        print(f"[notify +{t*1000:6.0f}ms] {bytes(data).hex()}", flush=True)

    try:
        async with BleakClient(device, timeout=15.0) as client:
            print(f"[connect] connected, MTU={client.mtu_size}", flush=True)
            await client.start_notify(CHAR_NOTIFY, on_notify)
            print("[notify] subscribed to 0x2BB0", flush=True)
            # Send the fake auth frame
            await asyncio.sleep(0.3)  # let CCCD settle
            print(f"[send ] fake auth frame ({len(FAKE_AUTH)} B): {FAKE_AUTH.hex()}", flush=True)
            await client.write_gatt_char(CHAR_WRITE, FAKE_AUTH, response=False)
            print("[send ] write_gatt_char returned", flush=True)
            # Observe for 6 seconds
            for i in range(12):
                await asyncio.sleep(0.5)
                if not client.is_connected:
                    print(f"[state] disconnected at +{(time.monotonic()-start)*1000:.0f}ms", flush=True)
                    break
            still = client.is_connected
            print(f"[result] connected after 6s: {still}")
            print(f"[result] notifications received: {len(notifs)}")
            if notifs:
                print("[result] first notification:", notifs[0].hex())
            return 0 if still else 1
    except Exception as e:
        print(f"[error] {type(e).__name__}: {e}", flush=True)
        return 3


if __name__ == "__main__":
    sys.exit(asyncio.run(main()))
