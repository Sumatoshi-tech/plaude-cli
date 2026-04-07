//! [`SimDiscovery`] — the in-process analogue of a BLE central's
//! scan + connect + authenticate pipeline.
//!
//! Tests that want to exercise the full auth flow build a
//! `SimDiscovery` via [`crate::SimDevice::discovery`], scan it, and
//! call `connect` with the offered token. A mismatch against the
//! sim's configured `expected_token` yields
//! `Error::AuthRejected { status: 0x01 }`, matching what the real M5
//! BLE transport will return after its timeout-to-AuthRejected
//! translation.

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use async_trait::async_trait;
use plaud_domain::{AuthToken, DeviceCandidate, TransportHint};
use plaud_transport::{DeviceDiscovery, Error, Result, Transport};

use crate::{
    constants::{AUTH_STATUS_REJECTED, PLAUD_LOCAL_NAME, PLAUD_MANUFACTURER_ID_NORDIC},
    device::lock_state,
    state::{AuthState, SimState},
    transport::SimTransport,
};

/// Default RSSI reported for the sim's sole candidate. Picked to
/// match the values we saw in the real R0 capture so filters that
/// compare against "realistic" ranges are not surprised by the sim.
const DEFAULT_RSSI_DBM: i16 = -67;

/// Error message when a caller tries to `connect` to a candidate
/// the sim did not produce from its own `scan`.
const UNKNOWN_CANDIDATE_MSG: &str = "candidate did not originate from this SimDiscovery";

/// A sim-side implementation of [`DeviceDiscovery`]. Unlike a real
/// BLE central, the scan always returns a single deterministic
/// candidate and the connect path short-circuits the underlying
/// GATT state machine.
#[derive(Debug)]
pub struct SimDiscovery {
    state: Arc<Mutex<SimState>>,
    offered_token: AuthToken,
}

impl SimDiscovery {
    pub(crate) fn new(state: Arc<Mutex<SimState>>, offered_token: AuthToken) -> Self {
        Self { state, offered_token }
    }

    fn canonical_candidate(&self) -> DeviceCandidate {
        DeviceCandidate {
            local_name: PLAUD_LOCAL_NAME.to_owned(),
            manufacturer_id: PLAUD_MANUFACTURER_ID_NORDIC,
            rssi_dbm: Some(DEFAULT_RSSI_DBM),
            transport_hint: TransportHint::Ble,
        }
    }
}

#[async_trait]
impl DeviceDiscovery for SimDiscovery {
    async fn scan(&self, _timeout: Duration) -> Result<Vec<DeviceCandidate>> {
        Ok(vec![self.canonical_candidate()])
    }

    async fn connect(&self, candidate: &DeviceCandidate) -> Result<Box<dyn Transport>> {
        let expected_candidate = self.canonical_candidate();
        if *candidate != expected_candidate {
            return Err(Error::Transport(UNKNOWN_CANDIDATE_MSG.to_owned()));
        }
        let mut state = lock_state(&self.state);
        let token_ok = match &state.expected_token {
            Some(expected) => expected.as_str() == self.offered_token.as_str(),
            None => true, // unset expected token = accept anything
        };
        if token_ok {
            state.auth = AuthState::Accepted;
            Ok(Box::new(SimTransport::new(Arc::clone(&self.state))))
        } else {
            state.auth = AuthState::SoftRejected;
            Err(Error::AuthRejected {
                status: AUTH_STATUS_REJECTED,
            })
        }
    }
}
