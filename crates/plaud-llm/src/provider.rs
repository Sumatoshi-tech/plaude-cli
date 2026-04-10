//! LLM provider wrapper around the `genai` crate.
//!
//! [`LlmProvider`] owns a `genai::Client` configured from
//! [`LlmConfig`](crate::config::LlmConfig) and exposes a
//! `check()` method that verifies connectivity.

use genai::Client;

use crate::config::LlmConfig;

/// Status returned by [`LlmProvider::check`].
#[derive(Debug)]
pub struct ProviderStatus {
    /// Resolved model identifier.
    pub model: String,
    /// Whether the provider endpoint responded successfully.
    pub reachable: bool,
    /// Human-readable error if the endpoint was not reachable.
    pub error: Option<String>,
}

/// Errors from provider operations.
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    /// Failed to construct the genai client.
    #[error("failed to create LLM client: {0}")]
    ClientInit(String),
    /// An LLM request failed.
    #[error("LLM request failed: {0}")]
    Request(String),
}

/// Wraps `genai::Client` with plaude-specific configuration.
pub struct LlmProvider {
    client: Client,
    config: LlmConfig,
}

impl LlmProvider {
    /// Create a new provider from the given configuration.
    pub fn new(config: LlmConfig) -> Self {
        let client = Client::builder().build();
        Self { client, config }
    }

    /// Return the configured model name.
    pub fn model(&self) -> &str {
        &self.config.model
    }

    /// Return a reference to the underlying genai client.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Check connectivity to the configured provider.
    ///
    /// Sends a minimal chat request to verify the endpoint responds.
    /// Returns status even on failure — callers inspect
    /// [`ProviderStatus::reachable`] rather than handling errors.
    pub async fn check(&self) -> ProviderStatus {
        use genai::chat::{ChatMessage, ChatRequest};

        let chat_req = ChatRequest::new(vec![ChatMessage::system("Reply with exactly: ok"), ChatMessage::user("ping")]);

        match self.client.exec_chat(&self.config.model, chat_req, None).await {
            Ok(_) => ProviderStatus {
                model: self.config.model.clone(),
                reachable: true,
                error: None,
            },
            Err(e) => ProviderStatus {
                model: self.config.model.clone(),
                reachable: false,
                error: Some(e.to_string()),
            },
        }
    }
}
