//! [`BatteryReader`] — abstraction over "how do I read the standard
//! SIG Battery Service".
//!
//! The M5 evidence is clear that battery reads succeed without any
//! vendor authentication, so the battery path lives outside the
//! `BleSession` state machine. Production uses a btleplug-backed
//! reader; tests use [`FixedBatteryReader`] for deterministic values.

use async_trait::async_trait;
use plaud_domain::BatteryLevel;
use plaud_transport::Result;

/// Read the current battery percentage from a connected Plaud Note.
#[async_trait]
pub trait BatteryReader: Send + Sync {
    /// Return the current battery percentage. Implementations MUST
    /// NOT require the session to be authenticated.
    async fn read_battery(&self) -> Result<BatteryLevel>;
}

/// Test-only `BatteryReader` that always returns the same value.
#[derive(Debug, Clone)]
pub struct FixedBatteryReader {
    level: BatteryLevel,
}

impl FixedBatteryReader {
    /// Construct a reader that always returns `level`.
    #[must_use]
    pub const fn new(level: BatteryLevel) -> Self {
        Self { level }
    }
}

#[async_trait]
impl BatteryReader for FixedBatteryReader {
    async fn read_battery(&self) -> Result<BatteryLevel> {
        Ok(self.level)
    }
}
