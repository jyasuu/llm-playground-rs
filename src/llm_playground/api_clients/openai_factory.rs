// OpenAI provider factory implementation
use super::provider_factory::ProviderFactory;
use super::traits::LLMClient;
use super::openai_client::OpenAIClient;
use crate::llm_playground::provider_config::ProviderConfig;

/// Factory for creating OpenAI-compatible clients
/// Implements the Factory pattern for OCP compliance
pub struct OpenAIProviderFactory;

impl OpenAIProviderFactory {
    pub fn new() -> Self {
        Self
    }
}

impl ProviderFactory for OpenAIProviderFactory {
    fn supports_provider(&self, provider_type: &str) -> bool {
        matches!(provider_type, "openai" | "anthropic" | "ollama" | "custom")
    }

    fn create_client(&self, _config: &ProviderConfig) -> Result<Box<dyn LLMClient>, String> {
        Ok(Box::new(OpenAIClient::new()))
    }

    fn provider_type(&self) -> &str {
        "openai"
    }
}

impl Default for OpenAIProviderFactory {
    fn default() -> Self {
        Self::new()
    }
}