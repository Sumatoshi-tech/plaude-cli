//! In-place WAV `pad ` chunk sanitiser.
//!
//! Every Plaud WAV carries a 460-byte `pad ` RIFF chunk whose first
//! 22 bytes are `SN:<18-digit-serial>\0`. `docs/protocol/file-formats.md`
//! calls this out as a forensic watermark: uploading a recording
//! anywhere leaks the device serial.
//!
//! `WavSanitiser::sanitise` overwrites the SN region with zeros
//! **in place**. Size, offsets, and every other byte are preserved
//! so audio decoders see a byte-equal file apart from the 22
//! sanitised bytes.

use thiserror::Error;

use crate::constants::{WAV_SN_REGION_END, WAV_SN_REGION_START};

/// Byte count of the sanitised region. Derived from the two
/// region-offset constants so any future adjustment propagates.
const SANITISED_BYTES: usize = WAV_SN_REGION_END - WAV_SN_REGION_START;

/// RIFF chunk-id expected at file offset 0.
const RIFF_MAGIC: &[u8; 4] = b"RIFF";

/// WAVE form marker expected at file offset 8.
const WAVE_MAGIC: &[u8; 4] = b"WAVE";

/// Minimum number of bytes required before [`WavSanitiser::sanitise`]
/// will touch a buffer. We need everything up to and including the
/// SN region.
const MIN_WAV_LENGTH: usize = WAV_SN_REGION_END;

/// Errors emitted by [`WavSanitiser::sanitise`].
#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum WavSanitiseError {
    /// Buffer was shorter than the bytes required to reach the SN
    /// region.
    #[error("wav buffer too short to sanitise: got {got} bytes, need at least {need}")]
    TooShort {
        /// Actual buffer length.
        got: usize,
        /// Minimum length required.
        need: usize,
    },
    /// First four bytes were not `RIFF`.
    #[error("input does not start with RIFF magic")]
    NotRiff,
    /// Bytes 8..12 were not `WAVE`.
    #[error("input RIFF form is not WAVE")]
    NotWave,
}

/// Stateless helper around the sanitise operation. Defined as a
/// struct (rather than a free function) so future versions can
/// carry configuration such as "also zero trailing metadata" once
/// we understand the rest of the `pad ` chunk.
#[derive(Debug, Default, Clone, Copy)]
pub struct WavSanitiser;

impl WavSanitiser {
    /// Construct a new sanitiser.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Overwrite the `pad ` chunk SN region with zeros. Returns the
    /// number of bytes that were modified (always [`SANITISED_BYTES`]
    /// on success).
    ///
    /// # Errors
    ///
    /// Returns [`WavSanitiseError`] variants for buffers that are
    /// too short or that do not carry the expected RIFF/WAVE magic.
    pub fn sanitise(&self, buf: &mut [u8]) -> Result<usize, WavSanitiseError> {
        if buf.len() < MIN_WAV_LENGTH {
            return Err(WavSanitiseError::TooShort {
                got: buf.len(),
                need: MIN_WAV_LENGTH,
            });
        }
        if &buf[..RIFF_MAGIC.len()] != RIFF_MAGIC {
            return Err(WavSanitiseError::NotRiff);
        }
        if &buf[8..8 + WAVE_MAGIC.len()] != WAVE_MAGIC {
            return Err(WavSanitiseError::NotWave);
        }
        for byte in &mut buf[WAV_SN_REGION_START..WAV_SN_REGION_END] {
            *byte = 0x00;
        }
        Ok(SANITISED_BYTES)
    }
}

#[cfg(test)]
mod tests {
    use super::{SANITISED_BYTES, WAV_SN_REGION_END, WAV_SN_REGION_START, WavSanitiseError, WavSanitiser};

    const PROBE_BYTE: u8 = 0x5A;
    const SN_FILL_BYTE: u8 = b'9';
    const NON_SN_FILL_BYTE: u8 = 0xAA;
    const TEST_WAV_LEN: usize = 0x200;

    fn build_fixture_wav() -> Vec<u8> {
        let mut buf = vec![NON_SN_FILL_BYTE; TEST_WAV_LEN];
        buf[..4].copy_from_slice(b"RIFF");
        buf[8..12].copy_from_slice(b"WAVE");
        // Pre-fill the SN region with recognisable non-zero bytes.
        for byte in &mut buf[WAV_SN_REGION_START..WAV_SN_REGION_END] {
            *byte = SN_FILL_BYTE;
        }
        // A probe byte right after the SN region that must survive.
        buf[WAV_SN_REGION_END] = PROBE_BYTE;
        buf
    }

    #[test]
    fn sanitise_zeroes_only_the_sn_region() {
        let mut buf = build_fixture_wav();
        let count = WavSanitiser::new().sanitise(&mut buf).expect("ok");
        assert_eq!(count, SANITISED_BYTES);
        for byte in &buf[WAV_SN_REGION_START..WAV_SN_REGION_END] {
            assert_eq!(*byte, 0x00);
        }
        // Byte immediately after the SN region is untouched.
        assert_eq!(buf[WAV_SN_REGION_END], PROBE_BYTE);
    }

    #[test]
    fn sanitise_preserves_buffer_length() {
        let mut buf = build_fixture_wav();
        let original_len = buf.len();
        WavSanitiser::new().sanitise(&mut buf).expect("ok");
        assert_eq!(buf.len(), original_len);
    }

    #[test]
    fn sanitise_preserves_riff_and_wave_magic() {
        let mut buf = build_fixture_wav();
        WavSanitiser::new().sanitise(&mut buf).expect("ok");
        assert_eq!(&buf[..4], b"RIFF");
        assert_eq!(&buf[8..12], b"WAVE");
    }

    #[test]
    fn sanitise_is_idempotent() {
        let mut buf = build_fixture_wav();
        let first = WavSanitiser::new().sanitise(&mut buf).expect("first");
        let snapshot = buf.clone();
        let second = WavSanitiser::new().sanitise(&mut buf).expect("second");
        assert_eq!(first, second);
        assert_eq!(buf, snapshot);
    }

    #[test]
    fn sanitise_rejects_a_too_short_buffer() {
        let mut buf = vec![0u8; WAV_SN_REGION_START];
        let err = WavSanitiser::new().sanitise(&mut buf).unwrap_err();
        assert!(matches!(err, WavSanitiseError::TooShort { .. }));
    }

    #[test]
    fn sanitise_rejects_non_riff_input() {
        let mut buf = build_fixture_wav();
        buf[..4].copy_from_slice(b"XXXX");
        let err = WavSanitiser::new().sanitise(&mut buf).unwrap_err();
        assert!(matches!(err, WavSanitiseError::NotRiff));
    }

    #[test]
    fn sanitise_rejects_non_wave_input() {
        let mut buf = build_fixture_wav();
        buf[8..12].copy_from_slice(b"AVI ");
        let err = WavSanitiser::new().sanitise(&mut buf).unwrap_err();
        assert!(matches!(err, WavSanitiseError::NotWave));
    }
}
