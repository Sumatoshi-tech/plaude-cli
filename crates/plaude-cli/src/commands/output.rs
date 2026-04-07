//! Shared `--output` enum for every command that supports text and
//! JSON emission. Kept in one module so renames propagate and the
//! default is single-sourced.

use clap::ValueEnum;

/// Output format selector used by `battery`, `device info`, and every
/// future command that prints structured data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum OutputFormat {
    /// Human-readable text (default).
    Text,
    /// Single-line JSON object.
    Json,
}
