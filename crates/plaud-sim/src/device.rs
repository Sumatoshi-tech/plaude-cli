//! Public entry points: [`SimDevice`] and [`SimDeviceBuilder`].
//!
//! A test builds a `SimDevice` via its builder, then asks the device
//! either for a shortcut [`Transport`] handle (
//! [`SimDevice::authenticated_transport`]) or for a full
//! [`crate::SimDiscovery`] that exercises the scan + connect + auth
//! flow end to end.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

use plaud_domain::{AuthToken, BatteryLevel, CommonSettingKey, DeviceInfo, Recording, RecordingId, SettingValue, StorageStats};
use plaud_transport::Transport;

use crate::{
    constants::DEFAULT_BATTERY_PERCENT,
    discovery::SimDiscovery,
    state::{AuthState, FailureInjection, PreloadedRecording, RecordingState, SimState},
    transport::SimTransport,
};

/// An in-process simulator of a Plaud device. Cheap to clone — the
/// inner state is wrapped in an `Arc`, so every clone sees the same
/// state and every transport handle talks to the same device.
#[derive(Debug, Clone)]
pub struct SimDevice {
    state: Arc<Mutex<SimState>>,
}

impl SimDevice {
    /// Start building a sim.
    #[must_use]
    pub fn builder() -> SimDeviceBuilder {
        SimDeviceBuilder::default()
    }

    /// Return a trait object that pretends it has already
    /// authenticated. Intended as a convenience for tests that only
    /// want to verify the business logic of a `Transport` method
    /// without rerunning the auth flow on every call.
    ///
    /// Calling this method also resets [`AuthState`] to `Accepted`
    /// on the underlying state.
    #[must_use]
    pub fn authenticated_transport(&self) -> Box<dyn Transport> {
        self.set_auth_state(AuthState::Accepted);
        Box::new(SimTransport::new(Arc::clone(&self.state)))
    }

    /// Return a trait object that has **not** authenticated. Useful
    /// for verifying that battery (standard SIG service analogue)
    /// still succeeds without auth, and that every vendor opcode
    /// returns `Error::AuthRequired`.
    #[must_use]
    pub fn unauthenticated_transport(&self) -> Box<dyn Transport> {
        self.set_auth_state(AuthState::Unauthenticated);
        Box::new(SimTransport::new(Arc::clone(&self.state)))
    }

    /// Build a discovery handle that exercises the scan, connect, and
    /// auth flow against the configured expected token. The caller
    /// supplies the token the discovery will attempt to present.
    #[must_use]
    pub fn discovery(&self, offered_token: AuthToken) -> SimDiscovery {
        SimDiscovery::new(Arc::clone(&self.state), offered_token)
    }

    /// Read the preloaded ASR sidecar bytes for a recording.
    ///
    /// Sim-only accessor. Tests use it to assert the sim preserved
    /// the bytes they provided at build time. The `Transport` trait
    /// does not currently surface ASR bytes — that is an M7 concern.
    #[must_use]
    pub fn asr_bytes_for(&self, id: &RecordingId) -> Option<Vec<u8>> {
        let state = lock_state(&self.state);
        state.recordings.get(id).map(|r| r.asr.clone())
    }

    /// Observe how many transport operations have been served so
    /// far. Used by failure-injection tests to verify the counter
    /// advances on every call.
    #[must_use]
    pub fn op_count(&self) -> u32 {
        lock_state(&self.state).op_count
    }

    fn set_auth_state(&self, new_state: AuthState) {
        let mut state = lock_state(&self.state);
        state.auth = new_state;
    }
}

/// Fluent builder for a [`SimDevice`].
#[derive(Debug, Clone)]
pub struct SimDeviceBuilder {
    device_info: DeviceInfo,
    battery: BatteryLevel,
    storage: StorageStats,
    recordings: Vec<PreloadedRecording>,
    settings: HashMap<CommonSettingKey, SettingValue>,
    expected_token: Option<AuthToken>,
    privacy_on: bool,
    failure: FailureInjection,
}

impl Default for SimDeviceBuilder {
    fn default() -> Self {
        Self {
            device_info: DeviceInfo::placeholder(),
            battery: default_battery(),
            storage: StorageStats::ZERO,
            recordings: Vec::new(),
            settings: HashMap::new(),
            expected_token: None,
            privacy_on: false,
            failure: FailureInjection::default(),
        }
    }
}

impl SimDeviceBuilder {
    /// Override the simulated device identity. Defaults to
    /// [`DeviceInfo::placeholder`].
    #[must_use]
    pub fn with_device_info(mut self, info: DeviceInfo) -> Self {
        self.device_info = info;
        self
    }

    /// Override the simulated battery percentage. Defaults to 100%.
    #[must_use]
    pub fn with_battery(mut self, battery: BatteryLevel) -> Self {
        self.battery = battery;
        self
    }

    /// Override the simulated storage counters. Defaults to
    /// [`StorageStats::ZERO`].
    #[must_use]
    pub fn with_storage(mut self, storage: StorageStats) -> Self {
        self.storage = storage;
        self
    }

    /// Preload a recording. `wav` and `asr` are both stored; the
    /// `Transport::read_recording` method returns `wav`; tests that
    /// need the ASR bytes call [`SimDevice::asr_bytes_for`].
    #[must_use]
    pub fn preload_recording(mut self, meta: Recording, wav: Vec<u8>, asr: Vec<u8>) -> Self {
        self.recordings.push(PreloadedRecording { meta, wav, asr });
        self
    }

    /// Prepopulate one device setting.
    #[must_use]
    pub fn with_setting(mut self, key: CommonSettingKey, value: SettingValue) -> Self {
        self.settings.insert(key, value);
        self
    }

    /// Configure the token that authentication will accept. If
    /// unset, the sim auto-accepts any token.
    #[must_use]
    pub fn with_expected_token(mut self, token: AuthToken) -> Self {
        self.expected_token = Some(token);
        self
    }

    /// Configure the initial privacy flag. Defaults to `false`.
    #[must_use]
    pub fn with_privacy(mut self, on: bool) -> Self {
        self.privacy_on = on;
        self
    }

    /// Inject a per-op sleep. Every transport method call waits
    /// `delay` before returning.
    #[must_use]
    pub fn inject_delay(mut self, delay: Duration) -> Self {
        self.failure.delay_per_op = Some(delay);
        self
    }

    /// Inject a simulated disconnect after `n_ops` successful
    /// transport operations. The `(n+1)`-th op returns
    /// `Error::Transport("injected disconnect")`.
    #[must_use]
    pub fn inject_disconnect_after(mut self, n_ops: u32) -> Self {
        self.failure.disconnect_after_op_count = Some(n_ops);
        self
    }

    /// Finalise the builder and return a ready-to-use `SimDevice`.
    #[must_use]
    pub fn build(self) -> SimDevice {
        let recordings: HashMap<RecordingId, PreloadedRecording> = self.recordings.into_iter().map(|r| (r.meta.id().clone(), r)).collect();
        let state = SimState {
            device_info: self.device_info,
            battery: self.battery,
            storage: self.storage,
            recordings,
            settings: self.settings,
            expected_token: self.expected_token,
            auth: AuthState::Unauthenticated,
            privacy_on: self.privacy_on,
            recording_state: RecordingState::Idle,
            failure: self.failure,
            op_count: 0,
        };
        SimDevice {
            state: Arc::new(Mutex::new(state)),
        }
    }
}

/// Lock a [`Mutex<SimState>`] and recover from poisoning. Poisoning
/// can only happen if a previous lock-holder panicked; for an
/// in-memory simulator that is almost certainly a test asserting a
/// contradiction, and the safest recovery is to hand the lock back
/// so the test can continue to its next assertion.
pub(crate) fn lock_state(state: &Mutex<SimState>) -> MutexGuard<'_, SimState> {
    state.lock().unwrap_or_else(|poison| poison.into_inner())
}

fn default_battery() -> BatteryLevel {
    // `DEFAULT_BATTERY_PERCENT` is a compile-time constant in the
    // valid `0..=100` range, so `BatteryLevel::new` cannot fail here.
    // If a future edit breaks this invariant, the match will fall
    // back to the documented `EMPTY` constant rather than panic.
    match BatteryLevel::new(DEFAULT_BATTERY_PERCENT) {
        Ok(level) => level,
        Err(_) => BatteryLevel::EMPTY,
    }
}
