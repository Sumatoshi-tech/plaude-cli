//! [`SimTransport`] — an in-process implementation of
//! [`plaud_transport::Transport`].
//!
//! The transport shares an `Arc<Mutex<SimState>>` with every other
//! handle created from the same [`crate::SimDevice`]. All state
//! mutation is synchronous; tokio is used only for the optional
//! per-op delay injection.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use plaud_domain::{BatteryLevel, CommonSettingKey, DeviceInfo, Recording, RecordingId, SettingValue, StorageStats};
use plaud_transport::{Error, Result, Transport};

use crate::{
    constants::AUTH_STATUS_REJECTED,
    device::lock_state,
    state::{AuthState, RecordingState, SimState},
};

/// Error message used when the sim's injected-disconnect trigger fires.
const INJECTED_DISCONNECT_MSG: &str = "injected disconnect";

/// Error message returned when a record-control call asks for an
/// invalid state transition.
const INVALID_RECORD_TRANSITION_MSG: &str = "invalid recording-state transition";

/// A trait-object-safe Transport backed by a [`crate::SimDevice`].
#[derive(Debug)]
pub struct SimTransport {
    state: Arc<Mutex<SimState>>,
}

impl SimTransport {
    pub(crate) fn new(state: Arc<Mutex<SimState>>) -> Self {
        Self { state }
    }

    /// Apply per-op delay injection and increment the op counter.
    /// Returns `Err(Transport(..))` if the disconnect injector has
    /// already fired.
    async fn tick(&self) -> Result<()> {
        let delay = {
            let mut state = lock_state(&self.state);
            if let Some(limit) = state.failure.disconnect_after_op_count {
                if state.op_count >= limit {
                    return Err(Error::Transport(INJECTED_DISCONNECT_MSG.to_owned()));
                }
            }
            state.op_count = state.op_count.saturating_add(1);
            state.failure.delay_per_op
        };
        if let Some(d) = delay {
            tokio::time::sleep(d).await;
        }
        Ok(())
    }

    fn ensure_vendor_access(state: &SimState) -> Result<()> {
        match state.auth {
            AuthState::Accepted => Ok(()),
            AuthState::Unauthenticated => Err(Error::AuthRequired),
            AuthState::SoftRejected => Err(Error::AuthRejected {
                status: AUTH_STATUS_REJECTED,
            }),
        }
    }
}

#[async_trait]
impl Transport for SimTransport {
    async fn device_info(&self) -> Result<DeviceInfo> {
        self.tick().await?;
        let state = lock_state(&self.state);
        Self::ensure_vendor_access(&state)?;
        Ok(state.device_info.clone())
    }

    async fn battery(&self) -> Result<BatteryLevel> {
        // Battery read is the SIG-service analogue and intentionally
        // does NOT check auth state — matches live evidence from
        // Test 2b of the token-validation capture.
        self.tick().await?;
        let state = lock_state(&self.state);
        Ok(state.battery)
    }

    async fn storage(&self) -> Result<StorageStats> {
        self.tick().await?;
        let state = lock_state(&self.state);
        Self::ensure_vendor_access(&state)?;
        Ok(state.storage)
    }

    async fn list_recordings(&self) -> Result<Vec<Recording>> {
        self.tick().await?;
        let state = lock_state(&self.state);
        Self::ensure_vendor_access(&state)?;
        let mut list: Vec<Recording> = state.recordings.values().map(|r| r.meta.clone()).collect();
        // Deterministic ordering for the "two sims produce identical
        // traces" property in `tests/determinism.rs`.
        list.sort_by(|a, b| a.id().as_str().cmp(b.id().as_str()));
        Ok(list)
    }

    async fn read_recording(&self, id: &RecordingId) -> Result<Vec<u8>> {
        self.tick().await?;
        let state = lock_state(&self.state);
        Self::ensure_vendor_access(&state)?;
        state
            .recordings
            .get(id)
            .map(|r| r.wav.clone())
            .ok_or_else(|| Error::NotFound(id.as_str().to_owned()))
    }

    async fn read_recording_asr(&self, id: &RecordingId) -> Result<Vec<u8>> {
        self.tick().await?;
        let state = lock_state(&self.state);
        Self::ensure_vendor_access(&state)?;
        state
            .recordings
            .get(id)
            .map(|r| r.asr.clone())
            .ok_or_else(|| Error::NotFound(id.as_str().to_owned()))
    }

    async fn delete_recording(&self, id: &RecordingId) -> Result<()> {
        self.tick().await?;
        let mut state = lock_state(&self.state);
        Self::ensure_vendor_access(&state)?;
        match state.recordings.remove(id) {
            Some(_) => Ok(()),
            None => Err(Error::NotFound(id.as_str().to_owned())),
        }
    }

    async fn read_setting(&self, key: CommonSettingKey) -> Result<SettingValue> {
        self.tick().await?;
        let state = lock_state(&self.state);
        Self::ensure_vendor_access(&state)?;
        state
            .settings
            .get(&key)
            .copied()
            .ok_or_else(|| Error::NotFound(key.name().to_owned()))
    }

    async fn write_setting(&self, key: CommonSettingKey, value: SettingValue) -> Result<()> {
        self.tick().await?;
        let mut state = lock_state(&self.state);
        Self::ensure_vendor_access(&state)?;
        state.settings.insert(key, value);
        Ok(())
    }

    async fn start_recording(&self) -> Result<()> {
        self.tick().await?;
        let mut state = lock_state(&self.state);
        Self::ensure_vendor_access(&state)?;
        match state.recording_state {
            RecordingState::Idle | RecordingState::Paused => {
                state.recording_state = RecordingState::Recording;
                Ok(())
            }
            RecordingState::Recording => Err(Error::Protocol(INVALID_RECORD_TRANSITION_MSG.to_owned())),
        }
    }

    async fn stop_recording(&self) -> Result<()> {
        self.tick().await?;
        let mut state = lock_state(&self.state);
        Self::ensure_vendor_access(&state)?;
        match state.recording_state {
            RecordingState::Recording | RecordingState::Paused => {
                state.recording_state = RecordingState::Idle;
                Ok(())
            }
            RecordingState::Idle => Err(Error::Protocol(INVALID_RECORD_TRANSITION_MSG.to_owned())),
        }
    }

    async fn pause_recording(&self) -> Result<()> {
        self.tick().await?;
        let mut state = lock_state(&self.state);
        Self::ensure_vendor_access(&state)?;
        match state.recording_state {
            RecordingState::Recording => {
                state.recording_state = RecordingState::Paused;
                Ok(())
            }
            RecordingState::Idle | RecordingState::Paused => Err(Error::Protocol(INVALID_RECORD_TRANSITION_MSG.to_owned())),
        }
    }

    async fn resume_recording(&self) -> Result<()> {
        self.tick().await?;
        let mut state = lock_state(&self.state);
        Self::ensure_vendor_access(&state)?;
        match state.recording_state {
            RecordingState::Paused => {
                state.recording_state = RecordingState::Recording;
                Ok(())
            }
            RecordingState::Idle | RecordingState::Recording => Err(Error::Protocol(INVALID_RECORD_TRANSITION_MSG.to_owned())),
        }
    }

    async fn set_privacy(&self, on: bool) -> Result<()> {
        self.tick().await?;
        let mut state = lock_state(&self.state);
        Self::ensure_vendor_access(&state)?;
        state.privacy_on = on;
        Ok(())
    }
}
