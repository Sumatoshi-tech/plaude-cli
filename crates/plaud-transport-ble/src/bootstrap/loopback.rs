//! Hermetic in-memory bootstrap peripheral + its paired fake phone.
//!
//! Used by both integration tests and the CLI's `--backend sim`
//! runtime to exercise the full capture-and-respond handshake
//! without a BlueZ adapter.

use bytes::Bytes;
use thiserror::Error;

use super::session::{BootstrapChannel, BootstrapSession, PhoneChannel};

/// Factory + convenience wrapper around [`BootstrapChannel::pair`] +
/// [`BootstrapSession::new`]. Splits the peripheral's protocol
/// state machine from the fake phone so tests can drive both ends
/// independently.
#[derive(Debug)]
pub struct LoopbackBootstrap {
    session: BootstrapSession,
    phone: TestPhone,
}

impl LoopbackBootstrap {
    /// Construct a new loopback pair. The returned value owns a
    /// `BootstrapSession` ready for [`BootstrapSession::run`] and a
    /// [`TestPhone`] the caller uses to write the auth frame.
    #[must_use]
    pub fn new() -> Self {
        let (channel, phone) = BootstrapChannel::pair();
        Self {
            session: BootstrapSession::new(channel),
            phone: TestPhone::new(phone),
        }
    }

    /// Split the loopback into its two halves. Callers typically
    /// spawn one task per half.
    #[must_use]
    pub fn split(self) -> (BootstrapSession, TestPhone) {
        (self.session, self.phone)
    }
}

impl Default for LoopbackBootstrap {
    fn default() -> Self {
        Self::new()
    }
}

/// Fake phone counterpart for [`LoopbackBootstrap`]. Pushes auth
/// writes into the peripheral and observes the notifications the
/// peripheral sends back.
#[derive(Debug)]
pub struct TestPhone {
    inner: PhoneChannel,
}

impl TestPhone {
    pub(crate) fn new(inner: PhoneChannel) -> Self {
        Self { inner }
    }

    /// Write one frame (typically a pre-built auth frame from
    /// [`plaud_proto::encode::auth::authenticate`]) to the peripheral.
    ///
    /// # Errors
    ///
    /// Returns [`TestPhoneError::Closed`] if the peripheral dropped
    /// its receiving half before the write landed.
    pub async fn write(&self, bytes: Bytes) -> Result<(), TestPhoneError> {
        self.inner.writes_out.send(bytes).await.map_err(|_| TestPhoneError::Closed)
    }

    /// Await and return the next notification the peripheral sends
    /// back. Used by tests to assert the peripheral echoes the
    /// mock auth-accepted frame after capturing a write.
    pub async fn receive_notification(&mut self) -> Option<Bytes> {
        self.inner.notify_in.recv().await
    }
}

/// Errors produced by [`TestPhone::write`].
#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum TestPhoneError {
    /// The peripheral side of the loopback has been dropped.
    #[error("peripheral channel closed")]
    Closed,
}
