//! Tests for [`plaud_proto::parse_notification`] bulk path, including
//! the `0xFFFFFFFF` end-of-stream sentinel.
//!
//! Journey: specs/plaude-cli-v1/journeys/M02-proto-codec.md

use bytes::{BufMut, Bytes, BytesMut};
use plaud_proto::{DecodeError, Frame, parse_notification};

const FILE_ID: u32 = 0x69D26658;
const OFFSET_ZERO: u32 = 0;
const OFFSET_EIGHTY: u32 = 80;
const END_SENTINEL: u32 = 0xFFFF_FFFF;
const PAYLOAD_BYTE: u8 = 0xAB;
const PAYLOAD_LEN: usize = 80;
const BULK_HEADER_LEN: usize = 10;

fn build_bulk_frame(file_id: u32, offset: u32, payload: &[u8]) -> Bytes {
    let mut buf = BytesMut::with_capacity(BULK_HEADER_LEN + payload.len());
    buf.put_u8(0x02); // FRAME_TYPE_BULK
    buf.put_u32_le(file_id);
    buf.put_u32_le(offset);
    buf.put_u8(payload.len() as u8); // chunk_len
    buf.put_slice(payload);
    buf.freeze()
}

#[test]
fn parses_a_bulk_data_frame_into_its_fields() {
    let payload = [PAYLOAD_BYTE; PAYLOAD_LEN];
    let raw = build_bulk_frame(FILE_ID, OFFSET_ZERO, &payload);
    let frame = parse_notification(raw).expect("parses");
    match frame {
        Frame::Bulk {
            file_id,
            offset,
            payload: got,
        } => {
            assert_eq!(file_id, FILE_ID);
            assert_eq!(offset, OFFSET_ZERO);
            assert_eq!(got.len(), PAYLOAD_LEN);
            assert_eq!(got[0], PAYLOAD_BYTE);
        }
        other => panic!("expected Bulk, got {other:?}"),
    }
}

#[test]
fn parses_a_second_bulk_frame_with_offset_80() {
    let payload = [PAYLOAD_BYTE; PAYLOAD_LEN];
    let raw = build_bulk_frame(FILE_ID, OFFSET_EIGHTY, &payload);
    let frame = parse_notification(raw).expect("parses");
    match frame {
        Frame::Bulk { offset, .. } => assert_eq!(offset, OFFSET_EIGHTY),
        other => panic!("expected Bulk, got {other:?}"),
    }
}

#[test]
fn ffffffff_offset_is_decoded_as_bulk_end_not_bulk() {
    let payload = [PAYLOAD_BYTE; PAYLOAD_LEN];
    let raw = build_bulk_frame(FILE_ID, END_SENTINEL, &payload);
    let frame = parse_notification(raw).expect("parses");
    match frame {
        Frame::BulkEnd { file_id, .. } => assert_eq!(file_id, FILE_ID),
        other => panic!("expected BulkEnd, got {other:?}"),
    }
}

#[test]
fn bulk_frame_shorter_than_header_returns_too_short() {
    // Magic byte present but header truncated to 5 bytes.
    let raw = Bytes::from_static(&[0x02, 0x58, 0x66, 0xd2, 0x69]);
    let err = parse_notification(raw).unwrap_err();
    assert!(matches!(err, DecodeError::TooShort { .. }));
}

#[test]
fn bulk_frame_with_exact_header_only_has_empty_payload() {
    let raw = build_bulk_frame(FILE_ID, OFFSET_ZERO, &[]);
    let frame = parse_notification(raw).expect("parses");
    match frame {
        Frame::Bulk { payload, .. } => assert!(payload.is_empty()),
        other => panic!("expected Bulk, got {other:?}"),
    }
}
