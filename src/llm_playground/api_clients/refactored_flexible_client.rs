// Refactored flexible client using SOLID principles
use super::client_service::ClientService;
use super::factory_setup::initialize_provider_system;
use super::traits::LLMClient;
use super::traits::{LLMResponse, StreamCallback, UnifiedMessage};
use crate::llm_playground::{
    provider_config::{FlexibleApiConfig, ProviderConfig},
    Message, ApiConfig,
};
use std::sync::Arc;

/// Refactored flexible LLM client following SOLID principles
/// This client uses dependency injection and follows the Single Responsibility Principle
pub struct RefactoredFlexibleLLMClient {
    client_service: Arc<ClientService>,
}

impl RefactoredFlexibleLLMClient {
    /// Create new client with dependency injection (DIP compliance)
    pub fn new(client_service: Arc<ClientService>) -> Self {
        Self { client_service }
    }

    /// Create client with default configuration
    pub fn new_with_defaults() -> Self {
        let service = initialize_provider_system();
        Self {
            client_service: Arc::new(service),
        }
    }

    /// Send message using the new architecture
    pub async fn send_message(
        &self,
        provider: &ProviderConfig,
        config: &FlexibleApiConfig,
        messages: &[Message],
        system_prompt: Option<&str>,
    ) -> Result<LLMResponse, String> {
        // Convert flexible config to legacy config for compatibility
        let model = provider.models.first().unwrap_or(&"default".to_string()).clone();
        let legacy_config = self.create_legacy_config(provider, config, &model);
        
        self.client_service
            .send_message(provider, &legacy_config, messages, system_prompt)
            .await
    }

    /// Send streaming message
    pub async fn send_message_stream(
        &self,
        provider: &ProviderConfig,
        config: &FlexibleApiConfig,
        messages: &[Message],
        system_prompt: Option<&str>,
        callback: StreamCallback,
    ) -> Result<(), String> {
        // Get streaming client if available
        if let Some(streaming_client) = self.client_service.get_streaming_client(provider)? {
            let model = provider.models.first().unwrap_or(&"default".to_string()).clone();
            let legacy_config = self.create_legacy_config(provider, config, &model);
            let unified_messages = self.convert_messages_to_unified(messages);
            
            streaming_client
                .send_message_stream(&unified_messages, &legacy_config, system_prompt, callback)
                .await
        } else {
            // Fallback to regular message sending
            let response = self.send_message(provider, config, messages, system_prompt).await?;
            if let Some(content) = response.content {
                callback(content, None);
            }
            Ok(())
        }
    }

    /// Get client for a specific provider (useful for advanced use cases)
    pub fn get_client_for_provider(&self, provider: &ProviderConfig) -> Result<Box<dyn LLMClient>, String> {
        self.client_service.get_client(provider)
    }

    /// List all supported providers
    pub fn get_supported_providers(&self) -> Vec<String> {
        self.client_service.get_supported_providers()
    }

    /// Check if a provider is supported
    pub fn is_provider_supported(&self, provider_type: &str) -> bool {
        self.get_supported_providers().contains(&provider_type.to_string())
    }

    // Helper methods (SRP - single responsibility for conversion logic)
    
    fn convert_messages_to_unified(&self, messages: &[Message]) -> Vec<UnifiedMessage> {
        // This could be delegated to the injected message converter
        // For now, simplified implementation
        messages
            .iter()
            .map(|msg| UnifiedMessage {
                id: msg.id.clone(),
                role: match msg.role {
                    crate::llm_playground::MessageRole::User => super::traits::UnifiedMessageRole::User,
                    crate::llm_playground::MessageRole::Assistant => super::traits::UnifiedMessageRole::Assistant,
                    crate::llm_playground::MessageRole::System => super::traits::UnifiedMessageRole::System,
                    crate::llm_playground::MessageRole::Function => super::traits::UnifiedMessageRole::Assistant, // Map function to assistant
                },
                content: Some(msg.content.clone()),
                timestamp: msg.timestamp,
                function_calls: vec![],
                function_responses: vec![],
            })
            .collect()
    }

    fn create_legacy_config(
        &self,
        provider: &ProviderConfig,
        config: &FlexibleApiConfig,
        model: &str,
    ) -> ApiConfig {
        use crate::llm_playground::mcp_client::McpConfig;
        use crate::llm_playground::{ApiProvider, GeminiConfig, OpenAIConfig};

        if provider.transformer.r#use.contains(&"gemini".to_string()) {
            ApiConfig {
                current_provider: ApiProvider::Gemini,
                gemini: GeminiConfig {
                    api_key: provider.api_key.clone(),
                    model: model.to_string(),
                    base_url: provider.api_base_url.clone(),
                },
                openai: OpenAIConfig {
                    base_url: "".to_string(),
                    api_key: "".to_string(),
                    model: "".to_string(),
                },
                shared_settings: crate::llm_playground::types::SharedSettings {
                    temperature: config.shared_settings.temperature,
                    max_tokens: config.shared_settings.max_tokens,
                    retry_delay: config.shared_settings.retry_delay,
                },
                system_prompt: config.system_prompt.clone(),
                function_tools: config
                    .get_enabled_function_tools()
                    .into_iter()
                    .cloned()
                    .collect(),
                structured_outputs: config.structured_outputs.clone(),
                mcp_config: McpConfig::default(),
            }
        } else {
            ApiConfig {
                current_provider: ApiProvider::OpenAI,
                gemini: GeminiConfig {
                    api_key: "".to_string(),
                    model: "".to_string(),
                    base_url: "".to_string(),
                },
                openai: OpenAIConfig {
                    base_url: provider.api_base_url.clone(),
                    api_key: provider.api_key.clone(),
                    model: model.to_string(),
                },
                shared_settings: crate::llm_playground::types::SharedSettings {
                    temperature: config.shared_settings.temperature,
                    max_tokens: config.shared_settings.max_tokens,
                    retry_delay: config.shared_settings.retry_delay,
                },
                system_prompt: config.system_prompt.clone(),
                function_tools: config
                    .get_enabled_function_tools()
                    .into_iter()
                    .cloned()
                    .collect(),
                structured_outputs: config.structured_outputs.clone(),
                mcp_config: McpConfig::default(),
            }
        }
    }
}

impl Default for RefactoredFlexibleLLMClient {
    fn default() -> Self {
        Self::new_with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm_playground::provider_config::SharedSettings;

    #[test]
    fn test_client_creation() {
        let client = RefactoredFlexibleLLMClient::new_with_defaults();
        let providers = client.get_supported_providers();
        assert!(!providers.is_empty());
    }

    #[test]
    fn test_provider_support_check() {
        let client = RefactoredFlexibleLLMClient::new_with_defaults();
        assert!(client.is_provider_supported("openai"));
        assert!(client.is_provider_supported("gemini"));
        assert!(!client.is_provider_supported("unknown"));
    }
}