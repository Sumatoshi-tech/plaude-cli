//! Transport, discovery, and auth-store traits for plaude-cli.
//!
//! This crate defines the boundary abstractions every `plaude`
//! transport implements:
//!
//! * [`Transport`] — the vendor command surface of a connected device.
//! * [`DeviceDiscovery`] — scanning and connecting.
//! * [`AuthStore`] — pluggable credential storage.
//! * [`Error`] / [`Result`] — the unified error taxonomy.
//!
//! Concrete implementations live in `plaud-transport-ble`,
//! `plaud-transport-usb`, `plaud-transport-wifi`, `plaud-auth`, and
//! `plaud-sim`. See
//! [`specs/plaude-cli-v1/journeys/M01-domain-traits.md`](../../../specs/plaude-cli-v1/journeys/M01-domain-traits.md)
//! for context.
//!
//! All three boundary traits are `async` and `dyn`-compatible via
//! [`async_trait::async_trait`]. Native `async fn` in traits gives
//! cleaner syntax but as of Rust 1.85 still has rough edges around
//! `dyn Trait` that `async-trait` smooths over.

pub mod auth_store;
pub mod discovery;
pub mod error;
pub mod transport;

pub use auth_store::AuthStore;
pub use discovery::DeviceDiscovery;
pub use error::{Error, Result};
pub use transport::Transport;
