//! btleplug-backed BLE adapter: scan, connect, CCCD, channel bridge,
//! and battery reader.
//!
//! All btleplug types are confined to this module. The rest of the
//! crate sees only [`BleChannel`], [`BatteryReader`], and
//! [`ScanProvider`].

use std::time::Duration;

use async_trait::async_trait;
use btleplug::api::{Central, Characteristic, Manager as _, Peripheral as _, ScanFilter, WriteType};
use btleplug::platform::{Adapter, Manager, Peripheral};
use bytes::Bytes;
use futures::stream::StreamExt;
use plaud_domain::{BatteryLevel, DeviceCandidate, TransportHint};
use plaud_transport::{Error, Result};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::battery::BatteryReader;
use crate::channel::BleChannel;
use crate::constants::DEFAULT_CHANNEL_CAPACITY;
use crate::discovery::ScanProvider;

// ---------------------------------------------------------------------------
// GATT UUIDs from docs/protocol/ble-gatt.md
// ---------------------------------------------------------------------------

/// Control-in characteristic (phone → device), UUID `0x2BB1`.
const CONTROL_IN_UUID: Uuid = Uuid::from_u128(0x00002bb1_0000_1000_8000_00805f9b34fb);

/// Control+bulk-out characteristic (device → phone, notify), UUID `0x2BB0`.
const NOTIFY_OUT_UUID: Uuid = Uuid::from_u128(0x00002bb0_0000_1000_8000_00805f9b34fb);

/// Standard SIG Battery Level characteristic, UUID `0x2A19`.
const BATTERY_LEVEL_UUID: Uuid = Uuid::from_u128(0x00002a19_0000_1000_8000_00805f9b34fb);

/// Expected local name in advertising data.
const EXPECTED_LOCAL_NAME: &str = "PLAUD_NOTE";

/// Nordic Semiconductor manufacturer ID from the advertising data.
const NORDIC_MANUFACTURER_ID: u16 = 0x0059;

// ---------------------------------------------------------------------------
// ScanProvider
// ---------------------------------------------------------------------------

/// Discovers Plaud devices via btleplug BLE scanning.
pub struct BtleplugScanProvider;

#[async_trait]
impl ScanProvider for BtleplugScanProvider {
    async fn scan(&self, timeout: Duration) -> Result<Vec<DeviceCandidate>> {
        let adapter = default_adapter().await?;

        adapter
            .start_scan(ScanFilter::default())
            .await
            .map_err(|e| Error::Transport(format!("scan start failed: {e}")))?;

        tokio::time::sleep(timeout).await;

        let _ = adapter.stop_scan().await;

        let peripherals = adapter
            .peripherals()
            .await
            .map_err(|e| Error::Transport(format!("enumerate peripherals: {e}")))?;

        let mut candidates = Vec::new();
        for p in peripherals {
            if let Some(candidate) = peripheral_to_candidate(&p).await {
                candidates.push(candidate);
            }
        }
        Ok(candidates)
    }
}

/// Check if a discovered peripheral matches the Plaud advertising
/// profile and convert it to a `DeviceCandidate`.
async fn peripheral_to_candidate(p: &Peripheral) -> Option<DeviceCandidate> {
    let props = p.properties().await.ok()??;
    let name = props.local_name.as_deref()?;
    if name != EXPECTED_LOCAL_NAME {
        return None;
    }
    if !props.manufacturer_data.contains_key(&NORDIC_MANUFACTURER_ID) {
        return None;
    }
    let rssi = props.rssi;
    Some(DeviceCandidate::new(
        name.to_owned(),
        NORDIC_MANUFACTURER_ID,
        rssi,
        TransportHint::Ble,
    ))
}

// ---------------------------------------------------------------------------
// connect_peripheral — the bridge from btleplug → BleChannel
// ---------------------------------------------------------------------------

/// Scan for a Plaud device, connect, enable CCCD on the vendor notify
/// characteristic, and return a [`BleChannel`] + [`BtleplugBatteryReader`]
/// ready for use by [`crate::session::BleSession`].
///
/// The returned channel bridges btleplug notifications into the
/// `BleChannel::rx` sender and `BleChannel::tx` receiver into btleplug
/// GATT writes, so the session layer is completely unaware of btleplug.
pub async fn connect_peripheral(scan_timeout: Duration) -> Result<(BleChannel, BtleplugBatteryReader)> {
    let adapter = default_adapter().await?;
    let peripheral = scan_and_find(&adapter, scan_timeout).await?;

    peripheral
        .connect()
        .await
        .map_err(|e| Error::Transport(format!("connect failed: {e}")))?;

    peripheral
        .discover_services()
        .await
        .map_err(|e| Error::Transport(format!("service discovery failed: {e}")))?;

    let chars = peripheral.characteristics();

    let control_in = find_characteristic(&chars, CONTROL_IN_UUID)?;
    let notify_out = find_characteristic(&chars, NOTIFY_OUT_UUID)?;
    let battery_char = find_characteristic(&chars, BATTERY_LEVEL_UUID)?;

    // Enable notifications on the vendor notify characteristic (CCCD)
    peripheral
        .subscribe(&notify_out)
        .await
        .map_err(|e| Error::Transport(format!("CCCD subscribe failed: {e}")))?;

    // Brief settle time for BlueZ to flush any stale notifications
    // from a previous connection.
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Build the BleChannel by bridging btleplug ↔ mpsc
    let (session_tx, peer_rx) = mpsc::channel::<Bytes>(DEFAULT_CHANNEL_CAPACITY);
    let (peer_tx, session_rx) = mpsc::channel::<Bytes>(DEFAULT_CHANNEL_CAPACITY);

    let channel = BleChannel {
        tx: session_tx,
        rx: session_rx,
    };

    // Spawn notification handler: btleplug notifications → peer_tx → session rx
    let p_notify = peripheral.clone();
    tokio::spawn(async move {
        let Ok(mut stream) = p_notify.notifications().await else {
            return;
        };
        while let Some(data) = stream.next().await {
            let val = &data.value;
            if val.len() >= 2 {
                tracing::trace!(
                    len = val.len(),
                    first_bytes = %val.iter().take(12.min(val.len())).map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" "),
                    "ble notification"
                );
            }
            if peer_tx.send(Bytes::from(data.value)).await.is_err() {
                break;
            }
        }
    });

    // Spawn write handler: session tx → peer_rx → btleplug writes
    let p_write = peripheral.clone();
    let control_in_write = control_in.clone();
    tokio::spawn(async move {
        let mut rx = peer_rx;
        while let Some(frame) = rx.recv().await {
            if p_write.write(&control_in_write, &frame, WriteType::WithoutResponse).await.is_err() {
                break;
            }
        }
    });

    let battery_reader = BtleplugBatteryReader { peripheral, battery_char };

    Ok((channel, battery_reader))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Get the first available BLE adapter.
async fn default_adapter() -> Result<Adapter> {
    let manager = Manager::new()
        .await
        .map_err(|e| Error::Transport(format!("btleplug manager: {e}")))?;
    let adapters = manager
        .adapters()
        .await
        .map_err(|e| Error::Transport(format!("no BLE adapters: {e}")))?;
    adapters
        .into_iter()
        .next()
        .ok_or_else(|| Error::Transport("no BLE adapter found".to_owned()))
}

/// Scan for a Plaud device and return the first matching peripheral.
async fn scan_and_find(adapter: &Adapter, timeout: Duration) -> Result<Peripheral> {
    adapter
        .start_scan(ScanFilter::default())
        .await
        .map_err(|e| Error::Transport(format!("scan failed: {e}")))?;

    tokio::time::sleep(timeout).await;

    let _ = adapter.stop_scan().await;

    let peripherals = adapter
        .peripherals()
        .await
        .map_err(|e| Error::Transport(format!("enumerate: {e}")))?;

    for p in peripherals {
        if peripheral_to_candidate(&p).await.is_some() {
            return Ok(p);
        }
    }
    Err(Error::Transport(
        "no PLAUD_NOTE device found — is it turned on and nearby?".to_owned(),
    ))
}

/// Find a characteristic by UUID in the discovered set.
fn find_characteristic(chars: &std::collections::BTreeSet<Characteristic>, uuid: Uuid) -> Result<Characteristic> {
    chars
        .iter()
        .find(|c| c.uuid == uuid)
        .cloned()
        .ok_or_else(|| Error::Transport(format!("characteristic {uuid} not found after service discovery")))
}

// ---------------------------------------------------------------------------
// BatteryReader
// ---------------------------------------------------------------------------

/// Reads the standard SIG Battery Level from a connected peripheral.
pub struct BtleplugBatteryReader {
    peripheral: Peripheral,
    battery_char: Characteristic,
}

#[async_trait]
impl BatteryReader for BtleplugBatteryReader {
    async fn read_battery(&self) -> Result<BatteryLevel> {
        let data = self
            .peripheral
            .read(&self.battery_char)
            .await
            .map_err(|e| Error::Transport(format!("battery read failed: {e}")))?;
        let percent = data.first().copied().unwrap_or(0);
        BatteryLevel::new(percent).map_err(|e| Error::Protocol(format!("invalid battery level: {e}")))
    }
}
