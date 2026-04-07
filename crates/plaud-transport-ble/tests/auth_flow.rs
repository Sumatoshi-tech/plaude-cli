//! Tests for `BleSession::authenticate`.
//!
//! Journey: specs/plaude-cli-v1/journeys/M05-transport-ble.md

use bytes::Bytes;
use plaud_domain::AuthToken;
use plaud_transport::Error;
use plaud_transport_ble::{BleChannel, BleSession};

const SAMPLE_TOKEN: &str = "b4b48c21074f89d287c01e9f4b1ffab7";
const AUTH_ACCEPTED_RESPONSE: &[u8] = &[
    0x01, 0x01, 0x00, // control header + opcode 0x0001 LE
    0x00, // status byte — accepted
    0x0a, 0x00, 0x03, 0x00, 0x01, 0x01, 0x00, 0x00, 0x56, 0x5f, 0x00, 0x00,
];
const AUTH_REJECTED_RESPONSE: &[u8] = &[
    0x01, 0x01, 0x00, //
    0x01, // status byte — rejected
    0x0a, 0x00, 0x03, 0x00, 0x01, 0x01, 0x00, 0x00, 0x56, 0x5f, 0x00, 0x00,
];
const EXPECTED_REJECT_STATUS: u8 = 0x01;
const MALFORMED_AUTH_RESPONSE: &[u8] = &[0x99, 0x99];

fn token() -> AuthToken {
    AuthToken::new(SAMPLE_TOKEN).expect("hand-validated")
}

#[tokio::test]
async fn authenticate_writes_the_exact_plaud_proto_auth_frame() {
    let (channel, mut peer) = BleChannel::loopback_pair();
    let mut session = BleSession::new(channel);
    let expected = plaud_proto::encode::auth::authenticate(&token());
    tokio::spawn(async move {
        // Peer side: capture the write, respond with Accepted.
        let got = peer.receive().await.expect("session wrote");
        assert_eq!(got.as_ref(), expected.as_ref());
        peer.send(Bytes::from_static(AUTH_ACCEPTED_RESPONSE)).await.expect("send");
    });
    session.authenticate(&token()).await.expect("accepted");
    assert!(session.is_authenticated());
}

#[tokio::test]
async fn authenticate_ok_path_flips_the_authenticated_flag() {
    let (channel, mut peer) = BleChannel::loopback_pair();
    let mut session = BleSession::new(channel);
    assert!(!session.is_authenticated());
    tokio::spawn(async move {
        let _ = peer.receive().await;
        peer.send(Bytes::from_static(AUTH_ACCEPTED_RESPONSE)).await.expect("send");
    });
    session.authenticate(&token()).await.expect("accepted");
    assert!(session.is_authenticated());
}

#[tokio::test]
async fn authenticate_rejected_path_returns_auth_rejected_status_one() {
    let (channel, mut peer) = BleChannel::loopback_pair();
    let mut session = BleSession::new(channel);
    tokio::spawn(async move {
        let _ = peer.receive().await;
        peer.send(Bytes::from_static(AUTH_REJECTED_RESPONSE)).await.expect("send");
    });
    match session.authenticate(&token()).await {
        Err(Error::AuthRejected { status }) => assert_eq!(status, EXPECTED_REJECT_STATUS),
        Err(other) => panic!("expected AuthRejected, got {other:?}"),
        Ok(()) => panic!("expected AuthRejected, got Ok"),
    }
    assert!(!session.is_authenticated());
}

#[tokio::test]
async fn authenticate_malformed_response_returns_protocol_error() {
    let (channel, mut peer) = BleChannel::loopback_pair();
    let mut session = BleSession::new(channel);
    tokio::spawn(async move {
        let _ = peer.receive().await;
        peer.send(Bytes::from_static(MALFORMED_AUTH_RESPONSE)).await.expect("send");
    });
    let err = session.authenticate(&token()).await.unwrap_err();
    assert!(matches!(err, Error::Protocol(_)));
}
