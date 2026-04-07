//! Tests for `set_privacy`.
//!
//! Journey: specs/plaude-cli-v1/journeys/M03-sim-v0.md

use plaud_sim::SimDevice;

#[tokio::test]
async fn set_privacy_on_and_off_both_succeed_when_authenticated() {
    let sim = SimDevice::builder().build();
    let transport = sim.authenticated_transport();
    transport.set_privacy(true).await.expect("privacy on");
    transport.set_privacy(false).await.expect("privacy off");
}

#[tokio::test]
async fn set_privacy_requires_authentication() {
    let sim = SimDevice::builder().build();
    let transport = sim.unauthenticated_transport();
    let err = transport.set_privacy(true).await.unwrap_err();
    assert!(matches!(err, plaud_transport::Error::AuthRequired));
}
