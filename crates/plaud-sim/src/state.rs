//! Internal state of a [`crate::SimDevice`]. Not part of the public
//! API — every accessor goes through `SimTransport` or
//! `SimDeviceBuilder`.

use std::{collections::HashMap, time::Duration};

use plaud_domain::{AuthToken, BatteryLevel, CommonSettingKey, DeviceInfo, Recording, RecordingId, SettingValue, StorageStats};

/// Whether the current (simulated) BLE session has successfully
/// authenticated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthState {
    /// No auth attempt has been made yet. Battery reads still work
    /// (standard SIG service analogue); every vendor opcode returns
    /// `Error::AuthRequired`.
    Unauthenticated,
    /// Auth accepted; every method on the transport works.
    Accepted,
    /// Auth attempted but token mismatched. The device keeps the
    /// connection alive and silently drops vendor opcodes. The sim
    /// returns `Error::AuthRejected { status: 0x01 }` for every
    /// vendor method to match what the M5 BLE transport will surface
    /// after its own timeout-to-AuthRejected translation.
    SoftRejected,
}

/// Physical recording-pipeline state of the simulated device.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecordingState {
    /// No recording in progress.
    Idle,
    /// A recording is active (microphones capturing).
    Recording,
    /// Recording has been paused via the `pause` opcode.
    Paused,
}

/// A preloaded recording: the domain `Recording` metadata plus the
/// raw `.WAV` bytes the sim will hand back from
/// `Transport::read_recording`, plus the `.ASR` sidecar bytes which
/// tests can inspect via `SimDevice::asr_bytes_for`.
#[derive(Debug, Clone)]
pub(crate) struct PreloadedRecording {
    pub(crate) meta: Recording,
    pub(crate) wav: Vec<u8>,
    pub(crate) asr: Vec<u8>,
}

/// Failure-injection knobs.
#[derive(Debug, Clone, Default)]
pub(crate) struct FailureInjection {
    /// If set, every transport method call sleeps for this duration
    /// before returning.
    pub(crate) delay_per_op: Option<Duration>,
    /// If set, the `(n+1)`-th transport call after this many ops
    /// returns `Error::Transport("injected disconnect")`.
    pub(crate) disconnect_after_op_count: Option<u32>,
}

/// The complete internal state of a simulator.
#[derive(Debug)]
pub(crate) struct SimState {
    pub(crate) device_info: DeviceInfo,
    pub(crate) battery: BatteryLevel,
    pub(crate) storage: StorageStats,
    pub(crate) recordings: HashMap<RecordingId, PreloadedRecording>,
    pub(crate) settings: HashMap<CommonSettingKey, SettingValue>,
    pub(crate) expected_token: Option<AuthToken>,
    pub(crate) auth: AuthState,
    pub(crate) privacy_on: bool,
    pub(crate) recording_state: RecordingState,
    pub(crate) failure: FailureInjection,
    pub(crate) op_count: u32,
}
