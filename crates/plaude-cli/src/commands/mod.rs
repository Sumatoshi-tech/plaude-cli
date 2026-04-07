//! Command dispatch modules. One submodule per top-level subcommand,
//! plus shared helpers (backend selection, output format).

pub(crate) mod auth;
pub(crate) mod backend;
pub(crate) mod battery;
pub(crate) mod decode;
pub(crate) mod device;
pub(crate) mod files;
pub(crate) mod output;
pub(crate) mod record;
pub(crate) mod settings;
pub(crate) mod sync;
pub(crate) mod transcribe;
