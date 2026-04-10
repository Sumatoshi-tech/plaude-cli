//! LLM provider configuration.
//!
//! Reads `llm.toml` from the plaude config directory with sensible
//! defaults (Ollama + `llama3.2:3b`). Env var `PLAUDE_LLM_MODEL`
//! overrides the configured model.

use std::path::Path;

use serde::Deserialize;

/// Default model used when no configuration is present.
pub const DEFAULT_MODEL: &str = "llama3.2:3b";

/// Environment variable that overrides the configured model name.
pub const MODEL_ENV_VAR: &str = "PLAUDE_LLM_MODEL";

/// Config filename within the plaude config directory.
const CONFIG_FILENAME: &str = "llm.toml";

/// LLM provider configuration loaded from `llm.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct LlmConfig {
    /// Model identifier (e.g. `llama3.2:3b`, `gpt-4o-mini`,
    /// `claude-sonnet-4-5-20250514`). The `genai` crate auto-detects
    /// the provider from the model name prefix.
    pub model: String,

    /// Optional provider override for custom endpoints.
    pub provider: Option<ProviderConfig>,
}

/// Provider-specific overrides for custom endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ProviderConfig {
    /// Provider kind hint (e.g. `"openai"`, `"anthropic"`).
    /// When set, bypasses genai's automatic model→provider mapping.
    pub kind: Option<String>,

    /// Custom base URL (e.g. `http://localhost:1234/v1` for LM Studio).
    pub base_url: Option<String>,

    /// Name of the environment variable holding the API key.
    /// The key itself is never stored in the config file.
    pub api_key_env: Option<String>,
}

/// Errors that can occur while loading LLM configuration.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The config file exists but contains invalid TOML.
    #[error("invalid llm.toml: {0}")]
    Parse(#[from] toml::de::Error),

    /// An I/O error occurred reading the config file (other than
    /// "not found", which is handled by falling back to defaults).
    #[error("failed to read {path}: {source}")]
    Io {
        /// Path that was being read.
        path: std::path::PathBuf,
        /// Underlying I/O error.
        source: std::io::Error,
    },
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            model: DEFAULT_MODEL.to_owned(),
            provider: None,
        }
    }
}

impl LlmConfig {
    /// Load configuration from `<config_dir>/llm.toml`.
    ///
    /// If the file does not exist, returns sensible defaults.
    /// If `PLAUDE_LLM_MODEL` is set, it overrides the model field.
    pub fn load(config_dir: &Path) -> Result<Self, ConfigError> {
        let mut config = Self::load_file(config_dir)?;

        // Env var override takes precedence over file.
        if let Ok(model) = std::env::var(MODEL_ENV_VAR) {
            if !model.is_empty() {
                config.model = model;
            }
        }

        Ok(config)
    }

    /// Load configuration from the TOML file only, without applying
    /// environment variable overrides. Returns defaults if the file
    /// does not exist.
    pub fn load_file(config_dir: &Path) -> Result<Self, ConfigError> {
        let path = config_dir.join(CONFIG_FILENAME);
        match std::fs::read_to_string(&path) {
            Ok(contents) => Ok(toml::from_str::<Self>(&contents)?),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(ConfigError::Io { path, source: e }),
        }
    }

    /// Apply a model override, returning a new config. Used for
    /// `--model` CLI flags.
    #[must_use]
    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_returns_ollama_model() {
        let config = LlmConfig::default();
        assert_eq!(config.model, DEFAULT_MODEL);
        assert!(config.provider.is_none());
    }

    #[test]
    fn load_file_missing_returns_defaults() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = LlmConfig::load_file(tmp.path()).expect("load");
        assert_eq!(config, LlmConfig::default());
    }

    #[test]
    fn load_file_minimal_toml() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(tmp.path().join("llm.toml"), "model = \"gpt-4o-mini\"\n").expect("write");
        let config = LlmConfig::load_file(tmp.path()).expect("load");
        assert_eq!(config.model, "gpt-4o-mini");
        assert!(config.provider.is_none());
    }

    #[test]
    fn load_file_full_toml() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            tmp.path().join("llm.toml"),
            r#"
model = "claude-sonnet-4-5-20250514"

[provider]
kind = "anthropic"
base_url = "https://api.custom.example/v1"
api_key_env = "MY_ANTHROPIC_KEY"
"#,
        )
        .expect("write");
        let config = LlmConfig::load_file(tmp.path()).expect("load");
        assert_eq!(config.model, "claude-sonnet-4-5-20250514");
        let provider = config.provider.as_ref().expect("provider present");
        assert_eq!(provider.kind.as_deref(), Some("anthropic"));
        assert_eq!(provider.base_url.as_deref(), Some("https://api.custom.example/v1"));
        assert_eq!(provider.api_key_env.as_deref(), Some("MY_ANTHROPIC_KEY"));
    }

    #[test]
    fn load_file_invalid_toml_returns_parse_error() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(tmp.path().join("llm.toml"), "{{invalid toml}}").expect("write");
        let err = LlmConfig::load_file(tmp.path()).unwrap_err();
        assert!(matches!(err, ConfigError::Parse(_)), "expected Parse error, got: {err:?}");
    }

    #[test]
    fn with_model_overrides() {
        let config = LlmConfig::default().with_model("gpt-4o".to_owned());
        assert_eq!(config.model, "gpt-4o");
    }
}
