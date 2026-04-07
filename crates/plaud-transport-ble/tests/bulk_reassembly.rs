//! Tests for `BulkReassembler` and `BleSession::read_bulk`.
//!
//! Journey: specs/plaude-cli-v1/journeys/M05-transport-ble.md

use bytes::{BufMut, Bytes, BytesMut};
use plaud_domain::AuthToken;
use plaud_proto::Frame;
use plaud_transport::Error;
use plaud_transport_ble::{BleChannel, BleSession, BulkReassembler, FeedStatus};

const SAMPLE_TOKEN: &str = "b4b48c21074f89d287c01e9f4b1ffab7";
const AUTH_ACCEPTED: &[u8] = &[
    0x01, 0x01, 0x00, 0x00, 0x0a, 0x00, 0x03, 0x00, 0x01, 0x01, 0x00, 0x00, 0x56, 0x5f, 0x00, 0x00,
];
const FILE_ID: u32 = 0x1234_5678;
const CHUNK_BYTES: usize = 80;
const BULK_MAGIC: u8 = 0x02;
const BULK_END_SENTINEL: u32 = 0xFFFF_FFFF;

fn token() -> AuthToken {
    AuthToken::new(SAMPLE_TOKEN).expect("hand-validated")
}

fn bulk_frame_bytes(file_id: u32, offset: u32, payload: &[u8]) -> Bytes {
    let mut buf = BytesMut::with_capacity(10 + payload.len());
    buf.put_u8(BULK_MAGIC);
    buf.put_u32_le(file_id);
    buf.put_u32_le(offset);
    buf.put_u8(payload.len() as u8);
    buf.put_slice(payload);
    buf.freeze()
}

#[tokio::test]
async fn reassembler_accepts_two_sequential_chunks_and_terminator() {
    let mut r = BulkReassembler::new();
    let chunk_a = vec![0xAA; CHUNK_BYTES];
    let chunk_b = vec![0xBB; CHUNK_BYTES];
    let status_a = r.feed(Frame::Bulk {
        file_id: FILE_ID,
        offset: 0,
        payload: Bytes::from(chunk_a.clone()),
    });
    assert_eq!(status_a.expect("ok"), FeedStatus::InProgress);
    let status_b = r.feed(Frame::Bulk {
        file_id: FILE_ID,
        offset: CHUNK_BYTES as u32,
        payload: Bytes::from(chunk_b.clone()),
    });
    assert_eq!(status_b.expect("ok"), FeedStatus::InProgress);
    let status_end = r.feed(Frame::BulkEnd {
        file_id: FILE_ID,
        payload: Bytes::new(),
    });
    assert_eq!(status_end.expect("ok"), FeedStatus::Done);
    let bytes = r.finish().expect("done");
    let mut expected = chunk_a;
    expected.extend_from_slice(&chunk_b);
    assert_eq!(bytes, expected);
}

#[tokio::test]
async fn reassembler_rejects_non_monotone_offset() {
    let mut r = BulkReassembler::new();
    let chunk = vec![0xCC; CHUNK_BYTES];
    r.feed(Frame::Bulk {
        file_id: FILE_ID,
        offset: CHUNK_BYTES as u32 * 2,
        payload: Bytes::from(chunk.clone()),
    })
    .expect("first chunk ok");
    let err = r
        .feed(Frame::Bulk {
            file_id: FILE_ID,
            offset: 0, // backwards
            payload: Bytes::from(chunk),
        })
        .unwrap_err();
    assert!(matches!(err, Error::Protocol(_)));
}

#[tokio::test]
async fn reassembler_rejects_mismatched_file_id() {
    let mut r = BulkReassembler::new();
    let chunk = vec![0xDD; CHUNK_BYTES];
    r.feed(Frame::Bulk {
        file_id: FILE_ID,
        offset: 0,
        payload: Bytes::from(chunk.clone()),
    })
    .expect("first chunk ok");
    let err = r
        .feed(Frame::Bulk {
            file_id: FILE_ID + 1,
            offset: CHUNK_BYTES as u32,
            payload: Bytes::from(chunk),
        })
        .unwrap_err();
    assert!(matches!(err, Error::Protocol(_)));
}

#[tokio::test]
async fn reassembler_finish_errors_without_bulk_end() {
    let mut r = BulkReassembler::new();
    r.feed(Frame::Bulk {
        file_id: FILE_ID,
        offset: 0,
        payload: Bytes::from_static(&[0xEE]),
    })
    .expect("feed ok");
    let err = r.finish().unwrap_err();
    assert!(matches!(err, Error::Protocol(_)));
}

#[tokio::test]
async fn read_bulk_end_to_end_over_loopback() {
    let (channel, mut peer) = BleChannel::loopback_pair();
    let mut session = BleSession::new(channel);
    // Auth
    let auth_task = tokio::spawn(async move {
        let _ = peer.receive().await;
        peer.send(Bytes::from_static(AUTH_ACCEPTED)).await.expect("auth");
        peer
    });
    session.authenticate(&token()).await.expect("auth");
    let mut peer = auth_task.await.expect("join");

    // Feed the session a bulk-trigger and respond with two chunks + end.
    let chunk_a = vec![0x11; CHUNK_BYTES];
    let chunk_b = vec![0x22; CHUNK_BYTES];
    let chunk_a_clone = chunk_a.clone();
    let chunk_b_clone = chunk_b.clone();
    tokio::spawn(async move {
        let _ = peer.receive().await;
        peer.send(bulk_frame_bytes(FILE_ID, 0, &chunk_a_clone)).await.expect("frame a");
        peer.send(bulk_frame_bytes(FILE_ID, CHUNK_BYTES as u32, &chunk_b_clone))
            .await
            .expect("frame b");
        peer.send(bulk_frame_bytes(FILE_ID, BULK_END_SENTINEL, &[])).await.expect("end");
    });
    let trigger = plaud_proto::encode::file::read_file_chunk(FILE_ID, 0, (CHUNK_BYTES * 2) as u32);
    let bytes = session.read_bulk(trigger).await.expect("bulk read");
    let mut expected = chunk_a;
    expected.extend_from_slice(&chunk_b);
    assert_eq!(bytes, expected);
}
