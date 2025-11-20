// Factory setup and initialization for the refactored system
use super::client_service::{ClientService, ClientServiceBuilder};
use super::gemini_factory::GeminiProviderFactory;
use super::message_service::MessageConversionService;
use super::openai_factory::OpenAIProviderFactory;
use super::provider_factory::ProviderRegistry;
use std::sync::Arc;

/// Initialize the provider system with all available factories
/// This demonstrates how to set up the system following SOLID principles
pub fn initialize_provider_system() -> ClientService {
    let mut registry = ProviderRegistry::new();
    
    // Register all available provider factories (OCP compliance)
    registry.register_factory(Arc::new(OpenAIProviderFactory::new()));
    registry.register_factory(Arc::new(GeminiProviderFactory::new()));
    
    // You can easily add new providers here without modifying existing code:
    // registry.register_factory(Arc::new(AnthropicProviderFactory::new()));
    // registry.register_factory(Arc::new(OllamaProviderFactory::new()));
    
    let message_converter = Arc::new(MessageConversionService::new());
    
    // Build the client service with dependency injection (DIP compliance)
    ClientServiceBuilder::new()
        .with_provider_registry(Arc::new(registry))
        .with_message_converter(message_converter)
        .build()
}

/// Create a minimal provider system for testing
pub fn create_test_provider_system() -> ClientService {
    let mut registry = ProviderRegistry::new();
    registry.register_factory(Arc::new(OpenAIProviderFactory::new()));
    
    ClientServiceBuilder::new()
        .with_provider_registry(Arc::new(registry))
        .build()
}

/// Example of how to add a new provider without modifying existing code
/// This demonstrates the Open/Closed Principle in action
#[cfg(feature = "example_extension")]
pub fn add_custom_provider(service: &mut ProviderRegistry) {
    use super::provider_factory::ProviderFactory;
    
    // Example custom provider factory
    struct CustomProviderFactory;
    
    impl ProviderFactory for CustomProviderFactory {
        fn supports_provider(&self, provider_type: &str) -> bool {
            provider_type == "custom"
        }
        
        fn create_client(&self, _config: &crate::llm_playground::provider_config::ProviderConfig) -> Result<Box<dyn super::traits::LLMClient>, String> {
            // Return a custom client implementation
            Err("Custom provider not implemented".to_string())
        }
        
        fn provider_type(&self) -> &str {
            "custom"
        }
    }
    
    service.register_factory(Arc::new(CustomProviderFactory));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_system_initialization() {
        let service = initialize_provider_system();
        let providers = service.get_supported_providers();
        
        assert!(providers.contains(&"openai".to_string()));
        assert!(providers.contains(&"gemini".to_string()));
        assert_eq!(providers.len(), 2);
    }

    #[test]
    fn test_minimal_provider_system() {
        let service = create_test_provider_system();
        let providers = service.get_supported_providers();
        
        assert!(providers.contains(&"openai".to_string()));
        assert_eq!(providers.len(), 1);
    }
}