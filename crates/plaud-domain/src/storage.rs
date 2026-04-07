//! Device storage statistics.
//!
//! The vendor protocol exposes storage counters through opcode `0x0006`
//! (see `specs/re/apk-notes/3.14.0-620/ble-protocol.md`). Raw values
//! are packed into a 27-byte response tuple; the CLI surfaces a
//! normalised view via this struct.

use std::fmt;

use thiserror::Error;

/// Error returned when `StorageStats` fields are internally inconsistent.
#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum StorageStatsError {
    /// `used_bytes` exceeded `total_bytes`, which is impossible for a
    /// well-behaved device report.
    #[error("used_bytes ({used}) exceeds total_bytes ({total})")]
    UsedExceedsTotal {
        /// The offending `used_bytes` value.
        used: u64,
        /// The `total_bytes` value against which it was compared.
        total: u64,
    },
}

/// On-device storage statistics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StorageStats {
    total_bytes: u64,
    used_bytes: u64,
    recording_count: u32,
}

impl StorageStats {
    /// A zero-capacity `StorageStats`. Useful as a default in tests
    /// and simulators where `total_bytes == used_bytes == 0` trivially
    /// satisfies the `used <= total` invariant.
    pub const ZERO: Self = Self {
        total_bytes: 0,
        used_bytes: 0,
        recording_count: 0,
    };

    /// Construct a `StorageStats` and validate that `used_bytes <= total_bytes`.
    ///
    /// # Errors
    ///
    /// Returns [`StorageStatsError::UsedExceedsTotal`] if the used-byte
    /// count is greater than the total-byte count.
    pub const fn new(total_bytes: u64, used_bytes: u64, recording_count: u32) -> Result<Self, StorageStatsError> {
        if used_bytes > total_bytes {
            return Err(StorageStatsError::UsedExceedsTotal {
                used: used_bytes,
                total: total_bytes,
            });
        }
        Ok(Self {
            total_bytes,
            used_bytes,
            recording_count,
        })
    }

    /// Total capacity in bytes.
    #[must_use]
    pub const fn total_bytes(self) -> u64 {
        self.total_bytes
    }

    /// Bytes currently in use by stored recordings.
    #[must_use]
    pub const fn used_bytes(self) -> u64 {
        self.used_bytes
    }

    /// Number of recordings currently on the device.
    #[must_use]
    pub const fn recording_count(self) -> u32 {
        self.recording_count
    }

    /// Bytes available for new recordings.
    #[must_use]
    pub const fn free_bytes(self) -> u64 {
        self.total_bytes - self.used_bytes
    }

    /// Fraction of capacity in use, in `[0.0, 1.0]`. Returns `0.0` when
    /// `total_bytes` is zero (to avoid division by zero on a device
    /// that reports an unformatted drive).
    #[must_use]
    pub fn used_ratio(self) -> f64 {
        if self.total_bytes == 0 {
            return 0.0;
        }
        let used = self.used_bytes as f64;
        let total = self.total_bytes as f64;
        used / total
    }
}

impl fmt::Display for StorageStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} / {} bytes used ({} recordings)",
            self.used_bytes, self.total_bytes, self.recording_count
        )
    }
}
