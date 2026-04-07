//! Magic values used by the simulator. Every literal the rest of the
//! crate depends on lives here, so a protocol change is a single-edit
//! site.

use std::time::Duration;

/// Manufacturer-data key advertised by real Plaud hardware
/// (Nordic Semiconductor). `SimDiscovery::scan` emits candidates
/// carrying this value so transport-layer filters match reality.
pub const PLAUD_MANUFACTURER_ID_NORDIC: u16 = 0x0059;

/// Stable local name every Plaud Note advertises. Used by the sim's
/// discovery helper to produce realistic candidates.
pub const PLAUD_LOCAL_NAME: &str = "PLAUD_NOTE";

/// Default battery percentage used by `SimDeviceBuilder` when the
/// test does not configure one explicitly.
pub const DEFAULT_BATTERY_PERCENT: u8 = 100;

/// Default scan duration honoured by `SimDiscovery::scan` when the
/// caller does not supply one. Tests typically pass a shorter value.
pub const DEFAULT_SCAN_TIMEOUT: Duration = Duration::from_millis(1);

/// How many device-like bytes a single bulk data frame carries
/// (matches `docs/protocol/ble-commands.md` §2). Used by
/// `bulk::frames_for` to split an arbitrary byte slice into
/// wire-shaped chunks.
pub const BULK_FRAME_CHUNK_BYTES: usize = 80;

/// Auth status byte the sim emits when the stored token does not
/// match the configured `expected_token`. Kept in sync with the
/// evidence in
/// `specs/re/captures/ble-live-tests/2026-04-05-token-validation.md`.
pub const AUTH_STATUS_REJECTED: u8 = 0x01;
