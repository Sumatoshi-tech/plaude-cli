//! SHA-256-truncated-to-16-hex-chars fingerprint helper.
//!
//! The fingerprint is used by `plaude auth show` and by every log
//! statement that needs to reference a token without revealing it. It
//! matches the `sha256(token)[:16]` format already documented in the
//! research evidence.

use std::fmt::Write;

use plaud_domain::AuthToken;
use sha2::{Digest, Sha256};

use crate::constants::FINGERPRINT_HEX_LEN;

/// Compute the public-safe fingerprint of an auth token.
///
/// Implementation detail: SHA-256 of the token's UTF-8 bytes, hex
/// encoded, truncated to [`FINGERPRINT_HEX_LEN`] characters.
///
/// This function is deterministic and side-effect-free. It does **not**
/// log or print anything; callers decide where the fingerprint goes.
#[must_use]
pub fn token_fingerprint(token: &AuthToken) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_str().as_bytes());
    let digest = hasher.finalize();
    let mut out = String::with_capacity(FINGERPRINT_HEX_LEN);
    for byte in &digest[..FINGERPRINT_HEX_LEN / 2] {
        // Writing into a `String` via `write!` is infallible — the
        // `fmt::Write` impl for `String` never returns an error — so
        // swallowing the `Result` with `let _` is sound and avoids
        // the banned `.unwrap()` / `.expect()`.
        let _ = write!(out, "{byte:02x}");
    }
    out
}
