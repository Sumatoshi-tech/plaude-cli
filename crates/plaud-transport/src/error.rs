//! The unified error taxonomy every `plaude` transport maps into.
//!
//! Variants are deliberately specific so the CLI can produce
//! distinct user-facing messages (and exit codes) for different
//! failure modes without the CLI layer having to pattern-match on
//! stringly-typed errors.

use thiserror::Error;

/// Result alias used throughout `plaud-transport` and its
/// implementations.
pub type Result<T> = std::result::Result<T, Error>;

/// Unified error type for every transport operation.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// A resource (recording, setting, device) the caller asked for
    /// does not exist.
    #[error("not found: {0}")]
    NotFound(String),

    /// The operation requires a stored auth token but none is present.
    /// The CLI converts this into a hint to run `plaude auth bootstrap`
    /// or `plaude auth import`.
    #[error("authentication required: run `plaude auth bootstrap` or `plaude auth import` first")]
    AuthRequired,

    /// The device rejected the stored auth token. `status` is byte 3
    /// of the `0x0001` response notification; a value of `0x01` is
    /// the documented "soft reject" (see
    /// `specs/re/captures/ble-live-tests/2026-04-05-token-validation.md`).
    #[error("device rejected the stored auth token (status byte: 0x{status:02x})")]
    AuthRejected {
        /// Raw status byte reported by the device.
        status: u8,
    },

    /// The operation timed out waiting for a transport-level response.
    #[error("operation timed out after {seconds} seconds")]
    Timeout {
        /// Timeout duration in whole seconds.
        seconds: u64,
    },

    /// An underlying I/O error (file, socket, serial). Carries the
    /// original `std::io::Error` so the CLI can surface a precise
    /// OS message.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// The wire bytes did not match what we expected from the
    /// protocol spec (missing magic byte, bad CRC, malformed frame).
    #[error("protocol error: {0}")]
    Protocol(String),

    /// The transport implementation itself reported a failure
    /// (BLE stack, USB stack, Wi-Fi stack). Carries a human-readable
    /// description; the CLI logs it and exits with a distinct code.
    #[error("transport error: {0}")]
    Transport(String),

    /// The requested capability is not supported by this transport
    /// (e.g. `read_recording` on a read-only USB fallback that can
    /// do it, but `start_recording` cannot).
    #[error("capability not supported on this transport: {capability}")]
    Unsupported {
        /// Stable capability name; see `Transport` trait docs.
        capability: &'static str,
    },
}
