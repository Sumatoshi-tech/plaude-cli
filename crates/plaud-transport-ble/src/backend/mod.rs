//! Real BLE hardware backend via `btleplug`.
//!
//! This module is compiled only when the `btleplug-backend` cargo
//! feature is enabled. It provides:
//!
//! - [`BtleplugScanProvider`] — discovers Plaud devices via BLE advertising
//! - [`BtleplugBatteryReader`] — reads the standard SIG Battery Service
//! - [`connect_peripheral`] — connects to a device and returns a
//!   [`BleChannel`] + [`BtleplugBatteryReader`] ready for use by
//!   [`crate::session::BleSession`]

mod adapter;

pub use adapter::{BtleplugBatteryReader, BtleplugScanProvider, connect_peripheral};
