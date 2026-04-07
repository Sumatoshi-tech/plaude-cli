//! `plaude device` — device information, privacy toggle, and name
//! query.
//!
//! Requires an authenticated transport. Loads the stored token via
//! the same `AuthStore` chain used by the `auth` subcommand tree,
//! authenticates through the active [`TransportProvider`], then
//! queries `device_info` + `storage`.
//!
//! Journey: specs/plaude-v1/journeys/M11-settings-record-control.md

use std::path::Path;

use clap::{Args, Subcommand};
use plaud_auth::DEFAULT_DEVICE_ID;
use plaud_domain::{AuthToken, DeviceInfo, StorageStats};
use plaud_transport::Transport;
use serde::Serialize;

use crate::{
    DispatchError,
    commands::{auth::build_store, backend::TransportProvider, output::OutputFormat},
};

/// Prefix for the local-name line in text mode.
const LOCAL_NAME_HEADER: &str = "Device:     ";
/// Prefix for the model line in text mode.
const MODEL_HEADER: &str = "Model:      ";
/// Prefix for the firmware line in text mode.
const FIRMWARE_HEADER: &str = "Firmware:   ";
/// Prefix for the serial line in text mode.
const SERIAL_HEADER: &str = "Serial:     ";
/// Prefix for the storage line in text mode.
const STORAGE_HEADER: &str = "Storage:    ";

/// `plaude device` subcommand tree.
#[derive(Debug, Subcommand)]
pub(crate) enum DeviceCommand {
    /// Print a formatted summary of the connected device.
    Info(DeviceInfoArgs),
    /// Toggle the device privacy flag.
    Privacy(DevicePrivacyArgs),
    /// Print the device name.
    Name,
}

/// Arguments for `plaude device privacy`.
#[derive(Debug, Args)]
pub(crate) struct DevicePrivacyArgs {
    /// `on` enables privacy mode, `off` disables it.
    #[arg(value_parser = parse_on_off)]
    state: OnOff,
}

/// Boolean wrapper that clap can treat as a positional value.
#[derive(Debug, Clone, Copy)]
pub(crate) struct OnOff(bool);

/// Parse `on`/`off` strings into [`OnOff`].
fn parse_on_off(s: &str) -> Result<OnOff, String> {
    match s {
        "on" => Ok(OnOff(true)),
        "off" => Ok(OnOff(false)),
        _ => Err(format!("expected `on` or `off`, got `{s}`")),
    }
}

/// Confirmation printed after `device privacy on`.
const PRIVACY_ON_MSG: &str = "privacy on";
/// Confirmation printed after `device privacy off`.
const PRIVACY_OFF_MSG: &str = "privacy off";

/// Arguments for `plaude device info`.
#[derive(Debug, Args)]
pub(crate) struct DeviceInfoArgs {
    /// Output format. `text` is the default human-readable form;
    /// `json` emits a single-line object suitable for piping.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    output: OutputFormat,
}

/// Flat JSON projection of a [`DeviceInfo`] + [`StorageStats`] pair.
#[derive(Debug, Serialize)]
struct DeviceInfoJson {
    local_name: String,
    model: String,
    firmware: String,
    serial: String,
    storage: StorageJson,
}

/// Flat JSON projection of a [`StorageStats`].
#[derive(Debug, Serialize)]
struct StorageJson {
    total_bytes: u64,
    used_bytes: u64,
    free_bytes: u64,
    recording_count: u32,
}

/// Entry point dispatched from `main::dispatch`.
pub(crate) async fn run(cmd: DeviceCommand, provider: &dyn TransportProvider, config_dir: Option<&Path>) -> Result<(), DispatchError> {
    match cmd {
        DeviceCommand::Info(args) => info(args, provider, config_dir).await,
        DeviceCommand::Privacy(args) => privacy(args, provider, config_dir).await,
        DeviceCommand::Name => name(provider, config_dir).await,
    }
}

async fn info(args: DeviceInfoArgs, provider: &dyn TransportProvider, config_dir: Option<&Path>) -> Result<(), DispatchError> {
    let token = load_token(config_dir).await?;
    let transport = provider
        .connect_authenticated(token)
        .await
        .map_err(|e| DispatchError::from_transport_error(&e))?;
    let (info, storage) = fetch(transport.as_ref()).await?;
    print_summary(&info, storage, args.output)
}

async fn load_token(config_dir: Option<&Path>) -> Result<AuthToken, DispatchError> {
    let store = build_store(config_dir)?;
    let stored = store
        .get_token(DEFAULT_DEVICE_ID)
        .await
        .map_err(|e| DispatchError::Runtime(format!("failed to read token store: {e}")))?;
    stored.ok_or(DispatchError::AuthRequired)
}

async fn fetch(transport: &dyn Transport) -> Result<(DeviceInfo, StorageStats), DispatchError> {
    let info = transport.device_info().await.map_err(|e| DispatchError::from_transport_error(&e))?;
    let storage = transport.storage().await.map_err(|e| DispatchError::from_transport_error(&e))?;
    Ok((info, storage))
}

async fn privacy(args: DevicePrivacyArgs, provider: &dyn TransportProvider, config_dir: Option<&Path>) -> Result<(), DispatchError> {
    let token = load_token(config_dir).await?;
    let transport = provider
        .connect_authenticated(token)
        .await
        .map_err(|e| DispatchError::from_transport_error(&e))?;
    transport
        .set_privacy(args.state.0)
        .await
        .map_err(|e| DispatchError::from_transport_error(&e))?;
    if args.state.0 {
        println!("{PRIVACY_ON_MSG}");
    } else {
        println!("{PRIVACY_OFF_MSG}");
    }
    Ok(())
}

async fn name(provider: &dyn TransportProvider, config_dir: Option<&Path>) -> Result<(), DispatchError> {
    let token = load_token(config_dir).await?;
    let transport = provider
        .connect_authenticated(token)
        .await
        .map_err(|e| DispatchError::from_transport_error(&e))?;
    let info = transport.device_info().await.map_err(|e| DispatchError::from_transport_error(&e))?;
    println!("{}", info.local_name);
    Ok(())
}

fn print_summary(info: &DeviceInfo, storage: StorageStats, output: OutputFormat) -> Result<(), DispatchError> {
    match output {
        OutputFormat::Text => {
            println!("{LOCAL_NAME_HEADER}{}", info.local_name);
            println!("{MODEL_HEADER}{}", info.model.name());
            println!("{FIRMWARE_HEADER}{}", info.firmware);
            println!("{SERIAL_HEADER}{}", info.serial.reveal());
            println!("{STORAGE_HEADER}{storage}");
            Ok(())
        }
        OutputFormat::Json => {
            let payload = DeviceInfoJson {
                local_name: info.local_name.clone(),
                model: info.model.name().to_owned(),
                firmware: format!("{}", info.firmware),
                serial: info.serial.reveal().to_owned(),
                storage: StorageJson {
                    total_bytes: storage.total_bytes(),
                    used_bytes: storage.used_bytes(),
                    free_bytes: storage.free_bytes(),
                    recording_count: storage.recording_count(),
                },
            };
            let rendered = serde_json::to_string(&payload).map_err(|e| DispatchError::Runtime(format!("json encode: {e}")))?;
            println!("{rendered}");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use plaud_domain::{DeviceInfo, StorageStats};
    use serde_json::Value;

    use super::{DeviceInfoJson, StorageJson};

    #[test]
    fn device_info_json_carries_the_documented_keys() {
        let info = DeviceInfo::placeholder();
        let storage = StorageStats::ZERO;
        let payload = DeviceInfoJson {
            local_name: info.local_name.clone(),
            model: info.model.name().to_owned(),
            firmware: format!("{}", info.firmware),
            serial: info.serial.reveal().to_owned(),
            storage: StorageJson {
                total_bytes: storage.total_bytes(),
                used_bytes: storage.used_bytes(),
                free_bytes: storage.free_bytes(),
                recording_count: storage.recording_count(),
            },
        };
        let rendered = serde_json::to_string(&payload).expect("encode");
        let parsed: Value = serde_json::from_str(&rendered).expect("parse");
        assert!(parsed.get("local_name").is_some());
        assert!(parsed.get("model").is_some());
        assert!(parsed.get("firmware").is_some());
        assert!(parsed.get("serial").is_some());
        assert!(parsed.get("storage").is_some());
        assert!(parsed["storage"].get("total_bytes").is_some());
        assert!(parsed["storage"].get("recording_count").is_some());
    }
}
