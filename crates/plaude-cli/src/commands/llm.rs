//! `plaude llm` command group — LLM provider management.
//!
//! Journey: `specs/journeys/JOURNEY-L1-llm-config-provider.md`

use std::path::Path;

use clap::Subcommand;

use crate::DispatchError;

/// LLM provider management commands.
#[derive(Debug, Subcommand)]
pub(crate) enum LlmCommand {
    /// Verify LLM provider connectivity and print resolved config.
    Check,
}

/// Default plaude config directory when none is specified.
fn default_config_dir() -> std::path::PathBuf {
    dirs::config_dir()
        .map(|d| d.join("plaude"))
        .unwrap_or_else(|| std::path::PathBuf::from(".config/plaude"))
}

/// Run the `plaude llm` subcommand.
pub(crate) async fn run(cmd: LlmCommand, config_dir: Option<&Path>) -> Result<(), DispatchError> {
    match cmd {
        LlmCommand::Check => check(config_dir).await,
    }
}

/// `plaude llm check` — print resolved provider config and test
/// connectivity.
async fn check(config_dir: Option<&Path>) -> Result<(), DispatchError> {
    let dir = config_dir.map(std::path::PathBuf::from).unwrap_or_else(default_config_dir);

    let config = plaud_llm::config::LlmConfig::load(&dir).map_err(|e| DispatchError::Runtime(format!("failed to load LLM config: {e}")))?;

    println!("Model:    {}", config.model);

    if let Some(ref provider) = config.provider {
        if let Some(ref kind) = provider.kind {
            println!("Provider: {kind}");
        }
        if let Some(ref url) = provider.base_url {
            println!("Endpoint: {url}");
        }
        if let Some(ref key_env) = provider.api_key_env {
            let key_set = std::env::var(key_env).is_ok_and(|v| !v.is_empty());
            println!("API key:  ${key_env} ({})", if key_set { "set" } else { "not set" });
        }
    } else {
        println!("Provider: auto-detect (Ollama fallback)");
    }

    let provider = plaud_llm::provider::LlmProvider::new(config);
    let status = provider.check().await;

    if status.reachable {
        println!("Status:   reachable");
    } else {
        let err_msg = status.error.as_deref().unwrap_or("unknown error");
        eprintln!("Status:   unreachable — {err_msg}");
        eprintln!();
        eprintln!("Tip: install Ollama (https://ollama.com) or configure a cloud");
        eprintln!("     provider in ~/.config/plaude/llm.toml");
    }

    Ok(())
}
