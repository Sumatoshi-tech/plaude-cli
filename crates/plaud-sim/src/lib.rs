//! In-process device simulator for plaude-cli.
//!
//! `plaud-sim` models a Plaud Note closely enough that every
//! transport-agnostic test in the workspace can run against it without
//! touching real hardware. It implements the auth-with-status-byte
//! flow (including silent soft-reject), battery reads over the
//! standard SIG service analogue, recording enumeration + read +
//! delete, device settings, recording control state transitions, and
//! a failure-injection layer for partial-failure testing.
//!
//! This crate is the **CI north star**: no milestone after M3 may
//! depend on physical hardware for its mandatory tests.
//!
//! # Quick start
//!
//! ```no_run
//! use plaud_sim::SimDevice;
//!
//! # async fn _example() -> Result<(), Box<dyn std::error::Error>> {
//! let sim = SimDevice::builder().build();
//! let transport = sim.authenticated_transport();
//! let info = transport.device_info().await?;
//! println!("connected to {}", info.model.name());
//! # Ok(()) }
//! ```
//!
//! See the integration tests under `tests/` for the full surface.

pub mod bulk;
pub mod constants;
pub mod device;
pub mod discovery;
pub mod state;
pub mod transport;

pub use device::{SimDevice, SimDeviceBuilder};
pub use discovery::SimDiscovery;
pub use transport::SimTransport;
