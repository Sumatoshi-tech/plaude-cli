//! Authentication frame encoder.
//!
//! Produces the V0095-compatible `0x0001 Authenticate` control frame
//! observed in [`specs/re/captures/btsnoop/2026-04-05-plaud-sync-session.md`]
//! and replay-validated in
//! [`specs/re/captures/ble-live-tests/2026-04-05-token-validation.md`].
//!
//! The wire layout is:
//!
//! ```text
//! 01 01 00  02 00  00  <ASCII-hex token bytes>
//! ```
//!
//! The fields are the 3-byte control header, then a 2-byte length
//! constant (`packInt(2)` in the tinnotech SDK source), then a single
//! version byte (observed as zero on V0095), then the token's ASCII
//! bytes (16 or 32 chars, enforced at [`AuthToken`] construction time
//! in `plaud-domain`). The fixed prefix is documented as
//! [`crate::constants::AUTH_PREFIX`].
//!
//! [`AuthToken`]: plaud_domain::AuthToken

use bytes::{BufMut, Bytes, BytesMut};
use plaud_domain::AuthToken;

use crate::constants::AUTH_PREFIX;

/// Build a V0095-compatible auth frame for `token`.
///
/// The returned bytes can be written verbatim to the vendor write
/// characteristic `0x2BB1` to authenticate a BLE session. Zero
/// intermediate allocations beyond the returned `Bytes` buffer.
#[must_use]
pub fn authenticate(token: &AuthToken) -> Bytes {
    let token_bytes = token.as_str().as_bytes();
    let mut buf = BytesMut::with_capacity(AUTH_PREFIX.len() + token_bytes.len());
    buf.put_slice(AUTH_PREFIX);
    buf.put_slice(token_bytes);
    buf.freeze()
}
