//! Tests for `SimDiscovery::scan`.
//!
//! Journey: specs/plaude-cli-v1/journeys/M03-sim-v0.md

use std::time::Duration;

use plaud_domain::{AuthToken, TransportHint};
use plaud_sim::SimDevice;
use plaud_transport::DeviceDiscovery;

const TOKEN: &str = "00000000000000000000000000000000";
const NORDIC_MANUFACTURER_ID: u16 = 0x0059;
const SCAN_WINDOW_MS: u64 = 1;

fn token() -> AuthToken {
    AuthToken::new(TOKEN).expect("hand-validated")
}

#[tokio::test]
async fn scan_returns_exactly_one_plaud_note_candidate() {
    let sim = SimDevice::builder().build();
    let discovery = sim.discovery(token());
    let candidates = discovery.scan(Duration::from_millis(SCAN_WINDOW_MS)).await.expect("scan");
    assert_eq!(candidates.len(), 1);
    let candidate = &candidates[0];
    assert_eq!(candidate.local_name, "PLAUD_NOTE");
    assert_eq!(candidate.manufacturer_id, NORDIC_MANUFACTURER_ID);
    assert_eq!(candidate.transport_hint, TransportHint::Ble);
    assert!(candidate.rssi_dbm.is_some());
}
