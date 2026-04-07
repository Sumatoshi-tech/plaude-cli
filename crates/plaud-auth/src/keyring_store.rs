//! OS-keyring-backed `AuthStore`.
//!
//! Wraps the [`keyring`] crate, which is synchronous. Every trait
//! method moves the work onto a blocking pool via
//! [`tokio::task::spawn_blocking`] so async callers don't pay a
//! thread-park cost on the main runtime.
//!
//! [`keyring`]: https://docs.rs/keyring

use async_trait::async_trait;
use plaud_domain::AuthToken;
use plaud_transport::{AuthStore, Error, Result};

use crate::constants::KEYRING_SERVICE;

/// OS-keyring-backed implementation of [`AuthStore`].
#[derive(Debug, Clone)]
pub struct KeyringStore {
    service: String,
}

impl KeyringStore {
    /// Build a keyring store under the given service name. Callers
    /// that just want the default service name should use
    /// [`Self::default`].
    #[must_use]
    pub fn new(service: impl Into<String>) -> Self {
        Self { service: service.into() }
    }
}

impl Default for KeyringStore {
    fn default() -> Self {
        Self::new(KEYRING_SERVICE)
    }
}

#[async_trait]
impl AuthStore for KeyringStore {
    async fn get_token(&self, device_id: &str) -> Result<Option<AuthToken>> {
        let service = self.service.clone();
        let device = device_id.to_owned();
        let join = tokio::task::spawn_blocking(move || -> Result<Option<AuthToken>> {
            let entry = keyring::Entry::new(&service, &device).map_err(map_keyring_err)?;
            match entry.get_password() {
                Ok(raw) => {
                    let token = AuthToken::new(raw).map_err(|e| Error::Protocol(format!("stored token is invalid: {e}")))?;
                    Ok(Some(token))
                }
                Err(keyring::Error::NoEntry) => Ok(None),
                Err(other) => Err(map_keyring_err(other)),
            }
        })
        .await;
        flatten(join)
    }

    async fn put_token(&self, device_id: &str, token: AuthToken) -> Result<()> {
        let service = self.service.clone();
        let device = device_id.to_owned();
        let raw = token.as_str().to_owned();
        let join = tokio::task::spawn_blocking(move || -> Result<()> {
            let entry = keyring::Entry::new(&service, &device).map_err(map_keyring_err)?;
            entry.set_password(&raw).map_err(map_keyring_err)
        })
        .await;
        flatten(join)
    }

    async fn remove_token(&self, device_id: &str) -> Result<()> {
        let service = self.service.clone();
        let device = device_id.to_owned();
        let join = tokio::task::spawn_blocking(move || -> Result<()> {
            let entry = keyring::Entry::new(&service, &device).map_err(map_keyring_err)?;
            match entry.delete_credential() {
                Ok(()) => Ok(()),
                Err(keyring::Error::NoEntry) => Ok(()),
                Err(other) => Err(map_keyring_err(other)),
            }
        })
        .await;
        flatten(join)
    }
}

fn map_keyring_err(e: keyring::Error) -> Error {
    Error::Transport(format!("keyring error: {e}"))
}

fn flatten<T>(join: std::result::Result<Result<T>, tokio::task::JoinError>) -> Result<T> {
    match join {
        Ok(inner) => inner,
        Err(e) => Err(Error::Transport(format!("keyring blocking task failed: {e}"))),
    }
}
