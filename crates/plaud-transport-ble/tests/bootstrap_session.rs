//! Tests for `LoopbackBootstrap` and `BootstrapSession::run`.
//!
//! Journey: specs/plaude-cli-v1/journeys/M08-auth-bootstrap-peripheral.md

use std::time::Duration;

use bytes::Bytes;
use plaud_domain::AuthToken;
use plaud_transport_ble::{BootstrapError, LoopbackBootstrap};

const SAMPLE_TOKEN: &str = "b4b48c21074f89d287c01e9f4b1ffab7";
const MOCK_ACCEPTED_FIRST_BYTE: u8 = 0x01;
const SHORT_TIMEOUT: Duration = Duration::from_millis(50);
const GENEROUS_TIMEOUT: Duration = Duration::from_secs(2);

#[tokio::test]
async fn loopback_handshake_captures_the_token_the_phone_wrote() {
    let (session, phone) = LoopbackBootstrap::new().split();
    let token = AuthToken::new(SAMPLE_TOKEN).expect("valid");
    let wire = plaud_proto::encode::auth::authenticate(&token);
    // Hold `phone` alive for the whole handshake so the peripheral's
    // notify_out send-back to the phone does not hit a closed receiver.
    let session_task = tokio::spawn(async move { session.run(GENEROUS_TIMEOUT).await });
    phone.write(wire).await.expect("write");
    let outcome = session_task.await.expect("join").expect("captured");
    assert_eq!(outcome.token.as_str(), SAMPLE_TOKEN);
    drop(phone);
}

#[tokio::test]
async fn loopback_peripheral_sends_back_mock_accepted_notification_on_success() {
    let (session, mut phone) = LoopbackBootstrap::new().split();
    let token = AuthToken::new(SAMPLE_TOKEN).expect("valid");
    let wire = plaud_proto::encode::auth::authenticate(&token);
    let session_task = tokio::spawn(async move { session.run(GENEROUS_TIMEOUT).await });
    phone.write(wire).await.expect("write");
    let notification = phone.receive_notification().await.expect("peripheral responded");
    assert_eq!(notification[0], MOCK_ACCEPTED_FIRST_BYTE);
    session_task.await.expect("join").expect("outcome");
}

#[tokio::test]
async fn loopback_run_times_out_when_no_write_arrives() {
    let (session, _phone) = LoopbackBootstrap::new().split();
    let err = session.run(SHORT_TIMEOUT).await.unwrap_err();
    assert!(matches!(err, BootstrapError::Timeout { .. }));
}

#[tokio::test]
async fn loopback_run_reports_decode_failure_for_a_malformed_write() {
    let (session, phone) = LoopbackBootstrap::new().split();
    let session_task = tokio::spawn(async move { session.run(GENEROUS_TIMEOUT).await });
    phone.write(Bytes::from_static(&[0xAA, 0xBB])).await.expect("write");
    let err = session_task.await.expect("join").unwrap_err();
    assert!(matches!(err, BootstrapError::DecodeFailed { .. }));
    drop(phone);
}
