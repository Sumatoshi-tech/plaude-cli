//! `plaude settings` — read and write device settings.
//!
//! Settings are identified by their human-readable name (as returned
//! by [`CommonSettingKey::name`]) and carry a [`SettingValue`] union
//! that the CLI renders in text or JSON form.
//!
//! Journey: specs/plaude-v1/journeys/M11-settings-record-control.md

use std::path::Path;

use clap::{Args, Subcommand};
use plaud_auth::DEFAULT_DEVICE_ID;
use plaud_domain::{AuthToken, CommonSettingKey, SettingValue};
use serde::Serialize;

use crate::{
    DispatchError,
    commands::{auth::build_store, backend::TransportProvider, output::OutputFormat},
};

/// Message printed on a successful `set` operation.
const SET_OK_PREFIX: &str = " = ";

/// `plaude settings` subcommand tree.
#[derive(Debug, Subcommand)]
pub(crate) enum SettingsCommand {
    /// List all settings that have a stored value on the device.
    List(SettingsListArgs),
    /// Read a single setting by name.
    Get(SettingsGetArgs),
    /// Write a single setting.
    Set(SettingsSetArgs),
}

/// Arguments for `plaude settings list`.
#[derive(Debug, Args)]
pub(crate) struct SettingsListArgs {
    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    output: OutputFormat,
}

/// Arguments for `plaude settings get`.
#[derive(Debug, Args)]
pub(crate) struct SettingsGetArgs {
    /// Setting name (e.g. `enable-vad`, `mic-gain`).
    name: String,
    /// Output format.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    output: OutputFormat,
}

/// Arguments for `plaude settings set`.
#[derive(Debug, Args)]
pub(crate) struct SettingsSetArgs {
    /// Setting name (e.g. `enable-vad`, `mic-gain`).
    name: String,
    /// Value to write (boolean, u8, or u32).
    value: String,
}

/// A single setting entry in JSON output.
#[derive(Debug, Serialize)]
struct SettingJson {
    name: &'static str,
    value: String,
}

/// Entry point dispatched from `main::dispatch`.
pub(crate) async fn run(cmd: SettingsCommand, provider: &dyn TransportProvider, config_dir: Option<&Path>) -> Result<(), DispatchError> {
    match cmd {
        SettingsCommand::List(args) => list(args, provider, config_dir).await,
        SettingsCommand::Get(args) => get(args, provider, config_dir).await,
        SettingsCommand::Set(args) => set(args, provider, config_dir).await,
    }
}

async fn load_token(config_dir: Option<&Path>) -> Result<AuthToken, DispatchError> {
    let store = build_store(config_dir)?;
    let stored = store
        .get_token(DEFAULT_DEVICE_ID)
        .await
        .map_err(|e| DispatchError::Runtime(format!("failed to read token store: {e}")))?;
    stored.ok_or(DispatchError::AuthRequired)
}

async fn list(args: SettingsListArgs, provider: &dyn TransportProvider, config_dir: Option<&Path>) -> Result<(), DispatchError> {
    let token = load_token(config_dir).await?;
    let transport = provider
        .connect_authenticated(token)
        .await
        .map_err(|e| DispatchError::from_transport_error(&e))?;

    let mut entries: Vec<(&'static str, SettingValue)> = Vec::new();
    for &key in CommonSettingKey::all() {
        match transport.read_setting(key).await {
            Ok(value) => entries.push((key.name(), value)),
            Err(plaud_transport::Error::NotFound(_)) => continue,
            Err(e) => return Err(DispatchError::from_transport_error(&e)),
        }
    }

    match args.output {
        OutputFormat::Text => {
            for (name, value) in &entries {
                println!("{name}{SET_OK_PREFIX}{value}");
            }
        }
        OutputFormat::Json => {
            let payload: Vec<SettingJson> = entries
                .iter()
                .map(|(name, value)| SettingJson {
                    name,
                    value: format!("{value}"),
                })
                .collect();
            let rendered = serde_json::to_string(&payload).map_err(|e| DispatchError::Runtime(format!("json encode: {e}")))?;
            println!("{rendered}");
        }
    }
    Ok(())
}

async fn get(args: SettingsGetArgs, provider: &dyn TransportProvider, config_dir: Option<&Path>) -> Result<(), DispatchError> {
    let key = CommonSettingKey::from_name(&args.name).map_err(|e| DispatchError::Usage(format!("{e}")))?;
    let token = load_token(config_dir).await?;
    let transport = provider
        .connect_authenticated(token)
        .await
        .map_err(|e| DispatchError::from_transport_error(&e))?;

    let value = transport
        .read_setting(key)
        .await
        .map_err(|e| DispatchError::from_transport_error(&e))?;

    match args.output {
        OutputFormat::Text => println!("{}{SET_OK_PREFIX}{value}", key.name()),
        OutputFormat::Json => {
            let payload = SettingJson {
                name: key.name(),
                value: format!("{value}"),
            };
            let rendered = serde_json::to_string(&payload).map_err(|e| DispatchError::Runtime(format!("json encode: {e}")))?;
            println!("{rendered}");
        }
    }
    Ok(())
}

async fn set(args: SettingsSetArgs, provider: &dyn TransportProvider, config_dir: Option<&Path>) -> Result<(), DispatchError> {
    let key = CommonSettingKey::from_name(&args.name).map_err(|e| DispatchError::Usage(format!("{e}")))?;
    let value = SettingValue::parse(&args.value).map_err(|e| DispatchError::Usage(format!("{e}")))?;
    let token = load_token(config_dir).await?;
    let transport = provider
        .connect_authenticated(token)
        .await
        .map_err(|e| DispatchError::from_transport_error(&e))?;

    transport
        .write_setting(key, value)
        .await
        .map_err(|e| DispatchError::from_transport_error(&e))?;

    println!("{}{SET_OK_PREFIX}{value}", key.name());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::SettingJson;

    #[test]
    fn setting_json_carries_name_and_value_keys() {
        let payload = SettingJson {
            name: "enable-vad",
            value: "true".to_owned(),
        };
        let rendered = serde_json::to_string(&payload).expect("encode");
        let parsed: serde_json::Value = serde_json::from_str(&rendered).expect("parse");
        assert!(parsed.get("name").is_some());
        assert!(parsed.get("value").is_some());
    }
}
