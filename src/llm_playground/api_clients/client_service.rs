// Client service with dependency injection (DIP compliance)
use super::traits::LLMClient;
use super::message_service::MessageConversionService;
use super::provider_factory::ProviderRegistry;
use super::traits::LLMResponse;
use crate::llm_playground::{provider_config::ProviderConfig, ApiConfig, Message};
use std::sync::Arc;

/// High-level client service that coordinates between different components
/// Follows DIP by depending on abstractions, not concretions
pub struct ClientService {
    provider_registry: Arc<ProviderRegistry>,
    message_converter: Arc<MessageConversionService>,
}

impl ClientService {
    /// Constructor with dependency injection (DIP compliance)
    pub fn new(
        provider_registry: Arc<ProviderRegistry>,
        message_converter: Arc<MessageConversionService>,
    ) -> Self {
        Self {
            provider_registry,
            message_converter,
        }
    }

    /// Send message using the appropriate provider
    pub async fn send_message(
        &self,
        provider_config: &ProviderConfig,
        api_config: &ApiConfig,
        messages: &[Message],
        system_prompt: Option<&str>,
    ) -> Result<LLMResponse, String> {
        // Convert messages using the injected converter
        let unified_messages = self.message_converter.convert_legacy_to_unified(messages);
        
        // Get client from the injected registry
        let client = self.provider_registry.create_client(provider_config)?;
        
        // Send message
        client.send_message(&unified_messages, api_config, system_prompt).await
    }

    /// Get client for provider (useful for advanced use cases)
    pub fn get_client(&self, provider_config: &ProviderConfig) -> Result<Box<dyn LLMClient>, String> {
        self.provider_registry.create_client(provider_config)
    }

    /// Get streaming client if supported
    pub fn get_streaming_client(&self, provider_config: &ProviderConfig) -> Result<Option<Box<dyn LLMClient>>, String> {
        let client = self.provider_registry.create_client(provider_config)?;
        Ok(Some(client))
    }

    /// List all supported providers
    pub fn get_supported_providers(&self) -> Vec<String> {
        self.provider_registry.get_supported_providers()
    }
}

/// Builder for ClientService with default dependencies
pub struct ClientServiceBuilder {
    provider_registry: Option<Arc<ProviderRegistry>>,
    message_converter: Option<Arc<MessageConversionService>>,
}

impl ClientServiceBuilder {
    pub fn new() -> Self {
        Self {
            provider_registry: None,
            message_converter: None,
        }
    }

    pub fn with_provider_registry(mut self, registry: Arc<ProviderRegistry>) -> Self {
        self.provider_registry = Some(registry);
        self
    }

    pub fn with_message_converter(mut self, converter: Arc<MessageConversionService>) -> Self {
        self.message_converter = Some(converter);
        self
    }

    pub fn build(self) -> ClientService {
        ClientService::new(
            self.provider_registry.unwrap_or_else(|| Arc::new(ProviderRegistry::default())),
            self.message_converter.unwrap_or_else(|| Arc::new(MessageConversionService::default())),
        )
    }
}

impl Default for ClientServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}