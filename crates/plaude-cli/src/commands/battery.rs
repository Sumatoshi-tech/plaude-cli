//! `plaude-cli battery` — print the device battery percentage.
//!
//! Hits the SIG-analogue battery path that does **not** require an
//! auth token (matching Test 2b live evidence). Supports `--output
//! text` (default) and `--output json`.

use clap::Args;
use plaud_domain::BatteryLevel;
use serde::Serialize;

use crate::{
    DispatchError,
    commands::{backend::TransportProvider, output::OutputFormat},
};

/// Text prefix for the human-readable battery line.
const BATTERY_TEXT_PREFIX: &str = "Battery: ";

/// `plaude-cli battery` subcommand arguments.
#[derive(Debug, Args)]
pub(crate) struct BatteryCommand {
    /// Output format. `text` is the default human-readable form;
    /// `json` emits a single-line object suitable for piping to `jq`.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    output: OutputFormat,
}

/// JSON projection for the battery payload. A dedicated struct
/// (rather than an ad-hoc map) pins the field names so renames are
/// caught by the type system.
#[derive(Debug, Serialize)]
struct BatteryJson {
    percent: u8,
}

/// Entry point invoked from `main::dispatch` after the tokio runtime
/// is built.
pub(crate) async fn run(cmd: BatteryCommand, provider: &dyn TransportProvider) -> Result<(), DispatchError> {
    let transport = provider
        .connect_anonymous()
        .await
        .map_err(|e| DispatchError::from_transport_error(&e))?;
    let level = transport.battery().await.map_err(|e| DispatchError::from_transport_error(&e))?;
    print_battery(level, cmd.output)
}

fn print_battery(level: BatteryLevel, output: OutputFormat) -> Result<(), DispatchError> {
    match output {
        OutputFormat::Text => {
            println!("{BATTERY_TEXT_PREFIX}{}%", level.percent());
            Ok(())
        }
        OutputFormat::Json => {
            let payload = BatteryJson { percent: level.percent() };
            let rendered = serde_json::to_string(&payload).map_err(|e| DispatchError::Runtime(format!("json encode: {e}")))?;
            println!("{rendered}");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use plaud_domain::BatteryLevel;
    use serde_json::Value;

    use super::{BATTERY_TEXT_PREFIX, BatteryJson};

    const SAMPLE_PERCENT: u8 = 42;

    #[test]
    fn battery_json_round_trips_percent_key() {
        let payload = BatteryJson { percent: SAMPLE_PERCENT };
        let rendered = serde_json::to_string(&payload).expect("encode");
        let parsed: Value = serde_json::from_str(&rendered).expect("parse");
        assert_eq!(parsed["percent"], SAMPLE_PERCENT);
    }

    #[test]
    fn battery_text_prefix_is_stable() {
        // Mutation target: if someone renames the prefix to "Level: "
        // this test fails and callers of `grep Battery:` are warned.
        assert_eq!(BATTERY_TEXT_PREFIX, "Battery: ");
        let level = BatteryLevel::new(SAMPLE_PERCENT).expect("valid");
        assert_eq!(level.percent(), SAMPLE_PERCENT);
    }
}
