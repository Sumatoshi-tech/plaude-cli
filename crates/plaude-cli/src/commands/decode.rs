//! Decode raw Opus frames (as received over BLE) into a PCM WAV file.
//!
//! The Plaud device sends recordings over BLE as raw Opus frames —
//! 80 bytes each, CELT-only 20 ms, mono 16 kHz. This module decodes
//! them to standard PCM WAV (16-bit, mono, 16 kHz).

use opus_decoder::OpusDecoder;

/// Number of samples per Opus frame at 16 kHz / 20 ms.
const SAMPLES_PER_FRAME: usize = 320;

/// Sample rate for Plaud recordings.
const SAMPLE_RATE: u32 = 16_000;

/// Bits per PCM sample.
const BITS_PER_SAMPLE: u16 = 16;

/// Mono channel count.
const NUM_CHANNELS: u16 = 1;

/// Size of each raw Opus frame from the BLE bulk transfer.
const OPUS_FRAME_SIZE: usize = 80;

/// Decode raw Opus frames into a WAV byte vector.
///
/// `raw_opus` is the concatenated 80-byte Opus frames from the BLE
/// bulk transfer. Returns a complete WAV file (RIFF header + PCM data)
/// or an error description.
pub(crate) fn opus_to_wav(raw_opus: &[u8]) -> Result<Vec<u8>, String> {
    let mut decoder = OpusDecoder::new(SAMPLE_RATE, NUM_CHANNELS as usize).map_err(|e| format!("opus init: {e}"))?;

    let frame_count = raw_opus.len() / OPUS_FRAME_SIZE;
    let mut pcm_samples: Vec<i16> = Vec::with_capacity(frame_count * SAMPLES_PER_FRAME);
    let mut frame_buf = vec![0i16; SAMPLES_PER_FRAME * 2]; // generous buffer

    for (i, chunk) in raw_opus.chunks_exact(OPUS_FRAME_SIZE).enumerate() {
        let decoded = decoder
            .decode(chunk, &mut frame_buf, false)
            .map_err(|e| format!("opus decode frame {i}: {e}"))?;
        pcm_samples.extend_from_slice(&frame_buf[..decoded]);
    }

    Ok(build_wav(&pcm_samples))
}

/// Build a minimal WAV file from PCM i16 samples.
fn build_wav(samples: &[i16]) -> Vec<u8> {
    let data_size = (samples.len() * 2) as u32;
    let byte_rate = SAMPLE_RATE * u32::from(NUM_CHANNELS) * u32::from(BITS_PER_SAMPLE) / 8;
    let block_align = NUM_CHANNELS * BITS_PER_SAMPLE / 8;
    let file_size = 36 + data_size;

    let mut wav = Vec::with_capacity(44 + data_size as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&NUM_CHANNELS.to_le_bytes());
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&BITS_PER_SAMPLE.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());
    for &sample in samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }
    wav
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_wav_produces_valid_riff_header() {
        let samples = vec![0i16; 320];
        let wav = build_wav(&samples);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(&wav[36..40], b"data");
    }
}
