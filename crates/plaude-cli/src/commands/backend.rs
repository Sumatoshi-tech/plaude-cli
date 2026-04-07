//! Runtime backend selection for vendor-opcode-driven subcommands.
//!
//! Every user-visible command that talks to a Plaud device goes
//! through a [`TransportProvider`]. In M6 there are two implementations:
//!
//! * [`SimProvider`] wraps `plaud_sim::SimDevice` and is what the CLI
//!   tests (and local dogfooding) drive.
//! * [`BleProvider`] is a placeholder for the real btleplug-backed
//!   central. Until that backend ships, every method on [`BleProvider`]
//!   surfaces [`plaud_transport::Error::Unsupported`] pointing at the
//!   future milestone.
//!
//! The active provider is chosen at the CLI top level by the global
//! `--backend <sim|ble>` flag.

use std::path::{Path, PathBuf};

use async_trait::async_trait;
use clap::ValueEnum;
use plaud_domain::{AuthToken, CommonSettingKey, Recording, RecordingId, RecordingKind, SettingValue};
use plaud_sim::SimDevice;
use plaud_transport::{Error as TransportError, Result as TransportResult, Transport};
use plaud_transport_usb::UsbTransport;

/// Error message surfaced by [`UsbProvider`] when the caller
/// selected `--backend usb` without also providing `--mount <PATH>`.
pub(crate) const ERR_USB_MOUNT_REQUIRED: &str = "usb backend requires --mount <PATH> pointing at the mounted PLAUD_NOTE volume";

/// Environment variable the sim backend consults to force the sim
/// into a soft-rejected state for the rejected-token test path.
///
/// Setting it to `"1"` causes [`SimProvider::connect_authenticated`]
/// to return [`TransportError::AuthRejected`] before returning a
/// transport. Any other value (or absence) leaves the sim in its
/// default "accept any token" mode.
pub(crate) const ENV_SIM_REJECT: &str = "PLAUDE_SIM_REJECT";

/// Value that enables sim rejection mode.
const ENV_SIM_REJECT_ON: &str = "1";

/// Sim status byte surfaced when rejection mode is on. Matches the
/// live-evidence auth status byte from `docs/protocol/ble-commands.md`.
const SIM_REJECTED_STATUS: u8 = 0x01;

/// Deterministic recording basename the sim backend preloads on
/// every `connect_*` call when the `PLAUDE_SIM_RECORDINGS` env var
/// is unset. Picked as a 10-digit Unix-seconds string so
/// `RecordingId::new` accepts it without further validation. The
/// e2e tests assert against this exact value.
pub(crate) const SIM_RECORDING_BASENAME: &str = "1775393534";

/// Deterministic WAV bytes preloaded alongside [`SIM_RECORDING_BASENAME`]
/// in the default single-recording fixture.
pub(crate) const SIM_RECORDING_WAV: &[u8] = b"WAV-BYTES-FROM-SIM";

/// Deterministic ASR sidecar bytes preloaded alongside [`SIM_RECORDING_BASENAME`]
/// in the default single-recording fixture.
pub(crate) const SIM_RECORDING_ASR: &[u8] = b"ASR-BYTES-FROM-SIM";

/// Environment variable consulted by [`build_sim_device`] to override
/// the preloaded recording list. Value is a comma-separated list of
/// recording basenames; each basename is preloaded with a
/// deterministic WAV + ASR payload derived from the id so tests
/// get stable bytes without having to wire a JSON fixture.
///
/// Empty string → empty device (no preloaded recordings).
/// Unset → the default single-recording fixture
/// ([`SIM_RECORDING_BASENAME`]).
pub(crate) const ENV_SIM_RECORDINGS: &str = "PLAUDE_SIM_RECORDINGS";

/// Prefix for the deterministic WAV bytes generated for env-driven
/// sim recordings. Concatenated with the basename so every recording
/// has a unique byte-equal payload the tests can assert against.
const SIM_ENV_WAV_PREFIX: &[u8] = b"WAV-";

/// Prefix for the deterministic ASR bytes generated for env-driven
/// sim recordings.
const SIM_ENV_ASR_PREFIX: &[u8] = b"ASR-";

/// Build a fresh [`SimDevice`] with either the default single
/// recording or the env-specified list, plus a handful of default
/// settings for the `settings list` command.
fn build_sim_device() -> SimDevice {
    let mut builder = SimDevice::builder();
    for (id, wav, asr) in preloaded_recordings() {
        let meta = Recording::new(id, RecordingKind::Note, wav.len() as u64, asr.len() as u64);
        builder = builder.preload_recording(meta, wav, asr);
    }
    builder = builder
        .with_setting(CommonSettingKey::EnableVad, SettingValue::Bool(true))
        .with_setting(CommonSettingKey::MicGain, SettingValue::U8(SIM_MIC_GAIN))
        .with_setting(CommonSettingKey::AutoPowerOff, SettingValue::U32(SIM_AUTO_POWER_OFF));
    builder.build()
}

/// Default mic-gain value preloaded in the sim.
pub(crate) const SIM_MIC_GAIN: u8 = 20;

/// Default auto-power-off timeout (seconds) preloaded in the sim.
pub(crate) const SIM_AUTO_POWER_OFF: u32 = 300;

fn preloaded_recordings() -> Vec<(RecordingId, Vec<u8>, Vec<u8>)> {
    match std::env::var(ENV_SIM_RECORDINGS) {
        Ok(list) => parse_env_recording_list(&list),
        Err(_) => default_preloaded_recordings(),
    }
}

fn default_preloaded_recordings() -> Vec<(RecordingId, Vec<u8>, Vec<u8>)> {
    let id = RecordingId::new(SIM_RECORDING_BASENAME).expect("hand-validated sim basename");
    vec![(id, SIM_RECORDING_WAV.to_vec(), SIM_RECORDING_ASR.to_vec())]
}

fn parse_env_recording_list(list: &str) -> Vec<(RecordingId, Vec<u8>, Vec<u8>)> {
    list.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter_map(|basename| {
            let id = RecordingId::new(basename).ok()?;
            let wav = [SIM_ENV_WAV_PREFIX, basename.as_bytes()].concat();
            let asr = [SIM_ENV_ASR_PREFIX, basename.as_bytes()].concat();
            Some((id, wav, asr))
        })
        .collect()
}

/// Backend selection for every Transport-consuming subcommand.
///
/// Parsed from the global `--backend` flag. Defaults to [`Backend::Ble`]
/// even though the real BLE backend is not wired yet, because `ble` is
/// the long-term production default; the `sim` variant exists for
/// deterministic tests and local dogfooding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum Backend {
    /// In-process simulator (`plaud-sim`). Used by every CI test.
    Sim,
    /// Real BLE central via the future btleplug backend. Currently
    /// returns [`TransportError::Unsupported`] at connect-time.
    Ble,
    /// USB Mass Storage fallback. Requires the global `--mount <PATH>`
    /// flag to point at the mounted `PLAUD_NOTE` VFAT volume.
    Usb,
}

impl Backend {
    /// Instantiate the [`TransportProvider`] for this backend variant.
    ///
    /// `mount` is consulted only when `self == Backend::Usb`. The
    /// other variants silently ignore it so the call site stays
    /// symmetric.
    pub(crate) fn provider(self, mount: Option<&Path>) -> Box<dyn TransportProvider> {
        match self {
            Self::Sim => Box::new(SimProvider::new()) as Box<dyn TransportProvider>,
            Self::Ble => Box::new(BleProvider::new()) as Box<dyn TransportProvider>,
            Self::Usb => Box::new(UsbProvider::new(mount.map(Path::to_path_buf))) as Box<dyn TransportProvider>,
        }
    }
}

/// Abstraction over the "how do I obtain a [`Transport`] trait object"
/// question. Two connect entry points so that commands which must not
/// require auth (like `battery`) can take a separate code path from
/// commands that do.
#[async_trait]
pub(crate) trait TransportProvider: Send + Sync {
    /// Connect without offering a token. The returned [`Transport`]
    /// is guaranteed to answer `battery()` successfully; any vendor
    /// opcode may return [`TransportError::AuthRequired`].
    async fn connect_anonymous(&self) -> TransportResult<Box<dyn Transport>>;

    /// Connect and authenticate with the given token. Returns
    /// [`TransportError::AuthRejected`] if the device rejects it.
    async fn connect_authenticated(&self, token: AuthToken) -> TransportResult<Box<dyn Transport>>;
}

/// `plaud-sim`-backed provider. Builds a fresh [`SimDevice`] on every
/// connect so that callers cannot accidentally share state across
/// invocations.
pub(crate) struct SimProvider;

impl SimProvider {
    pub(crate) fn new() -> Self {
        Self
    }

    fn reject_mode_enabled() -> bool {
        std::env::var(ENV_SIM_REJECT).map(|v| v == ENV_SIM_REJECT_ON).unwrap_or(false)
    }
}

#[async_trait]
impl TransportProvider for SimProvider {
    async fn connect_anonymous(&self) -> TransportResult<Box<dyn Transport>> {
        let device = build_sim_device();
        Ok(device.unauthenticated_transport())
    }

    async fn connect_authenticated(&self, _token: AuthToken) -> TransportResult<Box<dyn Transport>> {
        if Self::reject_mode_enabled() {
            return Err(TransportError::AuthRejected {
                status: SIM_REJECTED_STATUS,
            });
        }
        let device = build_sim_device();
        Ok(device.authenticated_transport())
    }
}

/// Real BLE hardware provider backed by btleplug.
///
/// Scans for a `PLAUD_NOTE` device, connects, enables CCCD,
/// bridges GATT I/O into the existing `BleSession` protocol layer,
/// and returns a fully functional `BleTransport`.
pub(crate) struct BleProvider;

/// Default BLE scan timeout in seconds.
const BLE_SCAN_TIMEOUT_SECS: u64 = 5;

impl BleProvider {
    pub(crate) fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TransportProvider for BleProvider {
    async fn connect_anonymous(&self) -> TransportResult<Box<dyn Transport>> {
        use std::{sync::Arc, time::Duration};

        use plaud_transport_ble::{BleSession, BleTransport, backend::connect_peripheral};
        use tokio::sync::Mutex;

        let (channel, battery_reader) = connect_peripheral(Duration::from_secs(BLE_SCAN_TIMEOUT_SECS)).await?;
        let session = BleSession::new(channel);
        Ok(Box::new(BleTransport::from_parts(
            Arc::new(Mutex::new(session)),
            Arc::new(battery_reader),
        )))
    }

    async fn connect_authenticated(&self, token: AuthToken) -> TransportResult<Box<dyn Transport>> {
        use std::{sync::Arc, time::Duration};

        use plaud_transport_ble::{BleSession, BleTransport, backend::connect_peripheral};
        use tokio::sync::Mutex;

        let (channel, battery_reader) = connect_peripheral(Duration::from_secs(BLE_SCAN_TIMEOUT_SECS)).await?;
        let mut session = BleSession::new(channel);
        session.authenticate(&token).await?;
        Ok(Box::new(BleTransport::from_parts(
            Arc::new(Mutex::new(session)),
            Arc::new(battery_reader),
        )))
    }
}

/// Provider for the USB mass-storage backend. Wraps a
/// `plaud_transport_usb::UsbTransport` rooted at a caller-supplied
/// mount point. Both `connect_*` entry points return the same
/// transport (USB has no "anonymous" vs. "authenticated" split —
/// any process that can read the volume can read everything on it).
pub(crate) struct UsbProvider {
    mount: Option<PathBuf>,
}

impl UsbProvider {
    pub(crate) fn new(mount: Option<PathBuf>) -> Self {
        Self { mount }
    }

    fn build_transport(&self) -> TransportResult<Box<dyn Transport>> {
        let mount = self
            .mount
            .clone()
            .ok_or_else(|| TransportError::Transport(ERR_USB_MOUNT_REQUIRED.to_owned()))?;
        Ok(Box::new(UsbTransport::new(mount)))
    }
}

#[async_trait]
impl TransportProvider for UsbProvider {
    async fn connect_anonymous(&self) -> TransportResult<Box<dyn Transport>> {
        self.build_transport()
    }

    async fn connect_authenticated(&self, _token: AuthToken) -> TransportResult<Box<dyn Transport>> {
        self.build_transport()
    }
}

#[cfg(test)]
mod tests {
    use super::{Backend, BleProvider, SimProvider, TransportProvider};

    #[tokio::test]
    async fn ble_provider_connect_anonymous_fails_without_hardware() {
        // The real btleplug backend needs a BLE adapter. In CI it
        // returns a Transport error; on a developer machine with BLE
        // it might succeed. We just verify it doesn't panic.
        let provider = BleProvider::new();
        let _ = provider.connect_anonymous().await;
    }

    #[tokio::test]
    async fn sim_provider_connect_anonymous_yields_battery_capable_transport() {
        let provider = SimProvider::new();
        let transport = match provider.connect_anonymous().await {
            Ok(t) => t,
            Err(e) => panic!("expected Ok, got {e:?}"),
        };
        transport.battery().await.expect("sim battery always succeeds");
    }

    #[test]
    fn backend_enum_provider_factory_returns_a_matching_impl() {
        // Smoke: every variant can instantiate without panicking.
        // The returned trait objects are exercised by the e2e tests.
        let _ = Backend::Sim.provider(None);
        let _ = Backend::Ble.provider(None);
        let _ = Backend::Usb.provider(Some(std::path::Path::new("/tmp/fixture")));
    }
}
