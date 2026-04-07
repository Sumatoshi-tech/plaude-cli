//! Magic values used by the auth store and the btsnoop parser.

/// Keyring service name the `KeyringStore` uses as the primary key.
pub const KEYRING_SERVICE: &str = "plaude-cli";

/// Device id used for the single-token M4 storage model. Multi-device
/// keying arrives in a later milestone.
pub const DEFAULT_DEVICE_ID: &str = "default";

/// Subdirectory under the user's config dir where the file backend
/// writes the token. Resolves to `~/.config/plaude/` on Linux and
/// the platform equivalent elsewhere via the `dirs` crate.
pub const CONFIG_SUBDIR: &str = "plaude";

/// File name of the token file inside the config dir.
pub const TOKEN_FILE_NAME: &str = "token";

/// Permissions the Unix implementation sets on the token file after
/// writing.
#[cfg(unix)]
pub const TOKEN_FILE_MODE: u32 = 0o600;

/// Permissions the Unix implementation sets on the parent directory.
#[cfg(unix)]
pub const TOKEN_PARENT_DIR_MODE: u32 = 0o700;

/// Number of hex characters the fingerprint helper emits from the
/// SHA-256 digest. Chosen to match what we have been using in evidence
/// documents (e.g. the 16-char fingerprint in
/// `specs/re/captures/ble-live-tests/2026-04-05-token-validation.md`).
pub const FINGERPRINT_HEX_LEN: usize = 16;

// ---------------------------------------------------------------------
// btsnoop format constants
// See https://opensource.apple.com/source/BluetoothHCI/ for the format
// reference.
// ---------------------------------------------------------------------

/// Magic bytes at the start of every btsnoop file: ASCII `"btsnoop\0"`.
pub const BTSNOOP_MAGIC: &[u8; 8] = b"btsnoop\0";

/// Length of the btsnoop file header (magic + version + datalink).
pub const BTSNOOP_HEADER_LEN: usize = 16;

/// Only version `1` is defined by the spec.
pub const BTSNOOP_VERSION: u32 = 1;

/// Length of each record's fixed header (before the packet data).
pub const BTSNOOP_RECORD_HEADER_LEN: usize = 24;

// ---------------------------------------------------------------------
// HCI H4 / L2CAP / ATT constants used to locate the auth frame inside
// btsnoop packets.
// ---------------------------------------------------------------------

/// HCI H4 packet type for ACL Data.
pub const HCI_PACKET_TYPE_ACL: u8 = 0x02;

/// Length of the ACL header (handle+flags 2 B + data length 2 B).
pub const ACL_HEADER_LEN: usize = 4;

/// Length of the L2CAP header (pdu length 2 B + cid 2 B).
pub const L2CAP_HEADER_LEN: usize = 4;

/// L2CAP channel id for the Attribute Protocol.
pub const L2CAP_CID_ATT: u16 = 0x0004;

/// ATT opcode for Write Command (what the phone uses for the vendor
/// auth frame on handle `0x000D`).
pub const ATT_OPCODE_WRITE_COMMAND: u8 = 0x52;

/// Vendor write characteristic handle on the Plaud Note (matches the
/// GATT map in `docs/protocol/ble-gatt.md`).
pub const VENDOR_WRITE_HANDLE: u16 = 0x000D;

/// ATT Write Command payload starts with a 2-byte handle, then value.
pub const ATT_WRITE_VALUE_OFFSET: usize = 3;

/// Fixed byte prefix every V0095-style auth frame begins with. The
/// token characters follow immediately afterwards (16 or 32 chars).
pub const AUTH_FRAME_PREFIX: &[u8] = &[0x01, 0x01, 0x00, 0x02, 0x00, 0x00];
