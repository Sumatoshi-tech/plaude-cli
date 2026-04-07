//! Tests for `BleDiscovery::scan` and its injected `ScanProvider`.
//!
//! Journey: specs/plaude-cli-v1/journeys/M05-transport-ble.md

use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use plaud_domain::{DeviceCandidate, TransportHint};
use plaud_transport::{DeviceDiscovery, Error, Result};
use plaud_transport_ble::{BleDiscovery, ScanProvider};

const EXPECTED_LOCAL_NAME: &str = "PLAUD_NOTE";
const NORDIC_MANUFACTURER_ID: u16 = 0x0059;
const EXPECTED_RSSI_DBM: i16 = -70;

struct FakeProvider;

#[async_trait]
impl ScanProvider for FakeProvider {
    async fn scan(&self, _timeout: Duration) -> Result<Vec<DeviceCandidate>> {
        Ok(vec![DeviceCandidate {
            local_name: EXPECTED_LOCAL_NAME.to_owned(),
            manufacturer_id: NORDIC_MANUFACTURER_ID,
            rssi_dbm: Some(EXPECTED_RSSI_DBM),
            transport_hint: TransportHint::Ble,
        }])
    }
}

#[tokio::test]
async fn scan_returns_candidates_from_the_injected_provider() {
    let discovery = BleDiscovery::new(Arc::new(FakeProvider));
    let candidates = discovery.scan(Duration::from_millis(1)).await.expect("scan");
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].local_name, EXPECTED_LOCAL_NAME);
    assert_eq!(candidates[0].manufacturer_id, NORDIC_MANUFACTURER_ID);
}

#[tokio::test]
async fn connect_is_not_wired_in_m5_and_returns_unsupported() {
    let discovery = BleDiscovery::new(Arc::new(FakeProvider));
    let candidate = discovery.scan(Duration::from_millis(1)).await.expect("scan").remove(0);
    match discovery.connect(&candidate).await {
        Err(Error::Unsupported { .. }) => {}
        Err(other) => panic!("expected Unsupported, got {other:?}"),
        Ok(_) => panic!("expected Unsupported, got Ok(transport)"),
    }
}
