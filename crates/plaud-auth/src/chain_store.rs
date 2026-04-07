//! `ChainStore` — composes two `AuthStore`s in a "primary + secondary"
//! fallback arrangement.
//!
//! Semantics, matching the M4 DoD:
//!
//! * `get_token` — tries primary first. If primary returns `Ok(Some)`,
//!   returns it. Else (primary returns `Ok(None)` **or** any error)
//!   falls through to secondary.
//! * `put_token` — writes to primary. If primary errors, retries on
//!   secondary. If both error, returns the primary error.
//! * `remove_token` — attempts removal on both backends. Returns
//!   success if either removal succeeded. Returns the primary error
//!   only if both failed.

use async_trait::async_trait;
use plaud_domain::AuthToken;
use plaud_transport::{AuthStore, Result};

/// Two-layer [`AuthStore`]. See the module docs for fallback
/// semantics.
pub struct ChainStore {
    primary: Box<dyn AuthStore>,
    secondary: Box<dyn AuthStore>,
}

impl std::fmt::Debug for ChainStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChainStore").finish_non_exhaustive()
    }
}

impl ChainStore {
    /// Build a chain from two concrete stores.
    #[must_use]
    pub fn new(primary: Box<dyn AuthStore>, secondary: Box<dyn AuthStore>) -> Self {
        Self { primary, secondary }
    }
}

#[async_trait]
impl AuthStore for ChainStore {
    async fn get_token(&self, device_id: &str) -> Result<Option<AuthToken>> {
        match self.primary.get_token(device_id).await {
            Ok(Some(t)) => Ok(Some(t)),
            Ok(None) | Err(_) => self.secondary.get_token(device_id).await,
        }
    }

    async fn put_token(&self, device_id: &str, token: AuthToken) -> Result<()> {
        match self.primary.put_token(device_id, token.clone()).await {
            Ok(()) => Ok(()),
            Err(primary_err) => match self.secondary.put_token(device_id, token).await {
                Ok(()) => Ok(()),
                Err(_) => Err(primary_err),
            },
        }
    }

    async fn remove_token(&self, device_id: &str) -> Result<()> {
        let primary_result = self.primary.remove_token(device_id).await;
        let secondary_result = self.secondary.remove_token(device_id).await;
        match (primary_result, secondary_result) {
            (Ok(()), _) | (_, Ok(())) => Ok(()),
            (Err(primary_err), Err(_)) => Err(primary_err),
        }
    }
}
