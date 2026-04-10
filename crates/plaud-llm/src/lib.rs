//! Multi-provider LLM abstraction for plaude-cli.
//!
//! Wraps the `genai` crate to provide config-driven access to Ollama,
//! OpenAI, Anthropic, and any OpenAI-compatible endpoint. The primary
//! entry points are [`LlmConfig`] for configuration and
//! [`LlmProvider`] for executing LLM operations.

pub mod chunk;
pub mod config;
pub mod correct;
pub mod provider;
pub mod summarize;
pub mod template;
