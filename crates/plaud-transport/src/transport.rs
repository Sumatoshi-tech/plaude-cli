//! The `Transport` trait — the vendor command surface of a connected
//! Plaud device, transport-agnostic.
//!
//! Every method maps 1:1 to either a standard SIG GATT read (for
//! [`Transport::battery`]) or a tinnotech BLE opcode (for everything
//! else). The opcode numbers are documented in
//! [`docs/protocol/ble-commands.md`](../../../docs/protocol/ble-commands.md)
//! and in
//! [`specs/re/apk-notes/3.14.0-620/ble-protocol.md`](../../../specs/re/apk-notes/3.14.0-620/ble-protocol.md).
//!
//! The trait is `dyn`-compatible via `async-trait`. Implementations
//! may be blocking on real hardware (BLE / USB) or purely in-memory
//! (`plaud-sim`).

use async_trait::async_trait;
use plaud_domain::{BatteryLevel, CommonSettingKey, DeviceInfo, Recording, RecordingId, SettingValue, StorageStats};

use crate::error::Result;

/// Every capability the CLI uses to talk to a Plaud device.
///
/// Implementations: `plaud-transport-ble`, `plaud-transport-usb`,
/// `plaud-transport-wifi`, `plaud-sim`.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Read identity from the device (model, firmware, serial,
    /// local name). Corresponds to opcodes `0x0003 GetState` and
    /// `0x006C GetDeviceName` on BLE, or the `MODEL.txt` parser on
    /// the USB fallback.
    async fn device_info(&self) -> Result<DeviceInfo>;

    /// Read battery percentage. On BLE this is a standard SIG service
    /// read (characteristic `0x2A19`) and does not require an auth
    /// token.
    async fn battery(&self) -> Result<BatteryLevel>;

    /// Read storage statistics. Corresponds to opcode `0x0006`.
    async fn storage(&self) -> Result<StorageStats>;

    /// Enumerate recordings on the device.
    async fn list_recordings(&self) -> Result<Vec<Recording>>;

    /// Read the stereo PCM `.WAV` bytes of a single recording.
    /// Corresponds to opcode `0x001C ReadFileChunk` plus the bulk
    /// stream on magic byte `0x02`, targeting the `.WAV` file of the
    /// recording pair.
    async fn read_recording(&self, id: &RecordingId) -> Result<Vec<u8>>;

    /// Read the mono Opus `.ASR` sidecar bytes of a single recording.
    /// Same protocol path as [`Self::read_recording`] but targets the
    /// sidecar file on the device filesystem. Returns [`Error::NotFound`]
    /// if the recording does not exist or has no ASR sidecar.
    async fn read_recording_asr(&self, id: &RecordingId) -> Result<Vec<u8>>;

    /// Delete a recording from the device.
    async fn delete_recording(&self, id: &RecordingId) -> Result<()>;

    /// Read a single device setting by key. Corresponds to opcode
    /// `0x0008 CommonSettings` with `ActionType::Read`.
    async fn read_setting(&self, key: CommonSettingKey) -> Result<SettingValue>;

    /// Write a single device setting. Corresponds to opcode
    /// `0x0008 CommonSettings` with `ActionType::Setting`.
    async fn write_setting(&self, key: CommonSettingKey, value: SettingValue) -> Result<()>;

    /// Begin a new recording. Corresponds to Flutter action
    /// `action/startRecord`.
    async fn start_recording(&self) -> Result<()>;

    /// Stop the current recording and finalise the `.WAV`/`.ASR` pair.
    async fn stop_recording(&self) -> Result<()>;

    /// Pause the current recording (not all firmware supports this).
    async fn pause_recording(&self) -> Result<()>;

    /// Resume a paused recording.
    async fn resume_recording(&self) -> Result<()>;

    /// Toggle the device's privacy flag. Corresponds to opcode
    /// `0x0067 SetPrivacy` (captured in the 0day re-pair session).
    async fn set_privacy(&self, on: bool) -> Result<()>;
}
