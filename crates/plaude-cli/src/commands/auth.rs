//! `plaude auth` subcommand tree.
//!
//! Four subcommands drive the `plaud-auth` storage layer:
//!
//! * `set-token <hex>` — store a manually-entered token.
//! * `import <path>` — extract a token from a btsnoop log.
//! * `show` — print the fingerprint of the stored token.
//! * `clear` — remove the stored token.

use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use plaud_auth::{ChainStore, DEFAULT_DEVICE_ID, FileStore, KeyringStore, btsnoop, token_fingerprint};
use plaud_domain::AuthToken;
use plaud_transport::AuthStore;
use plaud_transport_ble::{BootstrapError, LoopbackBootstrap};

use crate::{DispatchError, commands::backend::Backend};

/// File name used inside a caller-supplied `--config-dir` for the
/// sandboxed token file. Production uses the default path from
/// [`FileStore::default_path`].
const SANDBOX_TOKEN_FILE_NAME: &str = "token";

/// Default timeout used by `plaude auth bootstrap` when the user
/// does not pass `--timeout`. Matches the M8 DoD "default 120 s".
const DEFAULT_BOOTSTRAP_TIMEOUT_SECS: u64 = 120;

/// Deterministic auth token the sim bootstrap backend writes from the
/// fake phone. 32 ASCII hex chars so `AuthToken::new` accepts it; the
/// M8 e2e test asserts the CLI captures and stores this exact value.
const SIM_BOOTSTRAP_TOKEN: &str = "abcdef0123456789abcdef0123456789";

/// `plaude auth` subcommand tree.
#[derive(Debug, Subcommand)]
pub(crate) enum AuthCommand {
    /// Store a 16- or 32-character ASCII-hex token.
    SetToken {
        /// The token string. Must satisfy `AuthToken::new` validation.
        token: String,
    },
    /// Import a token from an Android HCI snoop log (`btsnoop_hci.log`).
    Import {
        /// Path to the btsnoop log file.
        path: PathBuf,
    },
    /// Show the fingerprint of the stored token without revealing
    /// its value.
    Show,
    /// Remove the stored token from all backends.
    Clear,
    /// Capture a token by impersonating a Plaud Note locally. The
    /// user opens their Plaud phone app, it connects to the laptop
    /// instead of the real pen, and the first auth write is
    /// captured and stored.
    Bootstrap(BootstrapArgs),
}

/// Arguments for `plaude auth bootstrap`.
#[derive(Debug, Args)]
pub(crate) struct BootstrapArgs {
    /// Budget in seconds to wait for the phone's auth write.
    #[arg(long, default_value_t = DEFAULT_BOOTSTRAP_TIMEOUT_SECS)]
    timeout: u64,
}

/// Entry point invoked by `main::dispatch` after the tokio runtime
/// is built.
pub(crate) async fn run(cmd: AuthCommand, config_dir: Option<&Path>, backend: Backend) -> Result<(), DispatchError> {
    let store = build_store(config_dir)?;
    match cmd {
        AuthCommand::SetToken { token } => set_token(store.as_ref(), &token).await,
        AuthCommand::Import { path } => import_from_log(store.as_ref(), &path).await,
        AuthCommand::Show => show(store.as_ref()).await,
        AuthCommand::Clear => clear(store.as_ref()).await,
        AuthCommand::Bootstrap(args) => bootstrap(store.as_ref(), args, backend).await,
    }
}

pub(crate) fn build_store(config_dir: Option<&Path>) -> Result<Box<dyn AuthStore>, DispatchError> {
    match config_dir {
        Some(dir) => {
            // Sandbox mode: file backend only, rooted in the supplied
            // directory. The keyring backend is intentionally skipped
            // so tests never touch the user's real keyring.
            let path = dir.join(SANDBOX_TOKEN_FILE_NAME);
            Ok(Box::new(FileStore::new(path)))
        }
        None => {
            // Production: keyring primary, file fallback.
            let keyring = Box::new(KeyringStore::default());
            let file_path = FileStore::default_path().map_err(|e| DispatchError::Runtime(format!("cannot resolve config dir: {e}")))?;
            let file = Box::new(FileStore::new(file_path));
            Ok(Box::new(ChainStore::new(keyring, file)))
        }
    }
}

async fn set_token(store: &dyn AuthStore, raw: &str) -> Result<(), DispatchError> {
    let token = AuthToken::new(raw).map_err(|e| DispatchError::Usage(format!("invalid token: {e}")))?;
    let fingerprint = token_fingerprint(&token);
    store
        .put_token(DEFAULT_DEVICE_ID, token)
        .await
        .map_err(|e| DispatchError::Runtime(format!("failed to store token: {e}")))?;
    println!("Token stored. Fingerprint: {fingerprint}");
    Ok(())
}

async fn import_from_log(store: &dyn AuthStore, path: &Path) -> Result<(), DispatchError> {
    let bytes = tokio::fs::read(path)
        .await
        .map_err(|e| DispatchError::Runtime(format!("failed to read {}: {e}", path.display())))?;
    let token = btsnoop::extract_auth_token(&bytes).map_err(|e| DispatchError::Runtime(format!("btsnoop parse: {e}")))?;
    let fingerprint = token_fingerprint(&token);
    store
        .put_token(DEFAULT_DEVICE_ID, token)
        .await
        .map_err(|e| DispatchError::Runtime(format!("failed to store token: {e}")))?;
    println!("Token imported. Fingerprint: {fingerprint}");
    Ok(())
}

async fn show(store: &dyn AuthStore) -> Result<(), DispatchError> {
    match store
        .get_token(DEFAULT_DEVICE_ID)
        .await
        .map_err(|e| DispatchError::Runtime(format!("failed to read store: {e}")))?
    {
        Some(token) => {
            println!("Token stored.");
            println!("Fingerprint: {}", token_fingerprint(&token));
            Ok(())
        }
        None => Err(DispatchError::Usage(
            "no token stored; run `plaude auth set-token <hex>` or `plaude auth import <log>`".to_owned(),
        )),
    }
}

async fn clear(store: &dyn AuthStore) -> Result<(), DispatchError> {
    store
        .remove_token(DEFAULT_DEVICE_ID)
        .await
        .map_err(|e| DispatchError::Runtime(format!("failed to clear token: {e}")))?;
    println!("Token removed.");
    Ok(())
}

async fn bootstrap(store: &dyn AuthStore, args: BootstrapArgs, backend: Backend) -> Result<(), DispatchError> {
    match backend {
        Backend::Sim => bootstrap_sim(store, args).await,
        Backend::Ble => bootstrap_ble(store, args).await,
        Backend::Usb => Err(DispatchError::Usage(
            "`auth bootstrap` is a BLE-peripheral flow; use `--backend sim` or wait for the btleplug backend".to_owned(),
        )),
    }
}

async fn bootstrap_sim(store: &dyn AuthStore, args: BootstrapArgs) -> Result<(), DispatchError> {
    let (session, phone) = LoopbackBootstrap::new().split();
    let sim_token = AuthToken::new(SIM_BOOTSTRAP_TOKEN).map_err(|e| DispatchError::Runtime(format!("sim token invalid: {e}")))?;
    let wire = plaud_proto::encode::auth::authenticate(&sim_token);
    let session_task = tokio::spawn(async move { session.run(std::time::Duration::from_secs(args.timeout)).await });
    phone
        .write(wire)
        .await
        .map_err(|e| DispatchError::Runtime(format!("sim phone write failed: {e}")))?;
    // Keep `phone` alive until the peripheral finishes so the
    // mock auth-accepted notification it sends back does not hit a
    // closed receiver.
    let outcome = session_task
        .await
        .map_err(|e| DispatchError::Runtime(format!("session task join failed: {e}")))?
        .map_err(map_bootstrap_error)?;
    drop(phone);
    let fingerprint = token_fingerprint(&outcome.token);
    store
        .put_token(DEFAULT_DEVICE_ID, outcome.token)
        .await
        .map_err(|e| DispatchError::Runtime(format!("failed to store captured token: {e}")))?;
    println!("Token captured. Fingerprint: {fingerprint}");
    Ok(())
}

async fn bootstrap_ble(store: &dyn AuthStore, args: BootstrapArgs) -> Result<(), DispatchError> {
    use plaud_transport_ble::bootstrap::bluer_peripheral::run_bluer_bootstrap;

    let outcome = run_bluer_bootstrap(std::time::Duration::from_secs(args.timeout))
        .await
        .map_err(map_bootstrap_error)?;
    let fingerprint = token_fingerprint(&outcome.token);
    store
        .put_token(DEFAULT_DEVICE_ID, outcome.token)
        .await
        .map_err(|e| DispatchError::Runtime(format!("failed to store captured token: {e}")))?;
    println!("Token captured. Fingerprint: {fingerprint}");
    Ok(())
}

fn map_bootstrap_error(err: BootstrapError) -> DispatchError {
    match err {
        BootstrapError::Timeout { seconds } => DispatchError::Runtime(format!("no phone connected within {seconds}s")),
        BootstrapError::PhoneDisconnected => DispatchError::Runtime("phone disconnected before completing handshake".to_owned()),
        BootstrapError::DecodeFailed { reason } => DispatchError::Runtime(format!("captured write was not a valid auth frame: {reason}")),
        other => DispatchError::Runtime(format!("bootstrap error: {other}")),
    }
}
