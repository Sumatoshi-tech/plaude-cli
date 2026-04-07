//! `BleChannel` — a bidirectional mpsc-based view of one BLE vendor
//! characteristic session.
//!
//! The channel abstracts over "whatever connected BLE peripheral is
//! on the other end". `BleSession` holds a `BleChannel` and never
//! knows whether its counterparty is a real btleplug peripheral, a
//! `plaud-sim`-backed test harness, or an in-memory loopback.

use bytes::Bytes;
use tokio::sync::mpsc;

use crate::constants::DEFAULT_CHANNEL_CAPACITY;

/// Owned outbound half of a `BleChannel` (what the session writes to).
pub type BleTx = mpsc::Sender<Bytes>;
/// Owned inbound half of a `BleChannel` (notifications flowing from
/// the device).
pub type BleRx = mpsc::Receiver<Bytes>;

/// Bidirectional channel pair connecting a `BleSession` to its
/// underlying transport (real BLE adapter, sim bridge, or test peer).
#[derive(Debug)]
pub struct BleChannel {
    /// Outbound half — frames the session emits toward the device.
    pub tx: BleTx,
    /// Inbound half — notifications the device emits toward the session.
    pub rx: BleRx,
}

impl BleChannel {
    /// Build a loopback `BleChannel` + `TestPeer` pair that
    /// communicate via in-memory `tokio::mpsc` channels. Used by
    /// every integration test in this crate.
    #[must_use]
    pub fn loopback_pair() -> (Self, TestPeer) {
        let (session_tx, peer_rx) = mpsc::channel(DEFAULT_CHANNEL_CAPACITY);
        let (peer_tx, session_rx) = mpsc::channel(DEFAULT_CHANNEL_CAPACITY);
        let channel = Self {
            tx: session_tx,
            rx: session_rx,
        };
        let peer = TestPeer { tx: peer_tx, rx: peer_rx };
        (channel, peer)
    }
}

/// The "other side" of a loopback `BleChannel`. A test uses it to
/// act as a fake Plaud device: `receive()` waits for the session to
/// write something, `send()` pushes a notification back.
#[derive(Debug)]
pub struct TestPeer {
    tx: mpsc::Sender<Bytes>,
    rx: mpsc::Receiver<Bytes>,
}

impl TestPeer {
    /// Wait for the next frame the session wrote. Returns `None` if
    /// the channel is closed.
    pub async fn receive(&mut self) -> Option<Bytes> {
        self.rx.recv().await
    }

    /// Push a notification back to the session.
    ///
    /// # Errors
    ///
    /// Returns `TestPeerError::Closed` if the session dropped its
    /// receiving half.
    pub async fn send(&self, bytes: Bytes) -> Result<(), TestPeerError> {
        self.tx.send(bytes).await.map_err(|_| TestPeerError::Closed)
    }
}

/// Errors produced by [`TestPeer::send`].
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum TestPeerError {
    /// The session half of the loopback has been dropped.
    #[error("session channel closed")]
    Closed,
}
