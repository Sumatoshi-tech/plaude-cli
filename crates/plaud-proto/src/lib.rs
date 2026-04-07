//! Pure codec for the tinnotech pen-BLE wire protocol used by Plaud devices.
//!
//! # Scope
//!
//! This crate is the single source of truth for the two frame families
//! the Plaud Note speaks over its BLE vendor characteristics:
//!
//! * **Control frames** (magic byte `0x01`) — every opcode request
//!   and response, including the `0x0001 Authenticate` handshake.
//! * **Bulk frames** (magic byte `0x02`) — one chunk of an ongoing
//!   file transfer, including the `0xFFFFFFFF`-offset end-of-stream
//!   sentinel.
//!
//! Pre-auth handshake preambles for the newer RSA + ChaCha20-Poly1305
//! mode (`0xFE11` / `0xFE12`) are **detected** so a downstream
//! transport can defer to the mode-B path, but are not yet
//! **parsed** at the crate level.
//!
//! See [`docs/protocol/ble-commands.md`](../../../docs/protocol/ble-commands.md)
//! for the canonical wire specification and
//! [`specs/re/apk-notes/3.14.0-620/ble-protocol.md`](../../../specs/re/apk-notes/3.14.0-620/ble-protocol.md)
//! for the APK-derived 45-opcode dictionary.
//!
//! # Performance
//!
//! The decoder operates on [`bytes::Bytes`] and uses the zero-copy
//! [`bytes::Bytes::slice`] to return payload slices without copying.
//! Encoders allocate exactly one [`bytes::BytesMut`] sized to the
//! final frame length.
//!
//! # Stability
//!
//! Every constant in [`constants`] is documented with evidence
//! citations; every opcode in [`opcode`] was observed on the wire in
//! at least one committed btsnoop walkthrough. Adding a new opcode
//! requires (a) an evidence file in `specs/re/captures/` and (b) a
//! round-trip test in `tests/`.

pub mod constants;
pub mod decode;
pub mod encode;
pub mod error;
pub mod frame;
pub mod opcode;

pub use decode::{auth_response, parse_auth_write, parse_notification};
pub use error::DecodeError;
pub use frame::{AuthStatus, Frame};
