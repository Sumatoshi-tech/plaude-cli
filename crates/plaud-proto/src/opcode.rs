//! 16-bit opcode constants for the tinnotech pen-BLE protocol.
//!
//! Every opcode listed here was observed on the wire in at least one
//! captured btsnoop session under
//! [`specs/re/captures/btsnoop/`](../../../specs/re/captures/btsnoop/)
//! and cross-referenced against the SDK builder classes under
//! [`specs/re/apk-notes/3.14.0-620/ble-protocol.md`](../../../specs/re/apk-notes/3.14.0-620/ble-protocol.md).
//!
//! Additional SDK opcodes that we have not yet exercised on the wire
//! are tracked in the backlog at
//! `specs/re/backlog.md` and will land with their typed `encode::*`
//! wrappers in milestone M11.

/// `Authenticate` — first vendor write of every BLE session. Payload
/// carries the V0095 prefix and the 16- or 32-char ASCII hex token.
pub const OPCODE_AUTHENTICATE: u16 = 0x0001;

/// `GetState` — nullary. Response is a 15-byte status tuple.
pub const OPCODE_GET_STATE: u16 = 0x0003;

/// Opcode `0x0004` — takes a `u32 LE` Unix timestamp plus a `u16`
/// trailer. Observed in every metadata sweep.
pub const OPCODE_04_TIMESTAMP: u16 = 0x0004;

/// Opcode `0x0006` — nullary. Response is a 27-byte tuple with
/// storage / counter values.
pub const OPCODE_GET_STORAGE_STATS: u16 = 0x0006;

/// `CommonSettings` — reads or writes a single device setting.
/// Payload shape: `<ActionType u8> <SettingType u8> 00 <long> <long>`.
pub const OPCODE_COMMON_SETTINGS: u16 = 0x0008;

/// Opcode `0x0009` — nullary. Response is a 1-byte percentage.
pub const OPCODE_09_PERCENT: u16 = 0x0009;

/// Opcode `0x0016` — file-id + u8 trailer.
pub const OPCODE_16_QUERY_BY_FILE_ID: u16 = 0x0016;

/// Opcode `0x0018` — nullary.
pub const OPCODE_18_NULLARY: u16 = 0x0018;

/// Opcode `0x0019` — single `u8` argument.
pub const OPCODE_19_U8: u16 = 0x0019;

/// Opcode `0x001A` — two `u32 LE` timestamps + 4 reserved bytes.
pub const OPCODE_1A_TIMESTAMP_WINDOW: u16 = 0x001A;

/// `ReadFileChunk` — triggers the bulk `0x02`-magic stream for a
/// range of a recording. Payload: `<file_id u32 LE> <offset u32 LE> <length u32 LE>`.
pub const OPCODE_READ_FILE_CHUNK: u16 = 0x001C;

/// Opcode `0x001E` — single `u32 LE` file-id.
pub const OPCODE_1E_FILE_ID: u16 = 0x001E;

/// Opcode `0x0026` — three `u32 LE` integers. Exact semantics TBD by M11.
pub const OPCODE_26_CONFIG_TRIPLE: u16 = 0x0026;

/// `SetPrivacy` — single `u8` boolean (0 off, 1 on).
pub const OPCODE_SET_PRIVACY: u16 = 0x0067;

/// `GetDeviceName` — nullary. Response is the ASCII `PLAUD_NOTE`
/// padded to ~30 bytes with `0x00`.
pub const OPCODE_GET_DEVICE_NAME: u16 = 0x006C;

/// `CloseSession` (conjectured) — single-byte arg, appears as the
/// last control write of every observed sync session.
pub const OPCODE_CLOSE_SESSION: u16 = 0x006D;
