//! plaude-cli — the `plaude-cli` binary entry point.
//!
//! Offline CLI for the Plaud Note voice recorder. Talks to Plaud
//! hardware directly over BLE, USB, or Wi-Fi without relying on
//! plaud.ai cloud or the Plaud phone app for day-to-day operation.
//!
//! M4 added the `auth` subcommand tree. M6 adds `battery` and
//! `device info` plus the global `--backend` flag. M11 adds
//! `settings`, `record`, `device privacy`, and `device name`.

#![deny(missing_docs)]

mod commands;

use std::{path::PathBuf, process::ExitCode};

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use plaud_transport::Error as TransportError;
use plaud_transport_usb::USB_DEPRECATION_NOTICE;
use tracing_subscriber::{EnvFilter, fmt};

use crate::commands::backend::Backend;

/// Exit code used when the CLI is invoked without any arguments or
/// with an invalid input.
///
/// CLIG (<https://clig.dev/#the-basics>) and `sysexits(3)` both define
/// `2` as the "command was used incorrectly" exit code.
const EXIT_USAGE: u8 = 2;

/// Exit code for genuinely runtime failures.
const EXIT_RUNTIME: u8 = 1;

/// Exit code when a vendor command was issued without a stored token.
/// Matches `sysexits(3)` `EX_NOPERM` (77).
const EXIT_AUTH_REQUIRED: u8 = 77;

/// Exit code when the device rejected the stored token. Matches
/// `sysexits(3)` `EX_CONFIG` (78) — the configuration (token) is
/// invalid for the running device.
const EXIT_AUTH_REJECTED: u8 = 78;

/// Exit code when the transport layer itself is unavailable
/// (connection failed, BLE not wired, device not reachable).
/// Matches `sysexits(3)` `EX_UNAVAILABLE` (69).
const EXIT_UNAVAILABLE: u8 = 69;

/// Minimum number of argv entries that represents an actual user
/// intent — we always have at least the program name (`argv[0]`).
const MIN_USER_ARGS: usize = 2;

/// Hint appended to the missing-token error so the user knows where
/// to go next.
const HINT_RUN_AUTH_HELP: &str = "run `plaude-cli auth --help` to store a token";

/// Hint appended to the rejected-token error. `bootstrap` is the
/// name of the future M8 onboarding command.
const HINT_RUN_BOOTSTRAP: &str = "run `plaude-cli auth bootstrap` or re-import a fresh token";

/// Default timeout in seconds for transport operations.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Privacy disclosure printed by `--about`.
const PRIVACY_DISCLOSURE: &str = "\
plaude-cli — Offline CLI for the Plaud Note voice recorder.

PRIVACY NOTICE:

  1. BLE traffic is CLEARTEXT on V0095 firmware. Anyone with a BLE
     sniffer within ~10 m can record your file data and the auth token
     during a sync. Do not sync in hostile physical environments.

  2. Every WAV file on a Plaud Note contains the device's 18-digit
     serial number embedded in a custom RIFF `pad` chunk. Use
     `--sanitise` on `sync` to scrub it on copy.

  3. The BLE auth token is a long-lived per-device credential stored
     in your OS keyring (or ~/.config/plaude/token mode 0600). Treat
     it like a password.

See docs/protocol/overview.md for the full security model.";

/// Offline CLI for the Plaud Note voice recorder.
#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Override the directory where the file-backed token store
    /// looks for (and writes) the token file. Defaults to the
    /// platform-standard config dir (`$XDG_CONFIG_HOME/plaude` on
    /// Linux). Used primarily by tests to sandbox `HOME`.
    #[arg(long, global = true, value_name = "PATH")]
    config_dir: Option<PathBuf>,

    /// Runtime backend: `sim` uses the in-process simulator, `ble` is reserved for the future real-hardware backend, `usb` mounts a pre-deprecation PLAUD_NOTE VFAT volume.
    #[arg(long, global = true, value_enum, default_value_t = Backend::Ble)]
    backend: Backend,

    /// Mount path for the `usb` backend, e.g. `/run/media/alice/PLAUD_NOTE`. Ignored for every other backend.
    #[arg(long, global = true, value_name = "PATH")]
    mount: Option<PathBuf>,

    /// Log output format. `text` is human-readable; `json` emits one JSON object per event. Activated by setting `RUST_LOG`.
    #[arg(long, global = true, value_enum, default_value_t = LogFormat::Text)]
    log_format: LogFormat,

    /// Timeout in seconds for transport operations. Also settable via `PLAUDE_TIMEOUT`. Default: 30.
    #[arg(long, global = true, value_name = "SECS", env = "PLAUDE_TIMEOUT")]
    timeout: Option<u64>,

    /// Print privacy disclosure and project information, then exit.
    #[arg(long)]
    about: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

/// Log output format for the `--log-format` flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum LogFormat {
    /// Human-readable text on stderr.
    Text,
    /// Machine-parseable JSON lines on stderr.
    Json,
}

/// Top-level subcommands.
#[derive(Debug, Subcommand)]
enum Commands {
    /// Manage the stored BLE authentication token.
    #[command(subcommand)]
    Auth(commands::auth::AuthCommand),
    /// Print the device battery percentage.
    Battery(commands::battery::BatteryCommand),
    /// Query or control the connected Plaud device.
    #[command(subcommand)]
    Device(commands::device::DeviceCommand),
    /// List and pull recordings off the connected Plaud device.
    #[command(subcommand)]
    Files(commands::files::FilesCommand),
    /// Remote recording pipeline control (start/stop/pause/resume).
    #[command(subcommand)]
    Record(commands::record::RecordCommand),
    /// Read and write device settings.
    #[command(subcommand)]
    Settings(commands::settings::SettingsCommand),
    /// Mirror every recording on the device into a local directory.
    Sync(commands::sync::SyncArgs),
    /// Transcribe WAV files to text using a local whisper.cpp binary.
    Transcribe(commands::transcribe::TranscribeArgs),
}

fn main() -> ExitCode {
    if std::env::args().count() < MIN_USER_ARGS {
        let mut cmd = Cli::command();
        let _ = cmd.print_help();
        println!();
        return ExitCode::from(EXIT_USAGE);
    }
    let cli = Cli::parse();
    init_logging(cli.log_format);
    if cli.about {
        println!("{PRIVACY_DISCLOSURE}");
        return ExitCode::SUCCESS;
    }
    match dispatch(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(DispatchError::Usage(message)) => {
            eprintln!("{message}");
            ExitCode::from(EXIT_USAGE)
        }
        Err(DispatchError::Runtime(message)) => {
            eprintln!("{message}");
            ExitCode::from(EXIT_RUNTIME)
        }
        Err(DispatchError::Unavailable(message)) => {
            eprintln!("{message}");
            ExitCode::from(EXIT_UNAVAILABLE)
        }
        Err(DispatchError::AuthRequired) => {
            eprintln!("no auth token stored for this device — {HINT_RUN_AUTH_HELP}");
            ExitCode::from(EXIT_AUTH_REQUIRED)
        }
        Err(DispatchError::AuthRejected { status }) => {
            eprintln!("device rejected the stored token (status byte 0x{status:02X}) — {HINT_RUN_BOOTSTRAP}");
            ExitCode::from(EXIT_AUTH_REJECTED)
        }
    }
}

/// Initialise the `tracing-subscriber` logging layer.
///
/// Logging is silent unless `RUST_LOG` is set. When active, logs go
/// to stderr so stdout remains clean for structured command output.
fn init_logging(format: LogFormat) {
    let filter = EnvFilter::from_default_env();
    match format {
        LogFormat::Text => {
            fmt::Subscriber::builder()
                .with_env_filter(filter)
                .with_writer(std::io::stderr)
                .with_target(true)
                .init();
        }
        LogFormat::Json => {
            fmt::Subscriber::builder()
                .with_env_filter(filter)
                .with_writer(std::io::stderr)
                .json()
                .init();
        }
    }
}

/// Dispatch CLI commands to their handlers, building a fresh tokio
/// runtime on demand for async operations.
fn dispatch(cli: Cli) -> Result<(), DispatchError> {
    let _timeout_secs = cli.timeout.unwrap_or(DEFAULT_TIMEOUT_SECS);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| DispatchError::Runtime(format!("failed to build runtime: {e}")))?;
    let config_dir = cli.config_dir.as_deref();
    if cli.backend == Backend::Usb {
        eprintln!("{USB_DEPRECATION_NOTICE}");
    }
    match cli.command {
        None => Err(DispatchError::Usage("no subcommand supplied; run `plaude-cli --help`".to_owned())),
        Some(Commands::Auth(auth_cmd)) => runtime.block_on(commands::auth::run(auth_cmd, config_dir, cli.backend)),
        Some(Commands::Battery(battery_cmd)) => {
            let provider = cli.backend.provider(cli.mount.as_deref());
            runtime.block_on(commands::battery::run(battery_cmd, provider.as_ref()))
        }
        Some(Commands::Device(device_cmd)) => {
            let provider = cli.backend.provider(cli.mount.as_deref());
            runtime.block_on(commands::device::run(device_cmd, provider.as_ref(), config_dir))
        }
        Some(Commands::Files(files_cmd)) => {
            let provider = cli.backend.provider(cli.mount.as_deref());
            runtime.block_on(commands::files::run(files_cmd, provider.as_ref(), config_dir))
        }
        Some(Commands::Record(record_cmd)) => {
            let provider = cli.backend.provider(cli.mount.as_deref());
            runtime.block_on(commands::record::run(record_cmd, provider.as_ref(), config_dir))
        }
        Some(Commands::Settings(settings_cmd)) => {
            let provider = cli.backend.provider(cli.mount.as_deref());
            runtime.block_on(commands::settings::run(settings_cmd, provider.as_ref(), config_dir))
        }
        Some(Commands::Sync(sync_args)) => {
            let provider = cli.backend.provider(cli.mount.as_deref());
            runtime.block_on(commands::sync::run(sync_args, provider.as_ref(), config_dir))
        }
        Some(Commands::Transcribe(transcribe_args)) => commands::transcribe::run(transcribe_args),
    }
}

/// Errors surfaced by `dispatch` back to `main` for exit-code mapping.
#[derive(Debug)]
pub(crate) enum DispatchError {
    /// The caller misused the CLI — missing subcommand, bad input,
    /// etc. Maps to `EXIT_USAGE` (2).
    Usage(String),
    /// A runtime failure — I/O, keyring unavailable, parse error.
    /// Maps to `EXIT_RUNTIME` (1).
    Runtime(String),
    /// The transport layer itself is unreachable — BLE not wired,
    /// device not found, connection dropped. Maps to
    /// `EXIT_UNAVAILABLE` (69, `EX_UNAVAILABLE`).
    Unavailable(String),
    /// A command required a token but none was stored. Maps to
    /// `EXIT_AUTH_REQUIRED` (77, `EX_NOPERM`).
    AuthRequired,
    /// The device rejected the stored token. Maps to
    /// `EXIT_AUTH_REJECTED` (78, `EX_CONFIG`).
    AuthRejected {
        /// Raw auth status byte from the device response.
        status: u8,
    },
}

impl DispatchError {
    /// Map a [`TransportError`] into the CLI's dispatch error enum
    /// so each failure mode reaches the right exit code.
    pub(crate) fn from_transport_error(err: &TransportError) -> Self {
        match err {
            TransportError::AuthRequired => Self::AuthRequired,
            TransportError::AuthRejected { status } => Self::AuthRejected { status: *status },
            TransportError::Unsupported { capability } => Self::Unavailable(format!("capability not supported: {capability}")),
            TransportError::Timeout { seconds } => Self::Unavailable(format!("operation timed out after {seconds}s")),
            TransportError::Transport(msg) => Self::Unavailable(format!("transport error: {msg}")),
            other => Self::Runtime(format!("{other}")),
        }
    }
}
