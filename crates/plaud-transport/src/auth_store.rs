//! The `AuthStore` trait — pluggable credential storage.
//!
//! Implementations: `plaud-auth::KeyringStore` (OS-native keyring),
//! `plaud-auth::FileStore` (file-backed fallback), and
//! `plaud-auth::ChainStore` (tries keyring first, falls back to file).

use async_trait::async_trait;
use plaud_domain::AuthToken;

use crate::error::Result;

/// Pluggable storage for per-device [`AuthToken`]s.
///
/// A "device id" is an opaque stable handle chosen by the caller —
/// typically the device serial, hashed for indexing. The storage
/// layer does not interpret it, it only uses it as a key.
#[async_trait]
pub trait AuthStore: Send + Sync {
    /// Fetch the token for `device_id` if one is stored.
    async fn get_token(&self, device_id: &str) -> Result<Option<AuthToken>>;

    /// Store `token` under `device_id`, overwriting any existing value.
    async fn put_token(&self, device_id: &str, token: AuthToken) -> Result<()>;

    /// Remove the token for `device_id`. Returns `Ok(())` whether or
    /// not a token was present.
    async fn remove_token(&self, device_id: &str) -> Result<()>;
}
