//! `BleDiscovery` — the M5 [`plaud_transport::DeviceDiscovery`]
//! implementation.
//!
//! Scanning is delegated to an injectable [`ScanProvider`] trait so
//! tests can supply a deterministic candidate list. The production
//! btleplug-backed provider lives in the `backend::btleplug` module
//! behind the `btleplug-backend` cargo feature.

use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use plaud_domain::DeviceCandidate;
use plaud_transport::{DeviceDiscovery, Error, Result, Transport};

/// Abstraction over "how do I enumerate reachable Plaud devices".
#[async_trait]
pub trait ScanProvider: Send + Sync {
    /// Return all device candidates visible within `timeout`.
    async fn scan(&self, timeout: Duration) -> Result<Vec<DeviceCandidate>>;
}

/// Error message for attempts to use the M5 discovery's `connect`
/// endpoint, which is deliberately not wired up until M8 (bootstrap)
/// gives us a credential-source-aware factory.
const ERR_CONNECT_NOT_WIRED: &str =
    "BleDiscovery::connect is not wired in M5 — use BleTransport::from_parts directly for tests or call the btleplug backend explicitly";

/// [`DeviceDiscovery`] implementation backed by an injectable
/// [`ScanProvider`]. M5 uses this in tests and as the scaffold the
/// btleplug backend will slot into.
pub struct BleDiscovery {
    provider: Arc<dyn ScanProvider>,
}

impl BleDiscovery {
    /// Build a discovery from a scan provider.
    #[must_use]
    pub fn new(provider: Arc<dyn ScanProvider>) -> Self {
        Self { provider }
    }
}

impl std::fmt::Debug for BleDiscovery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BleDiscovery").finish_non_exhaustive()
    }
}

#[async_trait]
impl DeviceDiscovery for BleDiscovery {
    async fn scan(&self, timeout: Duration) -> Result<Vec<DeviceCandidate>> {
        self.provider.scan(timeout).await
    }

    async fn connect(&self, _candidate: &DeviceCandidate) -> Result<Box<dyn Transport>> {
        Err(Error::Unsupported {
            capability: ERR_CONNECT_NOT_WIRED,
        })
    }
}
