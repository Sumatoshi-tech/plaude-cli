//! File-backed `AuthStore`.
//!
//! Stores a single token in a plain-text file at a caller-chosen path
//! (typically `~/.config/plaude/token`). On Unix the token file gets
//! `0600` permissions and its parent directory gets `0700`. On Windows
//! the file inherits the default ACLs — a v1 compromise that we
//! document in `docs/usage/auth.md`.

use std::{
    io,
    path::{Path, PathBuf},
};

use async_trait::async_trait;
use plaud_domain::AuthToken;
use plaud_transport::{AuthStore, Error, Result};

use crate::constants::{CONFIG_SUBDIR, TOKEN_FILE_NAME};

/// File-backed implementation of [`AuthStore`].
#[derive(Debug, Clone)]
pub struct FileStore {
    path: PathBuf,
}

impl FileStore {
    /// Build a file store that writes to `path`. The caller is
    /// responsible for ensuring the path is somewhere writable.
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Resolve the default token path under the user's config dir:
    /// `$XDG_CONFIG_HOME/plaude/token` on Linux (with
    /// `~/.config/plaude/token` as the XDG fallback), the analogous
    /// location on macOS and Windows.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Transport`] if the platform does not expose
    /// a config dir (vanishingly rare — only on exotic embedded
    /// targets that `dirs` does not support).
    pub fn default_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().ok_or_else(|| Error::Transport("the current platform has no config directory".to_owned()))?;
        Ok(config_dir.join(CONFIG_SUBDIR).join(TOKEN_FILE_NAME))
    }

    /// Borrow the path this store writes to. Exposed so the CLI can
    /// tell the user where the token lives in `auth show` output.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[async_trait]
impl AuthStore for FileStore {
    async fn get_token(&self, _device_id: &str) -> Result<Option<AuthToken>> {
        match tokio::fs::read_to_string(&self.path).await {
            Ok(content) => {
                let trimmed = content.trim();
                if trimmed.is_empty() {
                    return Ok(None);
                }
                let token = AuthToken::new(trimmed).map_err(|e| Error::Protocol(format!("stored token is invalid: {e}")))?;
                Ok(Some(token))
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(Error::Io(e)),
        }
    }

    async fn put_token(&self, _device_id: &str, token: AuthToken) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(Error::Io)?;
            #[cfg(unix)]
            set_parent_mode(parent).await?;
        }
        let contents = format!("{}\n", token.as_str());
        tokio::fs::write(&self.path, contents.as_bytes()).await.map_err(Error::Io)?;
        #[cfg(unix)]
        set_token_file_mode(&self.path).await?;
        Ok(())
    }

    async fn remove_token(&self, _device_id: &str) -> Result<()> {
        match tokio::fs::remove_file(&self.path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(Error::Io(e)),
        }
    }
}

#[cfg(unix)]
async fn set_token_file_mode(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let owned = path.to_owned();
    tokio::task::spawn_blocking(move || -> std::io::Result<()> {
        let perms = std::fs::Permissions::from_mode(crate::constants::TOKEN_FILE_MODE);
        std::fs::set_permissions(&owned, perms)
    })
    .await
    .map_err(|e| Error::Transport(format!("blocking task failed: {e}")))?
    .map_err(Error::Io)
}

#[cfg(unix)]
async fn set_parent_mode(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let owned = path.to_owned();
    tokio::task::spawn_blocking(move || -> std::io::Result<()> {
        let perms = std::fs::Permissions::from_mode(crate::constants::TOKEN_PARENT_DIR_MODE);
        std::fs::set_permissions(&owned, perms)
    })
    .await
    .map_err(|e| Error::Transport(format!("blocking task failed: {e}")))?
    .map_err(Error::Io)
}
