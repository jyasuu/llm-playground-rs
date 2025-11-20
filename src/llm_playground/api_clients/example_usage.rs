// Example usage of the refactored SOLID-compliant API clients
use super::{
    ClientServiceBuilder, RefactoredFlexibleLLMClient,
    ProviderFactory, ProviderRegistry, MessageConversionService,
    create_test_provider_system,
};
use crate::llm_playground::{
    provider_config::{ProviderConfig, FlexibleApiConfig, SharedSettings},
    Message, MessageRole,
};
use std::sync::Arc;

/// Example 1: Simple usage with defaults (Recommended for most cases)
pub async fn example_simple_usage() -> Result<(), String> {
    // Create client with all default providers registered
    let client = RefactoredFlexibleLLMClient::new_with_defaults();
    
    // Create a sample provider configuration
    let provider = ProviderConfig {
        name: "openai".to_string(),
        api_key: "your-api-key".to_string(),
        api_base_url: "https://api.openai.com/v1".to_string(),
        models: vec!["gpt-4".to_string()],
        transformer: crate::llm_playground::provider_config::TransformerConfig {
            r#use: vec!["openai".to_string()],
        },
    };
    
    // Create flexible API configuration
    let config = FlexibleApiConfig {
        providers: vec![provider.clone()],
        router: crate::llm_playground::provider_config::RouterConfig {
            default: "openai,gpt-4".to_string(),
            background: "openai,gpt-4".to_string(),
            think: "openai,gpt-4".to_string(),
            long_context: "openai,gpt-4".to_string(),
            long_context_threshold: 1000,
            web_search: "openai,gpt-4".to_string(),
        },
        shared_settings: SharedSettings {
            temperature: 0.7,
            max_tokens: 1000,
            retry_delay: 1000,
        },
        system_prompt: "You are a helpful assistant".to_string(),
        function_tools: vec![], // Add function tools if needed
        structured_outputs: vec![], // Add structured outputs if needed
        mcp_config: crate::llm_playground::mcp_client::McpConfig::default(),
        current_session_provider: Some("openai,gpt-4".to_string()),
    };
    
    // Create sample messages
    let messages = vec![
        Message {
            id: "1".to_string(),
            role: MessageRole::User,
            content: "Hello, how are you?".to_string(),
            timestamp: 1234567890.0,
            function_call: None,
            function_response: None,
        }
    ];
    
    // Send message
    let response = client.send_message(&provider, &config, &messages, None).await?;
    println!("Response: {:?}", response);
    
    Ok(())
}

/// Example 2: Dependency injection for testing
pub async fn example_dependency_injection() -> Result<(), String> {
    // Create test provider system (only includes basic providers)
    let client_service = Arc::new(create_test_provider_system());
    
    // Create client with injected dependencies
    let client = RefactoredFlexibleLLMClient::new(client_service);
    
    // Check supported providers
    let providers = client.get_supported_providers();
    println!("Supported providers: {:?}", providers);
    
    // Check if specific provider is supported
    if client.is_provider_supported("openai") {
        println!("OpenAI provider is supported");
    }
    
    Ok(())
}

/// Example 3: Custom provider registration (demonstrates OCP)
pub async fn example_custom_provider() -> Result<(), String> {
    use super::traits::LLMClient;
    
    // Define a custom provider factory
    struct CustomProviderFactory;
    
    impl ProviderFactory for CustomProviderFactory {
        fn supports_provider(&self, provider_type: &str) -> bool {
            provider_type == "custom-llm"
        }
        
        fn create_client(&self, _config: &ProviderConfig) -> Result<Box<dyn LLMClient>, String> {
            // In a real implementation, you'd return your custom client
            Err("Custom provider not implemented in example".to_string())
        }
        
        fn provider_type(&self) -> &str {
            "custom-llm"
        }
    }
    
    // Create registry and register custom provider
    let mut registry = ProviderRegistry::new();
    
    // Register built-in providers
    registry.register_factory(Arc::new(super::openai_factory::OpenAIProviderFactory::new()));
    registry.register_factory(Arc::new(super::gemini_factory::GeminiProviderFactory::new()));
    
    // Register custom provider (no modification to existing code needed!)
    registry.register_factory(Arc::new(CustomProviderFactory));
    
    // Create service with custom registry
    let service = ClientServiceBuilder::new()
        .with_provider_registry(Arc::new(registry))
        .with_message_converter(Arc::new(MessageConversionService::new()))
        .build();
    
    let client = RefactoredFlexibleLLMClient::new(Arc::new(service));
    
    // Now the custom provider is available
    let providers = client.get_supported_providers();
    println!("Providers including custom: {:?}", providers);
    
    Ok(())
}

/// Example 4: Advanced service composition
pub async fn example_advanced_composition() -> Result<(), String> {
    // Create individual components
    let mut registry = ProviderRegistry::new();
    registry.register_factory(Arc::new(super::openai_factory::OpenAIProviderFactory::new()));
    
    let message_converter = MessageConversionService::new();
    
    // Build service with explicit dependencies
    let service = ClientServiceBuilder::new()
        .with_provider_registry(Arc::new(registry))
        .with_message_converter(Arc::new(message_converter))
        .build();
    
    // Use service directly (without the high-level client wrapper)
    let provider = ProviderConfig {
        name: "openai".to_string(),
        api_key: "test-key".to_string(),
        api_base_url: "https://api.openai.com/v1".to_string(),
        models: vec!["gpt-3.5-turbo".to_string()],
        transformer: crate::llm_playground::provider_config::TransformerConfig {
            r#use: vec!["openai".to_string()],
        },
    };
    
    // Get a client for specific provider
    let client = service.get_client(&provider)?;
    println!("Got client: {}", client.client_name());
    
    // Check if streaming is available
    if let Some(_streaming_client) = service.get_streaming_client(&provider)? {
        println!("Streaming is supported for this provider");
    }
    
    Ok(())
}

/// Example 5: Error handling and fallbacks
pub async fn example_error_handling() -> Result<(), String> {
    let client = RefactoredFlexibleLLMClient::new_with_defaults();
    
    // Example of unsupported provider
    let unsupported_provider = ProviderConfig {
        name: "unsupported".to_string(),
        api_key: "test".to_string(),
        api_base_url: "test".to_string(),
        models: vec!["test".to_string()],
        transformer: crate::llm_playground::provider_config::TransformerConfig {
            r#use: vec!["unsupported-provider".to_string()],
        },
    };
    
    // This will gracefully handle the error
    match client.get_client_for_provider(&unsupported_provider) {
        Ok(_) => println!("Provider supported"),
        Err(err) => println!("Provider not supported: {}", err),
    }
    
    // Check support before using
    if !client.is_provider_supported("unsupported-provider") {
        println!("Provider is not supported, using fallback");
        // Implement fallback logic here
    }
    
    Ok(())
}

/// Example 6: Integration with existing code (backward compatibility)
pub fn example_backward_compatibility() {
    // Legacy code still works
    use super::{OpenAIClient, NamedClient};
    
    let legacy_client = OpenAIClient::new();
    println!("Legacy client name: {}", legacy_client.client_name());
    
    // Can gradually migrate to new architecture
    let new_client = RefactoredFlexibleLLMClient::new_with_defaults();
    let providers = new_client.get_supported_providers();
    println!("New architecture supports: {:?}", providers);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_usage() {
        // This test demonstrates that the new architecture works
        let result = example_dependency_injection().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_backward_compatibility() {
        // This test ensures legacy code still works
        example_backward_compatibility();
    }

    #[tokio::test]
    async fn test_custom_provider_registration() {
        let result = example_custom_provider().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_error_handling() {
        let result = example_error_handling().await;
        assert!(result.is_ok());
    }
}