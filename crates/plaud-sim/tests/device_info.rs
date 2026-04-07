//! Tests for `SimTransport::device_info`.
//!
//! Journey: specs/plaude-cli-v1/journeys/M03-sim-v0.md

use plaud_domain::DeviceInfo;
use plaud_sim::SimDevice;

#[tokio::test]
async fn device_info_returns_default_placeholder_when_not_overridden() {
    let sim = SimDevice::builder().build();
    let transport = sim.authenticated_transport();
    let info = transport.device_info().await.expect("authenticated sim");
    assert_eq!(info, DeviceInfo::placeholder());
}

#[tokio::test]
async fn device_info_returns_configured_override() {
    let custom = DeviceInfo::placeholder();
    let sim = SimDevice::builder().with_device_info(custom.clone()).build();
    let transport = sim.authenticated_transport();
    let info = transport.device_info().await.expect("authenticated sim");
    assert_eq!(info, custom);
}

#[tokio::test]
async fn device_info_requires_authentication() {
    let sim = SimDevice::builder().build();
    let transport = sim.unauthenticated_transport();
    let err = transport.device_info().await.unwrap_err();
    assert!(matches!(err, plaud_transport::Error::AuthRequired));
}
