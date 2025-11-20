// Gemini provider factory implementation
use super::provider_factory::ProviderFactory;
use super::traits::LLMClient;
use super::gemini_client::GeminiClient;
use crate::llm_playground::provider_config::ProviderConfig;

/// Factory for creating Gemini clients
/// Implements the Factory pattern for OCP compliance
pub struct GeminiProviderFactory;

impl GeminiProviderFactory {
    pub fn new() -> Self {
        Self
    }
}

impl ProviderFactory for GeminiProviderFactory {
    fn supports_provider(&self, provider_type: &str) -> bool {
        provider_type == "gemini"
    }

    fn create_client(&self, _config: &ProviderConfig) -> Result<Box<dyn LLMClient>, String> {
        Ok(Box::new(GeminiClient::new()))
    }

    fn provider_type(&self) -> &str {
        "gemini"
    }
}

impl Default for GeminiProviderFactory {
    fn default() -> Self {
        Self::new()
    }
}