//! Tests for the recording-control state machine exposed by
//! `start_recording` / `stop_recording` / `pause_recording` /
//! `resume_recording`.
//!
//! Journey: specs/plaude-cli-v1/journeys/M03-sim-v0.md

use plaud_sim::SimDevice;
use plaud_transport::Error;

#[tokio::test]
async fn idle_to_recording_to_paused_to_recording_to_idle_happy_path() {
    let sim = SimDevice::builder().build();
    let transport = sim.authenticated_transport();
    transport.start_recording().await.expect("idle → recording");
    transport.pause_recording().await.expect("recording → paused");
    transport.resume_recording().await.expect("paused → recording");
    transport.stop_recording().await.expect("recording → idle");
}

#[tokio::test]
async fn stop_from_idle_is_rejected_as_protocol_error() {
    let sim = SimDevice::builder().build();
    let transport = sim.authenticated_transport();
    let err = transport.stop_recording().await.unwrap_err();
    assert!(matches!(err, Error::Protocol(_)));
}

#[tokio::test]
async fn pause_from_idle_is_rejected_as_protocol_error() {
    let sim = SimDevice::builder().build();
    let transport = sim.authenticated_transport();
    let err = transport.pause_recording().await.unwrap_err();
    assert!(matches!(err, Error::Protocol(_)));
}

#[tokio::test]
async fn resume_from_idle_is_rejected_as_protocol_error() {
    let sim = SimDevice::builder().build();
    let transport = sim.authenticated_transport();
    let err = transport.resume_recording().await.unwrap_err();
    assert!(matches!(err, Error::Protocol(_)));
}

#[tokio::test]
async fn start_while_already_recording_is_rejected() {
    let sim = SimDevice::builder().build();
    let transport = sim.authenticated_transport();
    transport.start_recording().await.expect("first start");
    let err = transport.start_recording().await.unwrap_err();
    assert!(matches!(err, Error::Protocol(_)));
}

#[tokio::test]
async fn stop_from_paused_returns_to_idle() {
    let sim = SimDevice::builder().build();
    let transport = sim.authenticated_transport();
    transport.start_recording().await.expect("start");
    transport.pause_recording().await.expect("pause");
    transport.stop_recording().await.expect("stop from paused");
    let err = transport.stop_recording().await.unwrap_err();
    assert!(matches!(err, Error::Protocol(_)));
}
