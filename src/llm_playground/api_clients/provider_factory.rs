// Provider factory pattern for extensible client creation (OCP compliance)
use super::traits::LLMClient;
use crate::llm_playground::provider_config::ProviderConfig;
use std::collections::HashMap;
use std::sync::Arc;

/// Factory trait for creating provider-specific clients
/// This follows the Factory pattern and enables OCP compliance
pub trait ProviderFactory: Send + Sync {
    /// Check if this factory supports the given provider type
    fn supports_provider(&self, provider_type: &str) -> bool;
    
    /// Create a client instance for the provider
    fn create_client(&self, config: &ProviderConfig) -> Result<Box<dyn LLMClient>, String>;
    
    /// Get the provider type name
    fn provider_type(&self) -> &str;
}

/// Registry for managing provider factories
/// Follows the Registry pattern for extensibility
pub struct ProviderRegistry {
    factories: HashMap<String, Arc<dyn ProviderFactory>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    /// Register a new provider factory
    pub fn register_factory(&mut self, factory: Arc<dyn ProviderFactory>) {
        let provider_type = factory.provider_type().to_string();
        self.factories.insert(provider_type, factory);
    }

    /// Create a client for the given provider configuration
    pub fn create_client(&self, config: &ProviderConfig) -> Result<Box<dyn LLMClient>, String> {
        // Try to find a factory that supports this provider
        for factory in self.factories.values() {
            if factory.supports_provider(&self.detect_provider_type(config)) {
                return factory.create_client(config);
            }
        }
        
        Err(format!("No factory found for provider configuration: {:?}", config.transformer.r#use))
    }

    /// Detect provider type from configuration
    fn detect_provider_type(&self, config: &ProviderConfig) -> String {
        if config.transformer.r#use.contains(&"gemini".to_string()) {
            "gemini".to_string()
        } else {
            "openai".to_string()
        }
    }

    /// Get all supported provider types
    pub fn get_supported_providers(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}