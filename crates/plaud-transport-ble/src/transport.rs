//! `BleTransport` — the real `plaud_transport::Transport` implementation
//! over a BLE session.
//!
//! Drives [`BleSession::send_control`] and [`BleSession::read_bulk`]
//! to execute vendor opcodes against a connected Plaud device.
//! Battery reads bypass the session and go directly through the
//! [`BatteryReader`] trait (standard SIG service, no auth required).

use std::sync::Arc;

use async_trait::async_trait;
use plaud_domain::{
    BatteryLevel, CommonSettingKey, DeviceInfo, DeviceModel, DeviceSerial, FirmwareVersion, Recording, RecordingId, RecordingKind,
    SettingValue, StorageStats,
};
use plaud_proto::{
    encode::{self, device as device_enc, file as file_enc, metadata as meta_enc},
    opcode,
};
use plaud_transport::{Error, Result, Transport};
use tokio::sync::Mutex;
use tracing::debug;

use crate::{battery::BatteryReader, session::BleSession};

/// Public `Transport` impl for a connected Plaud BLE session.
///
/// Build one via [`BleTransport::from_parts`] (for tests) or via the
/// btleplug backend factory.
pub struct BleTransport {
    session: Arc<Mutex<BleSession>>,
    battery: Arc<dyn BatteryReader>,
}

impl BleTransport {
    /// Build a transport from an already-owned session and battery
    /// reader.
    #[must_use]
    pub fn from_parts(session: Arc<Mutex<BleSession>>, battery: Arc<dyn BatteryReader>) -> Self {
        Self { session, battery }
    }

    /// Borrow the underlying session for direct access.
    #[must_use]
    pub fn session(&self) -> Arc<Mutex<BleSession>> {
        Arc::clone(&self.session)
    }
}

impl std::fmt::Debug for BleTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BleTransport").finish_non_exhaustive()
    }
}

/// Format a byte slice as space-separated hex for debug logging.
fn hex_display(data: &[u8]) -> String {
    data.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ")
}

/// Trim trailing `0x00` bytes from a response payload and decode as
/// UTF-8 (lossy).
fn payload_to_string(payload: &[u8]) -> String {
    let trimmed = payload.iter().rposition(|&b| b != 0).map_or(&payload[..0], |i| &payload[..=i]);
    String::from_utf8_lossy(trimmed).into_owned()
}

#[async_trait]
impl Transport for BleTransport {
    async fn device_info(&self) -> Result<DeviceInfo> {
        let mut session = self.session.lock().await;

        let name_frame = device_enc::get_device_name();
        let name_payload = session.send_control(name_frame, opcode::OPCODE_GET_DEVICE_NAME).await?;
        let local_name = payload_to_string(&name_payload);

        let state_frame = device_enc::get_state();
        let _state_payload = session.send_control(state_frame, opcode::OPCODE_GET_STATE).await?;

        let model = if local_name.contains("NOTE") {
            DeviceModel::Note
        } else {
            DeviceModel::Unknown(local_name.clone())
        };

        Ok(DeviceInfo {
            local_name,
            model,
            firmware: FirmwareVersion::placeholder(),
            serial: DeviceSerial::placeholder(),
        })
    }

    async fn battery(&self) -> Result<BatteryLevel> {
        self.battery.read_battery().await
    }

    async fn storage(&self) -> Result<StorageStats> {
        let mut session = self.session.lock().await;
        let frame = device_enc::get_storage_stats();
        let payload = session.send_control(frame, opcode::OPCODE_GET_STORAGE_STATS).await?;

        debug!(len = payload.len(), hex = %hex_display(&payload), "0x0006 storage response");
        // Response is a 27-byte tuple. From btsnoop evidence:
        // `00 00 86 8f 0e 00 00 00 00 00 88 8f 0e 00 00 00 90 ab ce 36 00 00 00 00`
        // Parse conservatively — take what we can.
        if payload.len() < 12 {
            return Err(Error::Protocol(format!(
                "GetStorageStats response too short: {} bytes",
                payload.len()
            )));
        }
        // Layout (24 bytes, payload after control header stripped):
        // Evidence: `00 00 86 8f 0e 00 00 00 00 00 88 8f 0e 00 00 00 90 ab ce 36 00 00 00 00`
        //   [2..6]   = 0x000E8F86 = 954,246  — used pages/blocks
        //   [10..14] = 0x000E8F88 = 954,248  — total pages/blocks
        //   [16..20] = 0x36CEAB90 = 919,514,000 — total flash bytes
        //
        // The two near-equal u32s at [2] and [10] are page/block counts
        // (differ by 2). Total flash capacity is at [16].
        if payload.len() < 20 {
            return Err(Error::Protocol(format!(
                "GetStorageStats response too short: {} bytes",
                payload.len()
            )));
        }
        let used_blocks = u32::from_le_bytes([payload[2], payload[3], payload[4], payload[5]]) as u64;
        let total_blocks = u32::from_le_bytes([payload[10], payload[11], payload[12], payload[13]]) as u64;
        let total_bytes = u32::from_le_bytes([payload[16], payload[17], payload[18], payload[19]]) as u64;

        // Derive used_bytes proportionally from the block counts.
        let used_bytes = if total_blocks > 0 {
            total_bytes * used_blocks / total_blocks
        } else {
            0
        };
        // Recording count isn't directly available from 0x0006 — set to 0.
        // The phone app likely gets it from a different opcode or derives
        // it from the file list.
        let recording_count = 0;

        StorageStats::new(total_bytes, used_bytes, recording_count).map_err(|e| Error::Protocol(format!("storage stats: {e}")))
    }

    async fn list_recordings(&self) -> Result<Vec<Recording>> {
        let mut session = self.session.lock().await;

        // The Tinnotech SDK getFileList (opcode 0x001A) is a DELTA
        // protocol: it returns only recordings that haven't been synced
        // since the last session. For a device that the Plaud phone app
        // has already synced, it returns 0 files.
        //
        // To get a full listing, use `--backend usb` which reads the
        // VFAT filesystem directly.
        //
        // Protocol: C9592t(epochSeconds, sessionId=0, single=false)
        // Response: multi-control-frame with C9809v format.
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;
        let frame = meta_enc::get_file_list(now, 0, false);

        let responses = session.send_control_multi(frame, opcode::OPCODE_1A_TIMESTAMP_WINDOW).await?;
        for (i, r) in responses.iter().enumerate() {
            debug!(batch = i, len = r.len(), hex = %hex_display(r), "0x001A raw response");
        }

        let mut recordings = Vec::new();
        for resp in &responses {
            if resp.len() < 8 {
                continue;
            }
            let total_files = u16::from_le_bytes([resp[4], resp[5]]) as usize;
            debug!(total_files, resp_len = resp.len(), "file list batch");

            // Parse per-file entries (8 bytes each: file_id:u32 + size:u32)
            let mut offset = 8;
            while offset + 8 <= resp.len() && recordings.len() < total_files {
                let file_id = u32::from_le_bytes([resp[offset], resp[offset + 1], resp[offset + 2], resp[offset + 3]]);
                let file_size = u32::from_le_bytes([resp[offset + 4], resp[offset + 5], resp[offset + 6], resp[offset + 7]]);
                offset += 8;

                if let Ok(id) = RecordingId::new(format!("{file_id}")) {
                    recordings.push(Recording::new(id, RecordingKind::Note, file_size as u64, 0));
                }
            }
        }

        Ok(recordings)
    }

    async fn read_recording(&self, id: &RecordingId) -> Result<Vec<u8>> {
        let file_id = id.as_unix_seconds() as u32;
        let mut session = self.session.lock().await;

        // First get the file list to find the file size, then read it.
        // If the file isn't in the delta list, try a generous default.
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as u32;
        let list_frame = meta_enc::get_file_list(now, 0, false);
        let responses = session.send_control_multi(list_frame, opcode::OPCODE_1A_TIMESTAMP_WINDOW).await?;

        // Look up the file size from the delta listing.
        let mut file_size: Option<u32> = None;
        for resp in &responses {
            if resp.len() < 8 {
                continue;
            }
            let mut off = 8;
            while off + 8 <= resp.len() {
                let fid = u32::from_le_bytes([resp[off], resp[off + 1], resp[off + 2], resp[off + 3]]);
                let fsz = u32::from_le_bytes([resp[off + 4], resp[off + 5], resp[off + 6], resp[off + 7]]);
                if fid == file_id {
                    file_size = Some(fsz);
                }
                off += 8;
            }
        }
        // The device requires the exact file size in ReadFileChunk.
        // If we don't know it (delta list empty), we can't proceed.
        let file_size = file_size.ok_or_else(|| {
            Error::NotFound(format!(
                "recording {id} not in the device's unsynced file list — it may have already been synced by the phone app"
            ))
        })?;
        debug!(file_id, file_size, "read_recording");

        let trigger = file_enc::read_file_chunk(file_id, 0, file_size);
        let data = session.read_bulk(trigger).await?;
        Ok(data)
    }

    async fn read_recording_asr(&self, id: &RecordingId) -> Result<Vec<u8>> {
        // ASR sidecar: the device stores .WAV and .ASR files as separate
        // entities. The ASR file has the same file_id but is accessed
        // via a different mechanism. Based on btsnoop evidence, the
        // bulk transfer in session C was ~76 kB — likely the ASR sidecar.
        //
        // The exact opcode for requesting the ASR specifically vs the WAV
        // is not yet confirmed. For now, attempt a second read with an
        // offset indicator. If the device protocol distinguishes WAV vs ASR
        // by a flag in the file_id or a separate opcode, this will need
        // adjustment after further RE.
        //
        // Fallback: return NotFound to let the caller skip the ASR.
        Err(Error::NotFound(format!("ASR sidecar download over BLE not yet supported for {id}")))
    }

    async fn delete_recording(&self, _id: &RecordingId) -> Result<()> {
        // Delete opcode is not identified in wire captures.
        // The sim does it as an in-memory remove; real BLE needs
        // the actual opcode.
        Err(Error::Unsupported {
            capability: "delete_recording (opcode not yet identified from wire captures)",
        })
    }

    async fn read_setting(&self, key: CommonSettingKey) -> Result<SettingValue> {
        let mut session = self.session.lock().await;
        let frame = encode::settings::read_setting(key);
        let payload = session.send_control(frame, opcode::OPCODE_COMMON_SETTINGS).await?;

        if payload.len() < 3 {
            return Err(Error::Protocol(format!(
                "CommonSettings response too short: {} bytes",
                payload.len()
            )));
        }
        // Byte 0 = action echo, byte 1 = setting code echo, byte 2+ = value
        let value_byte = payload[2];
        Ok(SettingValue::U8(value_byte))
    }

    async fn write_setting(&self, key: CommonSettingKey, value: SettingValue) -> Result<()> {
        let mut session = self.session.lock().await;
        let raw_value: u64 = match value {
            SettingValue::Bool(b) => u64::from(b),
            SettingValue::U8(v) => u64::from(v),
            SettingValue::U32(v) => u64::from(v),
            _ => return Err(Error::Protocol("unsupported setting value type".to_owned())),
        };
        let frame = encode::settings::write_setting(key, raw_value);
        let _payload = session.send_control(frame, opcode::OPCODE_COMMON_SETTINGS).await?;
        Ok(())
    }

    async fn start_recording(&self) -> Result<()> {
        Err(Error::Unsupported {
            capability: "start_recording (opcode not yet identified from wire captures)",
        })
    }

    async fn stop_recording(&self) -> Result<()> {
        Err(Error::Unsupported {
            capability: "stop_recording (opcode not yet identified from wire captures)",
        })
    }

    async fn pause_recording(&self) -> Result<()> {
        Err(Error::Unsupported {
            capability: "pause_recording (opcode not yet identified from wire captures)",
        })
    }

    async fn resume_recording(&self) -> Result<()> {
        Err(Error::Unsupported {
            capability: "resume_recording (opcode not yet identified from wire captures)",
        })
    }

    async fn set_privacy(&self, on: bool) -> Result<()> {
        let mut session = self.session.lock().await;
        let frame = device_enc::set_privacy(on);
        let _payload = session.send_control(frame, opcode::OPCODE_SET_PRIVACY).await?;
        Ok(())
    }
}
