//! BLE central transport for Plaud devices.
//!
//! M5 ships the protocol layer — the [`session::BleSession`] state
//! machine, the [`bulk::BulkReassembler`], the [`transport::BleTransport`]
//! `Transport` stub, and a [`discovery::BleDiscovery`] that delegates
//! to an injectable [`discovery::ScanProvider`]. Every piece is tested
//! hermetically against in-memory `tokio::mpsc` channels via
//! [`channel::BleChannel::loopback_pair`].
//!
//! The real btleplug-backed backend that bridges a physical BLE
//! adapter into this crate's channel contract lives behind the
//! `btleplug-backend` cargo feature and is off by default in M5.
//! Real-hardware smoke tests are behind the `hw-tests` feature and
//! are never run in CI.
//!
//! See [`specs/plaude-cli-v1/journeys/M05-transport-ble.md`](../../../specs/plaude-cli-v1/journeys/M05-transport-ble.md)
//! for the milestone context and the full DoD.

pub mod battery;
pub mod bootstrap;
pub mod bulk;
pub mod channel;
pub mod constants;
pub mod discovery;
pub mod session;
pub mod transport;

#[cfg(feature = "btleplug-backend")]
pub mod backend;

pub use battery::{BatteryReader, FixedBatteryReader};
pub use bootstrap::{BootstrapChannel, BootstrapError, BootstrapOutcome, BootstrapSession, LoopbackBootstrap, TestPhone};
pub use bulk::{BulkReassembler, FeedStatus};
pub use channel::{BleChannel, BleRx, BleTx, TestPeer, TestPeerError};
pub use discovery::{BleDiscovery, ScanProvider};
pub use session::BleSession;
pub use transport::BleTransport;
