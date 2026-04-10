//! Command dispatch modules. One submodule per top-level subcommand,
//! plus shared helpers (backend selection, output format).

pub(crate) mod auth;
pub(crate) mod backend;
pub(crate) mod battery;
#[cfg(feature = "llm")]
pub(crate) mod correct;
pub(crate) mod decode;
pub(crate) mod device;
pub(crate) mod files;
#[cfg(feature = "llm")]
pub(crate) mod llm;
pub(crate) mod output;
pub(crate) mod record;
pub(crate) mod settings;
#[cfg(feature = "llm")]
pub(crate) mod summaries;
#[cfg(feature = "llm")]
pub(crate) mod summarize;
pub(crate) mod sync;
#[cfg(feature = "llm")]
pub(crate) mod template;
#[cfg(feature = "transcribe")]
pub(crate) mod transcribe;
