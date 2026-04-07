//! Auth token storage and bootstrap flows for plaude-cli.
//!
//! The Plaud Note's BLE vendor protocol authenticates a session with
//! a single pre-shared ASCII-hex token (see
//! [`specs/re/captures/ble-live-tests/2026-04-05-token-validation.md`](../../../specs/re/captures/ble-live-tests/2026-04-05-token-validation.md)).
//! This crate provides the storage and import layer that keeps the
//! token available across CLI invocations without ever printing it.
//!
//! # Backends
//!
//! * [`KeyringStore`] — OS-keyring backed (Linux Secret Service,
//!   macOS Keychain, Windows Credential Manager). Async-safe via
//!   `tokio::task::spawn_blocking`.
//! * [`FileStore`] — fallback file backend at `~/.config/plaude/token`
//!   (Unix permissions `0600`, parent directory `0700`).
//! * [`ChainStore`] — composes the two with primary-then-secondary
//!   fallback. See its module docs for the exact semantics.
//!
//! # Helpers
//!
//! * [`btsnoop::extract_auth_token`] — pure-Rust parser for Android
//!   HCI snoop logs. Walks records looking for the first ATT Write
//!   Command to handle `0x000D` whose value carries the V0095
//!   `AUTH_FRAME_PREFIX`; returns the embedded token as an
//!   [`plaud_domain::AuthToken`].
//! * [`fingerprint::token_fingerprint`] — SHA-256 of the token,
//!   truncated to 16 hex characters. The only form a user ever sees.
//!
//! # Convenience
//!
//! * [`default_store`] — builds the canonical `ChainStore` with
//!   `KeyringStore::default` primary and `FileStore::new(default_path()?)`
//!   secondary.

pub mod btsnoop;
pub mod chain_store;
pub mod constants;
pub mod file_store;
pub mod fingerprint;
pub mod keyring_store;

pub use chain_store::ChainStore;
pub use constants::DEFAULT_DEVICE_ID;
pub use file_store::FileStore;
pub use fingerprint::token_fingerprint;
pub use keyring_store::KeyringStore;
use plaud_transport::Result;

/// Build the canonical production auth store: OS keyring primary,
/// file fallback at `~/.config/plaude/token`.
///
/// # Errors
///
/// Returns [`plaud_transport::Error::Transport`] if the platform does
/// not expose a config directory. Every other setup concern
/// (keyring daemon reachable, file permissions writable) is deferred
/// to first use of the returned store.
pub fn default_store() -> Result<ChainStore> {
    let keyring = KeyringStore::default();
    let file = FileStore::new(FileStore::default_path()?);
    Ok(ChainStore::new(Box::new(keyring), Box::new(file)))
}
