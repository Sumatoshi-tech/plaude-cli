//! Core protocol state machine for a bootstrap peripheral.
//!
//! Wraps a [`BootstrapChannel`] (a tokio mpsc pair modelling the
//! peripheral's side of a single GATT connection) and drives the
//! three-step handshake:
//!
//! 1. Wait for a write on the vendor write characteristic, bounded
//!    by a caller-supplied timeout.
//! 2. Decode the bytes via [`plaud_proto::parse_auth_write`]. On
//!    success we hold an [`AuthToken`].
//! 3. Push a mock [`AUTH_STATUS_ACCEPTED`] notification back to the
//!    phone so its app sees "authenticated" and stops retrying.
//!
//! Every failure mode is a [`BootstrapError`] variant — no panics
//! on any input.

use std::time::Duration;

use bytes::Bytes;
use plaud_domain::AuthToken;
use plaud_proto::parse_auth_write;
use thiserror::Error;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    time::timeout,
};

use crate::constants::DEFAULT_CHANNEL_CAPACITY;

/// Mock auth-accepted notification bytes — the smallest wire-legal
/// control frame the real device emits for
/// `AuthStatus::Accepted`. Matches
/// `encode::auth::authenticate`'s response layout: `01 01 00 00`
/// (control magic + opcode 0x0001 LE + status byte 0x00).
pub(crate) const MOCK_AUTH_ACCEPTED_FRAME: &[u8] = &[0x01, 0x01, 0x00, 0x00];

/// Default timeout for a bootstrap `run()` call. Matches the M8 DoD
/// "default 120 s" budget.
pub const BOOTSTRAP_DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);

/// Outcome of a successful bootstrap handshake.
#[derive(Debug, Clone)]
pub struct BootstrapOutcome {
    /// The auth token captured from the phone's write.
    pub token: AuthToken,
}

/// Peripheral-side view of a bootstrap "radio" — two mpsc halves
/// representing writes arriving from the phone and notifications
/// sent back to the phone.
#[derive(Debug)]
pub struct BootstrapChannel {
    /// Writes arriving from the phone (phone → peripheral direction).
    pub writes_in: Receiver<Bytes>,
    /// Notifications sent back to the phone (peripheral → phone).
    pub notify_out: Sender<Bytes>,
}

impl BootstrapChannel {
    /// Construct a new channel pair. Returns the peripheral side and
    /// a handle the phone (real or fake) uses to drive the other end.
    #[must_use]
    pub fn pair() -> (Self, PhoneChannel) {
        let (phone_tx, peripheral_rx) = tokio::sync::mpsc::channel(DEFAULT_CHANNEL_CAPACITY);
        let (peripheral_tx, phone_rx) = tokio::sync::mpsc::channel(DEFAULT_CHANNEL_CAPACITY);
        let channel = Self {
            writes_in: peripheral_rx,
            notify_out: peripheral_tx,
        };
        let phone = PhoneChannel {
            writes_out: phone_tx,
            notify_in: phone_rx,
        };
        (channel, phone)
    }
}

/// Phone side of a [`BootstrapChannel::pair`]. A real btleplug
/// peripheral bridge writes GATT writes into `writes_out` and
/// forwards GATT notifications out of `notify_in`. Tests use the
/// [`super::loopback::TestPhone`] wrapper around this type.
#[derive(Debug)]
pub struct PhoneChannel {
    /// Writes the phone is making (phone → peripheral).
    pub writes_out: Sender<Bytes>,
    /// Notifications the phone is receiving (peripheral → phone).
    pub notify_in: Receiver<Bytes>,
}

/// A bootstrap protocol session driven over a [`BootstrapChannel`].
#[derive(Debug)]
pub struct BootstrapSession {
    channel: BootstrapChannel,
}

impl BootstrapSession {
    /// Wrap a [`BootstrapChannel`] as a protocol session.
    #[must_use]
    pub fn new(channel: BootstrapChannel) -> Self {
        Self { channel }
    }

    /// Run the handshake to completion or timeout.
    ///
    /// Preconditions: the channel is connected (tx/rx halves alive).
    /// Postcondition on `Ok`: the mock auth-accepted notification
    /// has been pushed back to the phone and the captured token is
    /// returned. On `Err`, the session has not stored anything and
    /// the caller should log and exit.
    pub async fn run(mut self, budget: Duration) -> Result<BootstrapOutcome, BootstrapError> {
        let write = timeout(budget, self.channel.writes_in.recv())
            .await
            .map_err(|_| BootstrapError::Timeout { seconds: budget.as_secs() })?
            .ok_or(BootstrapError::PhoneDisconnected)?;
        let token = parse_auth_write(write.as_ref()).map_err(|e| BootstrapError::DecodeFailed { reason: e.to_string() })?;
        self.channel
            .notify_out
            .send(Bytes::from_static(MOCK_AUTH_ACCEPTED_FRAME))
            .await
            .map_err(|_| BootstrapError::PhoneDisconnected)?;
        Ok(BootstrapOutcome { token })
    }
}

/// Errors produced by [`BootstrapSession::run`].
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum BootstrapError {
    /// No phone connected (or no write observed) within the budget.
    #[error("bootstrap timeout: no auth write within {seconds}s")]
    Timeout {
        /// Budget that elapsed.
        seconds: u64,
    },
    /// The channel's rx half closed before any write arrived.
    #[error("phone disconnected before sending auth frame")]
    PhoneDisconnected,
    /// The phone's write did not decode as an auth frame.
    #[error("decode failed: {reason}")]
    DecodeFailed {
        /// Wrapped [`plaud_proto::DecodeError`] message.
        reason: String,
    },
}
