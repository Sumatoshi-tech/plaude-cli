//! Decode errors for the plaud-proto codec.

use thiserror::Error;

/// Every distinct failure mode `decode::parse_notification` and its
/// callers can produce.
#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum DecodeError {
    /// Input byte slice was empty.
    #[error("notification is empty")]
    Empty,

    /// First byte did not match any known frame-type value.
    #[error("unknown frame type byte: 0x{byte:02x}")]
    UnknownFrameType {
        /// The offending first byte.
        byte: u8,
    },

    /// Input was shorter than the minimum header of the frame family
    /// its first byte identified.
    #[error("frame too short: expected at least {expected} bytes, got {got}")]
    TooShort {
        /// Required minimum byte count for this frame type.
        expected: usize,
        /// Actual byte count received.
        got: usize,
    },

    /// [`decode::auth_response`] was called on a `Frame` that is not
    /// a control frame with opcode `0x0001`.
    #[error("frame is not an auth response")]
    NotAuthResponse,

    /// [`decode::auth_response`] observed a status byte outside the
    /// documented `{0x00, 0x01}` value set.
    #[error("unknown auth status byte: 0x{byte:02x}")]
    UnknownAuthStatus {
        /// The offending status byte.
        byte: u8,
    },

    /// [`decode::parse_auth_write`] received bytes whose leading
    /// header did not match [`crate::constants::AUTH_PREFIX`].
    #[error("auth write prefix mismatch")]
    InvalidAuthPrefix,

    /// [`decode::parse_auth_write`] parsed the prefix successfully
    /// but the trailing token bytes failed [`plaud_domain::AuthToken`]
    /// validation (wrong length, non-ASCII-hex, or non-UTF-8).
    #[error("auth token validation failed: {reason}")]
    InvalidAuthToken {
        /// Human-readable reason from the token validator.
        reason: String,
    },
}
