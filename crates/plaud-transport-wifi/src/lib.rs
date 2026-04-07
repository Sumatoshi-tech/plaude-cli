//! Wi-Fi Fast Transfer hotspot client for Plaud devices.
//!
//! When triggered via a BLE opcode in the `0x78`–`0x7D` range, a Plaud
//! device opens a short-lived open Wi-Fi access point and serves file
//! bulk transfers over it. This crate is the client side of that
//! exchange.
//!
//! The exact wire format of the hotspot-side protocol is not yet
//! finalised and is tracked as stretch milestone **M13** in
//! `specs/plaude-cli-v1/ROADMAP.md`. M0 ships the crate as a
//! documented stub so the workspace layout is stable from day one.
