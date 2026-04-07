//! Tests for `BleSession::send_control`.
//!
//! Journey: specs/plaude-cli-v1/journeys/M05-transport-ble.md

use bytes::Bytes;
use plaud_domain::AuthToken;
use plaud_transport::Error;
use plaud_transport_ble::{BleChannel, BleSession};

const SAMPLE_TOKEN: &str = "b4b48c21074f89d287c01e9f4b1ffab7";
const AUTH_ACCEPTED: &[u8] = &[
    0x01, 0x01, 0x00, 0x00, 0x0a, 0x00, 0x03, 0x00, 0x01, 0x01, 0x00, 0x00, 0x56, 0x5f, 0x00, 0x00,
];
const GET_DEVICE_NAME_OPCODE: u16 = 0x006C;
const PLAUD_NAME_ASCII: &[u8] = b"PLAUD_NOTE";
/// The opcode the test tells `send_control` to expect. It MUST be
/// different from the one encoded in [`WRONG_OPCODE_RESPONSE`] so the
/// session surfaces a `Protocol` error.
const EXPECTED_OPCODE_FOR_MISMATCH: u16 = 0xDEAD;
const WRONG_OPCODE_RESPONSE: &[u8] = &[0x01, 0xef, 0xbe, 0xaa];

fn token() -> AuthToken {
    AuthToken::new(SAMPLE_TOKEN).expect("hand-validated")
}

async fn authenticated_session() -> (BleSession, plaud_transport_ble::TestPeer) {
    let (channel, mut peer) = BleChannel::loopback_pair();
    let mut session = BleSession::new(channel);
    // Prime auth via a background task.
    let peer_handle = tokio::spawn(async move {
        let _ = peer.receive().await;
        peer.send(Bytes::from_static(AUTH_ACCEPTED)).await.expect("auth send");
        peer
    });
    session.authenticate(&token()).await.expect("auth");
    let peer = peer_handle.await.expect("peer task joined");
    (session, peer)
}

#[tokio::test]
async fn send_control_returns_payload_when_opcode_matches() {
    let (mut session, mut peer) = authenticated_session().await;
    // Build a GetDeviceName response: 01 6C 00 <PLAUD_NOTE bytes padded with zeros>
    let mut response = Vec::from([0x01, 0x6c, 0x00]);
    response.extend_from_slice(PLAUD_NAME_ASCII);
    response.extend_from_slice(&[0x00; 20]);
    let response_bytes = Bytes::from(response);
    tokio::spawn(async move {
        let _ = peer.receive().await;
        peer.send(response_bytes).await.expect("send");
    });
    let frame = plaud_proto::encode::device::get_device_name();
    let payload = session.send_control(frame, GET_DEVICE_NAME_OPCODE).await.expect("ok");
    assert!(payload.starts_with(PLAUD_NAME_ASCII));
}

#[tokio::test]
async fn send_control_without_auth_returns_auth_required() {
    let (channel, _peer) = BleChannel::loopback_pair();
    let mut session = BleSession::new(channel);
    let frame = plaud_proto::encode::device::get_device_name();
    let err = session.send_control(frame, GET_DEVICE_NAME_OPCODE).await.unwrap_err();
    assert!(matches!(err, Error::AuthRequired));
}

#[tokio::test]
async fn send_control_returns_protocol_error_on_opcode_mismatch() {
    let (mut session, mut peer) = authenticated_session().await;
    tokio::spawn(async move {
        let _ = peer.receive().await;
        peer.send(Bytes::from_static(WRONG_OPCODE_RESPONSE)).await.expect("send");
    });
    let frame = plaud_proto::encode::device::get_device_name();
    let err = session.send_control(frame, EXPECTED_OPCODE_FOR_MISMATCH).await.unwrap_err();
    assert!(matches!(err, Error::Protocol(_)));
}
