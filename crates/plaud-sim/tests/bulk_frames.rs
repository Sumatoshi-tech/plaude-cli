//! Tests for `plaud_sim::bulk::frames_for` and `serialise_bulk`.
//!
//! Journey: specs/plaude-cli-v1/journeys/M03-sim-v0.md

use plaud_proto::Frame;
use plaud_sim::bulk::{frames_for, serialise_bulk};

const FILE_ID: u32 = 0x1234_5678;
const START_OFFSET: u32 = 0;
const CHUNK_BYTES: usize = 80;
const BULK_END_SENTINEL: u32 = 0xFFFF_FFFF;

fn payload_of_size(n: usize) -> Vec<u8> {
    (0..n).map(|i| (i & 0xff) as u8).collect()
}

#[tokio::test]
async fn frames_for_empty_data_produces_only_a_terminating_bulk_end() {
    let frames = frames_for(FILE_ID, START_OFFSET, &[]);
    assert_eq!(frames.len(), 1);
    match &frames[0] {
        Frame::BulkEnd { file_id, .. } => assert_eq!(*file_id, FILE_ID),
        other => panic!("expected BulkEnd, got {other:?}"),
    }
}

#[tokio::test]
async fn frames_for_one_full_chunk_produces_one_bulk_plus_bulk_end() {
    let data = payload_of_size(CHUNK_BYTES);
    let frames = frames_for(FILE_ID, START_OFFSET, &data);
    assert_eq!(frames.len(), 2);
    match &frames[0] {
        Frame::Bulk { file_id, offset, payload } => {
            assert_eq!(*file_id, FILE_ID);
            assert_eq!(*offset, START_OFFSET);
            assert_eq!(payload.as_ref(), data.as_slice());
        }
        other => panic!("expected Bulk, got {other:?}"),
    }
    matches!(frames[1], Frame::BulkEnd { .. });
}

#[tokio::test]
async fn frames_for_two_and_a_half_chunks_rounds_up_and_steps_offsets() {
    let total_bytes = CHUNK_BYTES * 2 + CHUNK_BYTES / 2;
    let data = payload_of_size(total_bytes);
    let frames = frames_for(FILE_ID, START_OFFSET, &data);
    // 3 data frames + 1 end = 4
    assert_eq!(frames.len(), 4);
    let offsets: Vec<u32> = frames
        .iter()
        .filter_map(|f| match f {
            Frame::Bulk { offset, .. } => Some(*offset),
            _ => None,
        })
        .collect();
    assert_eq!(offsets, vec![0, CHUNK_BYTES as u32, (CHUNK_BYTES * 2) as u32]);
}

#[tokio::test]
async fn serialise_bulk_then_parse_notification_round_trips() {
    let data = payload_of_size(CHUNK_BYTES);
    let frames = frames_for(FILE_ID, START_OFFSET, &data);
    for frame in &frames {
        let wire = serialise_bulk(frame);
        if wire.is_empty() {
            // serialise_bulk returns empty for non-bulk variants;
            // our fixtures only contain Bulk/BulkEnd so this is unreachable.
            panic!("unexpected non-bulk frame in fixture: {frame:?}");
        }
        let parsed = plaud_proto::parse_notification(wire).expect("round-trip parse");
        match (frame, &parsed) {
            (
                Frame::Bulk {
                    file_id: a, offset: ao, ..
                },
                Frame::Bulk {
                    file_id: b, offset: bo, ..
                },
            ) => {
                assert_eq!(a, b);
                assert_eq!(ao, bo);
            }
            (Frame::BulkEnd { file_id: a, .. }, Frame::BulkEnd { file_id: b, .. }) => {
                assert_eq!(a, b);
            }
            _ => panic!("round-trip mismatch: {frame:?} vs {parsed:?}"),
        }
    }
}

#[tokio::test]
async fn bulk_end_frame_serialises_with_ffffffff_offset() {
    let frames = frames_for(FILE_ID, START_OFFSET, &[]);
    let wire = serialise_bulk(&frames[0]);
    // bytes 5..9 are the u32 LE offset; expect 0xFFFFFFFF.
    assert_eq!(&wire[5..9], &BULK_END_SENTINEL.to_le_bytes());
}
