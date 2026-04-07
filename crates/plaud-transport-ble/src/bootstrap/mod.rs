//! Fake peripheral used by the M8 `plaude-cli auth bootstrap` flow.
//!
//! The bootstrap peripheral is the inverse of `BleSession`: the CLI
//! acts as a BLE peripheral advertising `PLAUD_NOTE`, and waits for
//! the user's Plaud phone app to connect and write its auth frame
//! (the `01 01 00 02 00 00 <token>` layout captured in the R1 APK
//! analysis and live-tested in Test 2b). Once the write is received
//! and decoded, the peripheral responds with a mock auth-accepted
//! notification so the phone does not error out, tears down, and
//! yields the captured [`AuthToken`] to the caller.
//!
//! M8 ships the **hermetic protocol layer** — the trait, the
//! loopback implementation, and the CLI wiring. The real BlueZ GATT
//! server that turns this into a genuine advertisement lands
//! alongside the btleplug central in a later milestone.

pub mod loopback;
pub mod session;

#[cfg(feature = "btleplug-backend")]
pub mod bluer_peripheral;

#[cfg(feature = "btleplug-backend")]
pub use bluer_peripheral::run_bluer_bootstrap;
pub use loopback::{LoopbackBootstrap, TestPhone};
pub use session::{BootstrapChannel, BootstrapError, BootstrapOutcome, BootstrapSession};
