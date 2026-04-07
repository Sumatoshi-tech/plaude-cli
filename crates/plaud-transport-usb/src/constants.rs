//! Constants for the USB MSC transport.
//!
//! Every magic byte, filename, and path fragment used by
//! `plaud-transport-usb` lives in this module so a protocol change
//! is a one-file edit and audit scanners can grep for literals.

/// Filename at the root of the `PLAUD_NOTE` volume that carries the
/// device identity fields. See `docs/protocol/file-formats.md`.
pub const MODEL_TXT_FILENAME: &str = "MODEL.txt";

/// Top-level directory for button-triggered recordings.
pub const NOTES_DIR: &str = "NOTES";

/// Top-level directory for call-mode recordings.
pub const CALLS_DIR: &str = "CALLS";

/// WAV file extension emitted by the device (stereo PCM).
pub const WAV_EXTENSION: &str = "WAV";

/// ASR sidecar extension emitted by the device (mono Opus stream).
pub const ASR_EXTENSION: &str = "ASR";

/// Capability string shared by every stub `Transport` method the
/// USB transport cannot implement (`battery`, `set_privacy`, etc.).
pub const CAP_USB_UNSUPPORTED: &str = "usb-transport-unsupported";

/// Absolute file offset within a V0095 WAV where the `pad ` chunk's
/// `SN:` payload begins. Derived from the `docs/protocol/file-formats.md`
/// chunk layout (RIFF header 12 + fmt chunk 24 + `pad ` header 8 = 44).
pub const WAV_SN_REGION_START: usize = 0x2C;

/// Absolute file offset within a V0095 WAV where the `SN:` payload
/// ends (exclusive). Covers `SN:` magic + 18-byte serial + null
/// terminator = 22 bytes, so `0x2C + 22 = 0x42`.
pub const WAV_SN_REGION_END: usize = 0x42;

/// Deprecation banner printed on every USB-backend command run.
pub const USB_DEPRECATION_NOTICE: &str = "warning: Plaud has announced USB will be disabled in a future firmware update; treat this transport as a pre-deprecation fallback only";
