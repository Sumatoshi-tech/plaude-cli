//! Tests for `SimTransport::storage`.
//!
//! Journey: specs/plaude-cli-v1/journeys/M03-sim-v0.md

use plaud_domain::StorageStats;
use plaud_sim::SimDevice;

const TOTAL: u64 = 1_000_000;
const USED: u64 = 250_000;
const COUNT: u32 = 3;

fn configured_storage() -> StorageStats {
    StorageStats::new(TOTAL, USED, COUNT).expect("hand-validated storage config")
}

#[tokio::test]
async fn storage_returns_the_configured_stats() {
    let sim = SimDevice::builder().with_storage(configured_storage()).build();
    let transport = sim.authenticated_transport();
    let stats = transport.storage().await.expect("authenticated");
    assert_eq!(stats.total_bytes(), TOTAL);
    assert_eq!(stats.used_bytes(), USED);
    assert_eq!(stats.recording_count(), COUNT);
}

#[tokio::test]
async fn storage_defaults_to_zero_when_not_configured() {
    let sim = SimDevice::builder().build();
    let transport = sim.authenticated_transport();
    let stats = transport.storage().await.expect("authenticated");
    assert_eq!(stats, StorageStats::ZERO);
}

#[tokio::test]
async fn storage_requires_authentication() {
    let sim = SimDevice::builder().with_storage(configured_storage()).build();
    let transport = sim.unauthenticated_transport();
    let err = transport.storage().await.unwrap_err();
    assert!(matches!(err, plaud_transport::Error::AuthRequired));
}
