//! `plaude record` — remote recording pipeline control.
//!
//! Maps `start`, `stop`, `pause`, `resume` to the corresponding
//! [`Transport`] methods. All four require an authenticated transport.
//!
//! Journey: specs/plaude-v1/journeys/M11-settings-record-control.md

use std::path::Path;

use clap::Subcommand;
use plaud_auth::DEFAULT_DEVICE_ID;
use plaud_domain::AuthToken;

use crate::{
    DispatchError,
    commands::{auth::build_store, backend::TransportProvider},
};

/// Confirmation message printed after a successful `start`.
const MSG_STARTED: &str = "recording started";
/// Confirmation message printed after a successful `stop`.
const MSG_STOPPED: &str = "recording stopped";
/// Confirmation message printed after a successful `pause`.
const MSG_PAUSED: &str = "recording paused";
/// Confirmation message printed after a successful `resume`.
const MSG_RESUMED: &str = "recording resumed";

/// `plaude record` subcommand tree.
#[derive(Debug, Subcommand)]
pub(crate) enum RecordCommand {
    /// Start a new recording.
    Start,
    /// Stop the current recording.
    Stop,
    /// Pause the current recording.
    Pause,
    /// Resume a paused recording.
    Resume,
}

/// Entry point dispatched from `main::dispatch`.
pub(crate) async fn run(cmd: RecordCommand, provider: &dyn TransportProvider, config_dir: Option<&Path>) -> Result<(), DispatchError> {
    let token = load_token(config_dir).await?;
    let transport = provider
        .connect_authenticated(token)
        .await
        .map_err(|e| DispatchError::from_transport_error(&e))?;

    match cmd {
        RecordCommand::Start => {
            transport
                .start_recording()
                .await
                .map_err(|e| DispatchError::from_transport_error(&e))?;
            println!("{MSG_STARTED}");
        }
        RecordCommand::Stop => {
            transport
                .stop_recording()
                .await
                .map_err(|e| DispatchError::from_transport_error(&e))?;
            println!("{MSG_STOPPED}");
        }
        RecordCommand::Pause => {
            transport
                .pause_recording()
                .await
                .map_err(|e| DispatchError::from_transport_error(&e))?;
            println!("{MSG_PAUSED}");
        }
        RecordCommand::Resume => {
            transport
                .resume_recording()
                .await
                .map_err(|e| DispatchError::from_transport_error(&e))?;
            println!("{MSG_RESUMED}");
        }
    }
    Ok(())
}

async fn load_token(config_dir: Option<&Path>) -> Result<AuthToken, DispatchError> {
    let store = build_store(config_dir)?;
    let stored = store
        .get_token(DEFAULT_DEVICE_ID)
        .await
        .map_err(|e| DispatchError::Runtime(format!("failed to read token store: {e}")))?;
    stored.ok_or(DispatchError::AuthRequired)
}
