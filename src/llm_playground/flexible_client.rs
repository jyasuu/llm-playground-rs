// Flexible LLM client that can work with any provider configuration
use crate::llm_playground::api_clients::{
    GeminiClient, LLMClient, LLMResponse, OpenAIClient, StreamCallback, UnifiedMessage,
};
use crate::llm_playground::{
    provider_config::{FlexibleApiConfig, ProviderConfig},
    Message,
};
use std::future::Future;
use std::pin::Pin;
use web_sys::js_sys;

#[derive(Clone)]
pub struct FlexibleLLMClient;

impl FlexibleLLMClient {
    pub fn new() -> Self {
        Self
    }

    /// Get the appropriate client for the current session provider
    fn get_client_for_provider(&self, provider: &ProviderConfig) -> Box<dyn LLMClient> {
        // Determine which client to use based on the transformer configuration
        if provider.transformer.r#use.contains(&"gemini".to_string()) {
            Box::new(GeminiClient::new())
        } else {
            // Default to OpenAI-compatible client for everything else
            Box::new(OpenAIClient::new())
        }
    }

    /// Create a temporary config for the legacy API
    fn create_legacy_config(
        &self,
        provider: &ProviderConfig,
        config: &FlexibleApiConfig,
        model: &str,
    ) -> crate::llm_playground::ApiConfig {
        use crate::llm_playground::mcp_client::McpConfig;
        use crate::llm_playground::{ApiConfig, ApiProvider, GeminiConfig, OpenAIConfig};

        if provider.transformer.r#use.contains(&"gemini".to_string()) {
            ApiConfig {
                current_provider: ApiProvider::Gemini,
                gemini: GeminiConfig {
                    api_key: provider.api_key.clone(),
                    model: model.to_string(),
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
            // OpenAI-compatible
            ApiConfig {
                current_provider: ApiProvider::OpenAI,
                gemini: GeminiConfig {
                    api_key: "".to_string(),
                    model: "".to_string(),
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

    pub fn send_message(
        &self,
        messages: &[Message],
        config: &FlexibleApiConfig,
    ) -> Pin<Box<dyn Future<Output = Result<LLMResponse, String>>>> {
        let (provider_name, model_name) = config.get_current_provider_and_model();
        
        // Debug logging
        use gloo_console::log;
        log!("🔍 FlexibleLLMClient::send_message - Provider: {}, Model: {}", &provider_name, &model_name);
        log!("🔍 Session provider setting:", format!("{:?}", &config.current_session_provider));
        log!("🔍 Router default: {}", &config.router.default);

        if let Some(provider) = config.get_provider(&provider_name) {
            log!("🔍 Found provider: {}", &provider.name);
            log!("🔍 Provider transformer:", format!("{:?}", &provider.transformer.r#use));
            log!("🔍 Provider API URL: {}", &provider.api_base_url);
            
            let client = self.get_client_for_provider(provider);
            let legacy_config = self.create_legacy_config(provider, config, &model_name);
            
            // Log which client type we're using
            if provider.transformer.r#use.contains(&"gemini".to_string()) {
                log!("🔍 Using GeminiClient for provider: {}", &provider_name);
            } else {
                log!("🔍 Using OpenAIClient for provider: {}", &provider_name);
            }
            
            // Convert legacy messages to unified format
            let unified_messages = client.convert_legacy_messages(messages);
            
            // Extract system prompt from config
            let system_prompt = if config.system_prompt.is_empty() {
                None
            } else {
                Some(config.system_prompt.as_str())
            };
            
            log!("📤 Sending to {} client with {} unified messages...", client.client_name(), unified_messages.len());
            client.send_message(&unified_messages, &legacy_config, system_prompt)
        } else {
            let provider_name_clone = provider_name.clone();
            log!("❌ Provider '{}' not found in config", &provider_name);
            Box::pin(async move { Err(format!("Provider '{}' not found", provider_name_clone)) })
        }
    }

    pub fn send_message_stream(
        &self,
        messages: &[Message],
        config: &FlexibleApiConfig,
        callback: StreamCallback,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>>>> {
        let (provider_name, model_name) = config.get_current_provider_and_model();

        if let Some(provider) = config.get_provider(&provider_name) {
            let client = self.get_client_for_provider(provider);
            let legacy_config = self.create_legacy_config(provider, config, &model_name);
            // Convert legacy messages to unified format
            let unified_messages = client.convert_legacy_messages(messages);
            
            // Extract system prompt from config
            let system_prompt = if config.system_prompt.is_empty() {
                None
            } else {
                Some(config.system_prompt.as_str())
            };
            
            client.send_message_stream(&unified_messages, &legacy_config, system_prompt, callback)
        } else {
            Box::pin(async move { Err(format!("Provider '{}' not found", provider_name)) })
        }
    }

    pub fn get_available_models(
        &self,
        config: &FlexibleApiConfig,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, String>>>> {
        let (provider_name, _) = config.get_current_provider_and_model();

        if let Some(provider) = config.get_provider(&provider_name) {
            let client = self.get_client_for_provider(provider);
            let legacy_config = self.create_legacy_config(provider, config, &provider.models[0]);
            client.get_available_models(&legacy_config)
        } else {
            Box::pin(async move { Err(format!("Provider '{}' not found", provider_name)) })
        }
    }

    /// Get client name for the current provider
    pub fn get_client_name(&self, config: &FlexibleApiConfig) -> String {
        let (provider_name, model_name) = config.get_current_provider_and_model();
        format!("{} ({})", provider_name, model_name)
    }

    /// Validate that a provider configuration is complete
    pub fn validate_provider(&self, provider: &ProviderConfig) -> Result<(), String> {
        if provider.name.is_empty() {
            return Err("Provider name cannot be empty".to_string());
        }
        if provider.api_base_url.is_empty() {
            return Err("API base URL cannot be empty".to_string());
        }
        if provider.api_key.is_empty() && provider.name != "ollama" {
            return Err("API key cannot be empty".to_string());
        }
        if provider.models.is_empty() {
            return Err("At least one model must be specified".to_string());
        }
        if provider.transformer.r#use.is_empty() {
            return Err("Transformer configuration cannot be empty".to_string());
        }

        // Check if transformer type is supported
        let supported_transformers = ["openai", "gemini"];
        if !provider
            .transformer
            .r#use
            .iter()
            .any(|t| supported_transformers.contains(&t.as_str()))
        {
            return Err(format!(
                "Unsupported transformer type. Supported: {:?}",
                supported_transformers
            ));
        }

        Ok(())
    }

    /// Test connection to a provider
    pub fn test_connection(
        &self,
        provider: &ProviderConfig,
        config: &FlexibleApiConfig,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>>>> {
        if let Err(e) = self.validate_provider(provider) {
            return Box::pin(async move { Err(e) });
        }

        let client = self.get_client_for_provider(provider);
        let legacy_config = self.create_legacy_config(provider, config, &provider.models[0]);

        // Send a simple test message
        let test_messages = vec![Message {
            id: "test".to_string(),
            role: crate::llm_playground::MessageRole::User,
            content: "Hello, this is a connection test.".to_string(),
            timestamp: js_sys::Date::now(),
            function_call: None,
            function_response: None,
        }];

        Box::pin(async move {
            // Convert legacy messages to unified format
            let unified_messages = client.convert_legacy_messages(&test_messages);
            
            // No system prompt for test
            let system_prompt = None;
            
            match client.send_message(&unified_messages, &legacy_config, system_prompt).await {
                Ok(_) => Ok("Connection successful".to_string()),
                Err(e) => Err(format!("Connection failed: {}", e)),
            }
        })
    }
}

impl Default for FlexibleLLMClient {
    fn default() -> Self {
        Self::new()
    }
}
