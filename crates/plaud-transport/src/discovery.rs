//! The `DeviceDiscovery` trait — scanning for reachable devices and
//! turning a discovery result into a live [`Transport`].

use std::time::Duration;

use async_trait::async_trait;
use plaud_domain::DeviceCandidate;

use crate::{error::Result, transport::Transport};

/// Scanning and connecting. Implemented by every transport crate.
#[async_trait]
pub trait DeviceDiscovery: Send + Sync {
    /// Scan for reachable devices for up to `timeout`. Returns every
    /// candidate observed during the scan window, deduplicated on the
    /// implementation's notion of identity (local name for BLE,
    /// mount path for USB, SSID for Wi-Fi).
    async fn scan(&self, timeout: Duration) -> Result<Vec<DeviceCandidate>>;

    /// Connect to a previously discovered candidate and return a live
    /// [`Transport`] for issuing commands.
    async fn connect(&self, candidate: &DeviceCandidate) -> Result<Box<dyn Transport>>;
}
