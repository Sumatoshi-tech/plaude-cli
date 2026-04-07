//! Recordings — stable identifier, kind, and top-level `Recording` struct.
//!
//! The wire-level recording path scheme is specified in
//! [`docs/protocol/file-formats.md`](../../../docs/protocol/file-formats.md):
//!
//! ```text
//! /{NOTES,CALLS}/<YYYYMMDD>/<unix_seconds>.{WAV,ASR}
//! ```
//!
//! The basename — a Unix epoch timestamp in seconds — is the stable
//! identifier the CLI uses to refer to a recording across transports.

use std::{fmt, str::FromStr};

use thiserror::Error;

/// Minimum length of a valid [`RecordingId`] string.
///
/// A 9-digit minimum covers Unix epoch timestamps from 2001-09-09 onwards,
/// which comfortably predates the Plaud Note's existence.
const RECORDING_ID_MIN_LEN: usize = 9;

/// Maximum length of a valid [`RecordingId`] string.
///
/// `i64::MAX` in base 10 is 19 characters; any longer input cannot be
/// a valid Unix timestamp and is rejected early.
const RECORDING_ID_MAX_LEN: usize = 19;

/// Validation errors produced by [`RecordingId::new`] and its [`FromStr`] impl.
#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum RecordingIdError {
    /// Input was the empty string.
    #[error("recording id must not be empty")]
    Empty,
    /// Input contained a byte that is not an ASCII decimal digit.
    #[error("recording id must be ASCII digits only")]
    NonDigit,
    /// Input length was outside the valid range.
    #[error("recording id length {got} is outside the valid range [{min}..={max}]")]
    InvalidLength {
        /// Observed length.
        got: usize,
        /// Inclusive minimum.
        min: usize,
        /// Inclusive maximum.
        max: usize,
    },
    /// Input parsed successfully but did not fit in `i64`.
    #[error("recording id {got:?} is not representable as a signed 64-bit integer")]
    OutOfRange {
        /// The offending input.
        got: String,
    },
}

/// Stable identifier for a recording: the Unix-epoch-seconds basename
/// of the `.WAV`/`.ASR` pair on the device, e.g. `"1775393534"`.
///
/// The value carries both the original string (for round-tripping to
/// the device) and the parsed `i64` (for time arithmetic), so neither
/// accessor has to fall back to runtime parsing or risk a panic.
///
/// Evidence: `specs/re/captures/usb/2026-04-05-plaud-note-v0095-first-recording.md`.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RecordingId {
    raw: String,
    seconds: i64,
}

impl RecordingId {
    /// Construct a `RecordingId`, validating that the input is a plausible
    /// Unix epoch seconds string (ASCII digits, length in the configured
    /// range, representable as `i64`).
    ///
    /// # Errors
    ///
    /// Returns [`RecordingIdError`] with a variant describing why the input
    /// was rejected; never panics.
    pub fn new(input: impl Into<String>) -> Result<Self, RecordingIdError> {
        let raw = input.into();
        if raw.is_empty() {
            return Err(RecordingIdError::Empty);
        }
        let len = raw.len();
        if !(RECORDING_ID_MIN_LEN..=RECORDING_ID_MAX_LEN).contains(&len) {
            return Err(RecordingIdError::InvalidLength {
                got: len,
                min: RECORDING_ID_MIN_LEN,
                max: RECORDING_ID_MAX_LEN,
            });
        }
        if !raw.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(RecordingIdError::NonDigit);
        }
        let Ok(seconds) = raw.parse::<i64>() else {
            return Err(RecordingIdError::OutOfRange { got: raw });
        };
        Ok(Self { raw, seconds })
    }

    /// Borrow the underlying string (matches the on-device basename).
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// The id interpreted as Unix epoch seconds. Parsed once at
    /// construction time and stored alongside the raw string.
    #[must_use]
    pub const fn as_unix_seconds(&self) -> i64 {
        self.seconds
    }
}

impl fmt::Display for RecordingId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.raw, f)
    }
}

impl fmt::Debug for RecordingId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("RecordingId").field(&self.raw).finish()
    }
}

impl FromStr for RecordingId {
    type Err = RecordingIdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

/// Whether a recording was captured in NOTE mode (button-triggered) or
/// CALL mode (phone-call capture). Determined by the physical slider
/// position at recording time; the on-device filesystem groups recordings
/// by mode under `/NOTES/` and `/CALLS/` respectively.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum RecordingKind {
    /// Button-triggered recording. Stored under `/NOTES/<YYYYMMDD>/`.
    Note,
    /// Call-mode recording. Stored under `/CALLS/<YYYYMMDD>/`.
    Call,
}

impl RecordingKind {
    /// Human-readable name used by the CLI's textual output.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Call => "call",
        }
    }
}

impl fmt::Display for RecordingKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A single recording as the CLI sees it: stable id, kind, and byte
/// sizes of the paired `.WAV` (stereo PCM) and `.ASR` (mono Opus
/// sidecar) files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recording {
    id: RecordingId,
    kind: RecordingKind,
    wav_size: u64,
    asr_size: u64,
}

impl Recording {
    /// Construct a `Recording` from validated components.
    #[must_use]
    pub const fn new(id: RecordingId, kind: RecordingKind, wav_size: u64, asr_size: u64) -> Self {
        Self {
            id,
            kind,
            wav_size,
            asr_size,
        }
    }

    /// Borrow the recording's stable id.
    #[must_use]
    pub const fn id(&self) -> &RecordingId {
        &self.id
    }

    /// Recording mode (note vs call).
    #[must_use]
    pub const fn kind(&self) -> RecordingKind {
        self.kind
    }

    /// Size of the stereo PCM `.WAV` file in bytes.
    #[must_use]
    pub const fn wav_size(&self) -> u64 {
        self.wav_size
    }

    /// Size of the mono Opus `.ASR` sidecar file in bytes.
    #[must_use]
    pub const fn asr_size(&self) -> u64 {
        self.asr_size
    }

    /// Recording start time as Unix epoch seconds, derived from the id.
    #[must_use]
    pub fn started_at_unix_seconds(&self) -> i64 {
        self.id.as_unix_seconds()
    }
}
