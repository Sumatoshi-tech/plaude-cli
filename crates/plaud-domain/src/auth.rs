//! Auth-token domain type.
//!
//! The Plaud Note's BLE vendor protocol authenticates a session with
//! a single pre-shared ASCII-hex token written to characteristic
//! `0x2BB1` at the start of every connection. Two token lengths are
//! observed in the wild:
//!
//! * **32 hex chars** — firmware `V0095` (live-tested).
//! * **16 hex chars** — older protocol versions per the tinnotech SDK
//!   source (`p257nh/C9555a0.java`).
//!
//! The domain type accepts either length and stores the value inside
//! a [`zeroize::Zeroizing`] buffer so the bytes are scrubbed from
//! memory when the token is dropped.
//!
//! Evidence: `specs/re/captures/ble-live-tests/2026-04-05-token-validation.md`
//! and `specs/re/apk-notes/3.14.0-620/auth-token.md`.

use std::fmt;

use thiserror::Error;
use zeroize::Zeroizing;

/// Short-form token length (16 ASCII hex chars) used by older firmware.
const AUTH_TOKEN_LEN_SHORT: usize = 16;

/// Long-form token length (32 ASCII hex chars) used by V0095 and newer
/// plaintext-auth firmware.
const AUTH_TOKEN_LEN_LONG: usize = 32;

/// Validation errors produced by [`AuthToken::new`].
#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum AuthTokenError {
    /// The input length was neither 16 nor 32 ASCII characters.
    #[error("auth token length {got} is not one of the accepted values ({short}, {long})")]
    InvalidLength {
        /// Observed length.
        got: usize,
        /// Short-form length.
        short: usize,
        /// Long-form length.
        long: usize,
    },
    /// The input contained a byte that is not an ASCII hex digit.
    #[error("auth token must be ASCII hex digits only")]
    NonHex,
}

/// A Plaud BLE vendor authentication token.
///
/// The wrapped [`Zeroizing<String>`] clears the string's heap buffer
/// when the token is dropped. The [`fmt::Debug`] impl never prints the
/// raw value; code that must send the token over the wire uses
/// [`Self::as_str`] explicitly.
#[derive(Clone, PartialEq, Eq)]
pub struct AuthToken(Zeroizing<String>);

impl AuthToken {
    /// Construct an `AuthToken` after validating length and hex-digit
    /// character class.
    ///
    /// # Errors
    ///
    /// Returns [`AuthTokenError`] on length mismatch or non-hex input.
    pub fn new(input: impl Into<String>) -> Result<Self, AuthTokenError> {
        let raw = input.into();
        let len = raw.len();
        if len != AUTH_TOKEN_LEN_SHORT && len != AUTH_TOKEN_LEN_LONG {
            return Err(AuthTokenError::InvalidLength {
                got: len,
                short: AUTH_TOKEN_LEN_SHORT,
                long: AUTH_TOKEN_LEN_LONG,
            });
        }
        if !raw.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(AuthTokenError::NonHex);
        }
        Ok(Self(Zeroizing::new(raw)))
    }

    /// Borrow the raw token. This is the only accessor that returns
    /// the unredacted string — it is named explicitly so logging or
    /// serialising the token requires a grep-visible call site.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Length of the token in bytes. Always equal to either
    /// [`AUTH_TOKEN_LEN_SHORT`] or [`AUTH_TOKEN_LEN_LONG`] by
    /// construction.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the underlying string is empty. Always `false` for a
    /// constructed `AuthToken`; provided so clippy's
    /// `len_without_is_empty` does not fire.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for AuthToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AuthToken")
            .field("len", &self.0.len())
            .field("value", &"<redacted>")
            .finish()
    }
}
