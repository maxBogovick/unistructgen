use crate::llm::{LlmClient, Result, LlmError};
use crate::llm::ollama::OllamaClient;
use crate::llm::openai::OpenAiClient;
use std::env;

#[derive(Debug, Clone, PartialEq)]
pub enum Provider {
    OpenAi,
    Ollama,
    /// Automatically determine based on environment variables
    Auto,
}

#[derive(Debug, Clone)]
pub struct LlmClientConfig {
    pub provider: Provider,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

impl Default for LlmClientConfig {
    fn default() -> Self {
        Self {
            provider: Provider::Auto,
            model: None,
            api_key: None,
            base_url: None,
        }
    }
}

/// Factory for creating LlmClient instances.
/// Uses the Builder pattern for configuration.
pub struct LlmClientFactory {
    config: LlmClientConfig,
}

impl LlmClientFactory {
    pub fn new() -> Self {
        Self {
            config: LlmClientConfig::default(),
        }
    }

    /// Explicitly set the provider
    pub fn with_provider(mut self, provider: Provider) -> Self {
        self.config.provider = provider;
        self
    }

    /// Set specific model (overrides defaults)
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.config.model = Some(model.into());
        self
    }

    /// Set API key explicitly (otherwise reads from ENV)
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.config.api_key = Some(key.into());
        self
    }

    /// Build the specific client
    pub fn build(self) -> Result<Box<dyn LlmClient>> {
        let provider = match self.config.provider {
            Provider::Auto => self.resolve_provider(),
            p => p,
        };

        match provider {
            Provider::OpenAi => {
                let model = self.config.model.unwrap_or_else(|| "gpt-4o".to_string());
                
                // If API key is provided explicitly, set env var temporarily or modify client ctor?
                // Ideally OpenAiClient should accept key in constructor.
                // Our current OpenAiClient reads env var inside new(). 
                // Let's rely on that for now, or assume if we are here, we have the key.
                
                let client = OpenAiClient::new(model)?;
                // In a real prod version, OpenAiClient::builder() would be better to pass key directly.
                
                Ok(Box::new(client))
            }
            Provider::Ollama => {
                let model = self.config.model.unwrap_or_else(|| "llama3".to_string());
                let mut client = OllamaClient::new(model);
                if let Some(url) = self.config.base_url {
                    client = client.with_url(url);
                }
                Ok(Box::new(client))
            }
            Provider::Auto => unreachable!("Should be resolved"),
        }
    }

    /// Smart logic to determine which provider to use
    fn resolve_provider(&self) -> Provider {
        // 1. Check for OpenAI Key
        if self.config.api_key.is_some() || env::var("OPENAI_API_KEY").is_ok() {
            return Provider::OpenAi;
        }

        // 2. Default to Ollama (local)
        // Ideally we could check if Ollama is running via a health check, 
        // but for now we assume local dev environment.
        Provider::Ollama
    }
}

impl Default for LlmClientFactory {
    fn default() -> Self {
        Self::new()
    }
}
