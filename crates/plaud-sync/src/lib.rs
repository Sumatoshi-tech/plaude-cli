//! High-level sync orchestrator for plaude-cli.
//!
//! `plaud-sync` drives the `plaude sync <dir>` command: idempotent
//! mirror of device recordings to a local directory with a state
//! file, resumable partial downloads, and partial-failure recovery.
//! It composes a `plaud_transport::Transport` with filesystem I/O
//! and a JSON-backed state tracker.
//!
//! Implementation lands in milestone **M9**. M0 ships the crate as a
//! documented stub.
