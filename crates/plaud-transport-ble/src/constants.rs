//! Magic values used by the BLE transport.

use std::time::Duration;

/// Maximum time the session waits for the auth-response notification
/// after writing an auth frame. Matches the 5-second budget the M5
/// DoD calls out.
pub const AUTH_RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);

/// Maximum time `send_control` waits for a response notification after
/// dispatching a vendor request.
pub const CONTROL_RESPONSE_TIMEOUT: Duration = Duration::from_secs(5);

/// Maximum time `read_bulk` waits between consecutive bulk frames.
/// Reset after every received frame.
pub const BULK_FRAME_TIMEOUT: Duration = Duration::from_secs(10);

/// Default channel buffer size for `BleChannel`. Picked as 64 so a
/// bulk burst of several hundred notifications does not back-pressure
/// the notification handler — 64 × 90 byte frames = ~5.7 kB in-flight,
/// which is small enough to avoid pressure while big enough that any
/// realistic test completes without re-allocation.
pub const DEFAULT_CHANNEL_CAPACITY: usize = 64;

/// Status byte the BLE transport translates into
/// [`plaud_transport::Error::AuthRejected`] when the device's auth
/// response carries "soft reject".
pub const AUTH_STATUS_REJECTED: u8 = 0x01;

/// Capability string attached to the `Unsupported` error for the
/// vendor commands M5 does not yet implement. Downstream milestones
/// overwrite the matching transport method and stop producing this
/// error.
pub const CAP_DEVICE_INFO: &str = "device_info (lands in M6)";
/// Capability string for `storage`, filled in by M6.
pub const CAP_STORAGE: &str = "storage (lands in M6)";
/// Capability string for `list_recordings`, filled in by M7.
pub const CAP_LIST_RECORDINGS: &str = "list_recordings (lands in M7)";
/// Capability string for `read_recording`, filled in by M7.
pub const CAP_READ_RECORDING: &str = "read_recording (lands in M7)";
/// Capability string for `read_recording_asr`, filled in by M7
/// on the sim; the real-hardware implementation lands with the
/// btleplug backend.
pub const CAP_READ_RECORDING_ASR: &str = "read_recording_asr (lands in M7)";
/// Capability string for `delete_recording`, filled in by M7.
pub const CAP_DELETE_RECORDING: &str = "delete_recording (lands in M7)";
/// Capability string for `read_setting`, filled in by M11.
pub const CAP_READ_SETTING: &str = "read_setting (lands in M11)";
/// Capability string for `write_setting`, filled in by M11.
pub const CAP_WRITE_SETTING: &str = "write_setting (lands in M11)";
/// Capability string for `start_recording`, filled in by M11.
pub const CAP_START_RECORDING: &str = "start_recording (lands in M11)";
/// Capability string for `stop_recording`, filled in by M11.
pub const CAP_STOP_RECORDING: &str = "stop_recording (lands in M11)";
/// Capability string for `pause_recording`, filled in by M11.
pub const CAP_PAUSE_RECORDING: &str = "pause_recording (lands in M11)";
/// Capability string for `resume_recording`, filled in by M11.
pub const CAP_RESUME_RECORDING: &str = "resume_recording (lands in M11)";
/// Capability string for `set_privacy`, filled in by M11.
pub const CAP_SET_PRIVACY: &str = "set_privacy (lands in M11)";
/// Capability string used when the device advertises the RSA + ChaCha20
/// handshake path that lands in M16.
pub const CAP_RSA_HANDSHAKE: &str = "rsa-chacha20-handshake (lands in M16)";
