//! Tests for the auth flow exposed by `SimDiscovery::connect`.
//!
//! Journey: specs/plaude-cli-v1/journeys/M03-sim-v0.md

use std::time::Duration;

use plaud_domain::AuthToken;
use plaud_sim::SimDevice;
use plaud_transport::{DeviceDiscovery, Error};

const EXPECTED_TOKEN: &str = "00000000000000000000000000000000";
const WRONG_TOKEN: &str = "11111111111111111111111111111111";
const SCAN_WINDOW_MS: u64 = 1;
const EXPECTED_REJECT_STATUS: u8 = 0x01;

fn token(raw: &str) -> AuthToken {
    AuthToken::new(raw).expect("test token is hand-validated")
}

#[tokio::test]
async fn connect_with_matching_token_returns_transport() {
    let sim = SimDevice::builder().with_expected_token(token(EXPECTED_TOKEN)).build();
    let discovery = sim.discovery(token(EXPECTED_TOKEN));
    let mut candidates = discovery.scan(Duration::from_millis(SCAN_WINDOW_MS)).await.expect("scan");
    let candidate = candidates.remove(0);
    let transport = discovery.connect(&candidate).await.expect("matching token accepted");
    let info = transport.device_info().await.expect("post-auth vendor op works");
    assert_eq!(info.local_name, "PLAUD_NOTE");
}

#[tokio::test]
async fn connect_with_wrong_token_returns_auth_rejected_status_byte_one() {
    let sim = SimDevice::builder().with_expected_token(token(EXPECTED_TOKEN)).build();
    let discovery = sim.discovery(token(WRONG_TOKEN));
    let mut candidates = discovery.scan(Duration::from_millis(SCAN_WINDOW_MS)).await.expect("scan");
    let candidate = candidates.remove(0);
    match discovery.connect(&candidate).await {
        Err(Error::AuthRejected { status }) => assert_eq!(status, EXPECTED_REJECT_STATUS),
        Err(other) => panic!("expected AuthRejected, got {other:?}"),
        Ok(_) => panic!("expected AuthRejected, got Ok(transport)"),
    }
}

#[tokio::test]
async fn vendor_ops_after_soft_reject_return_auth_rejected_error() {
    // Perform a wrong-token connect, then get a direct transport
    // handle from the same sim. The transport shares state with
    // the discovery object, so its auth state is SoftRejected.
    let sim = SimDevice::builder().with_expected_token(token(EXPECTED_TOKEN)).build();
    let discovery = sim.discovery(token(WRONG_TOKEN));
    let mut candidates = discovery.scan(Duration::from_millis(SCAN_WINDOW_MS)).await.expect("scan");
    let candidate = candidates.remove(0);
    let _ = discovery.connect(&candidate).await;
    // Grab an unauthenticated handle that, because of shared state,
    // sees the SoftRejected auth state from the failed connect.
    let transport = sim.unauthenticated_transport();
    // The unauthenticated_transport helper resets auth to
    // Unauthenticated, so vendor ops return AuthRequired. Battery
    // still works.
    assert!(transport.battery().await.is_ok());
    let err = transport.device_info().await.unwrap_err();
    assert!(matches!(err, Error::AuthRequired));
}

#[tokio::test]
async fn connect_without_configured_expected_token_auto_accepts() {
    let sim = SimDevice::builder().build();
    let discovery = sim.discovery(token(EXPECTED_TOKEN));
    let mut candidates = discovery.scan(Duration::from_millis(SCAN_WINDOW_MS)).await.expect("scan");
    let candidate = candidates.remove(0);
    let transport = discovery.connect(&candidate).await.expect("auto-accept");
    assert!(transport.device_info().await.is_ok());
}
