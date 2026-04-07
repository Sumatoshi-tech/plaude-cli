//! Battery level reporting.
//!
//! Battery is exposed by the Plaud Note via the standard SIG Battery
//! Service (`0x180F`) at characteristic `0x2A19`. It is the **only**
//! BLE read that succeeds before vendor authentication (confirmed in
//! `specs/re/captures/ble-live-tests/2026-04-05-token-validation.md`).

use std::fmt;

use thiserror::Error;

/// Lower bound of a valid battery percentage.
const BATTERY_LEVEL_MIN: u8 = 0;

/// Upper bound of a valid battery percentage.
const BATTERY_LEVEL_MAX: u8 = 100;

/// Error returned when an invalid battery percentage is supplied.
#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum BatteryLevelError {
    /// The input was outside the valid range `[0, 100]`.
    #[error("battery level {got} is outside the valid range [{min}..={max}]")]
    OutOfRange {
        /// Observed value.
        got: u8,
        /// Inclusive minimum.
        min: u8,
        /// Inclusive maximum.
        max: u8,
    },
}

/// Battery charge expressed as a whole-number percentage in `[0, 100]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BatteryLevel(u8);

impl BatteryLevel {
    /// Lowest valid battery level (`0 %`).
    pub const EMPTY: Self = Self(BATTERY_LEVEL_MIN);

    /// Highest valid battery level (`100 %`).
    pub const FULL: Self = Self(BATTERY_LEVEL_MAX);

    /// Construct a `BatteryLevel` from a percentage in `[0, 100]`.
    ///
    /// # Errors
    ///
    /// Returns [`BatteryLevelError::OutOfRange`] if `percent` exceeds
    /// `100`.
    pub const fn new(percent: u8) -> Result<Self, BatteryLevelError> {
        if percent > BATTERY_LEVEL_MAX {
            return Err(BatteryLevelError::OutOfRange {
                got: percent,
                min: BATTERY_LEVEL_MIN,
                max: BATTERY_LEVEL_MAX,
            });
        }
        Ok(Self(percent))
    }

    /// The stored percentage.
    #[must_use]
    pub const fn percent(self) -> u8 {
        self.0
    }
}

impl fmt::Display for BatteryLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0)
    }
}

impl TryFrom<u8> for BatteryLevel {
    type Error = BatteryLevelError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
