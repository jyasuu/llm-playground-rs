// Unified LLM client with generic data model for both Gemini and OpenAI
use crate::llm_playground::api_clients::{
    gemini_client::GeminiClient, openai_client::OpenAIClient, LLMClient, LLMResponse,
    StreamCallback, FunctionCallRequest,
};
use crate::llm_playground::FlexibleApiConfig;
use gloo_console::log;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

/// Unified message structure that abstracts away provider-specific differences
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnifiedMessage {
    pub role: UnifiedRole,
    pub content: Option<String>,
    pub function_calls: Vec<UnifiedFunctionCall>,
    pub function_responses: Vec<UnifiedFunctionResponse>,
    pub timestamp: f64,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnifiedRole {
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnifiedFunctionCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnifiedFunctionResponse {
    pub id: String,
    pub name: String,
    pub content: serde_json::Value,
}

/// Unified conversation that handles both providers transparently
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedConversation {
    pub messages: Vec<UnifiedMessage>,
    pub system_prompt: Option<String>,
}

/// Provider-agnostic LLM client
#[derive(Clone)]
pub struct UnifiedLLMClient {
    conversation: UnifiedConversation,
}

impl UnifiedLLMClient {
    pub fn new() -> Self {
        Self {
            conversation: UnifiedConversation {
                messages: Vec::new(),
                system_prompt: None,
            },
        }
    }

    /// Set system prompt (handled internally by each provider)
    pub fn set_system_prompt(&mut self, prompt: &str) {
        self.conversation.system_prompt = Some(prompt.to_string());
    }

    /// Add a user message to the conversation
    pub fn add_user_message(&mut self, content: &str) -> String {
        let message_id = format!("msg_{}", js_sys::Date::now() as u64);
        let message = UnifiedMessage {
            id: message_id.clone(),
            role: UnifiedRole::User,
            content: Some(content.to_string()),
            function_calls: Vec::new(),
            function_responses: Vec::new(),
            timestamp: js_sys::Date::now(),
        };
        self.conversation.messages.push(message);
        message_id
    }

    /// Add an assistant message with optional function calls
    pub fn add_assistant_message(
        &mut self,
        content: Option<String>,
        function_calls: Vec<UnifiedFunctionCall>,
    ) -> String {
        let message_id = format!("msg_{}", js_sys::Date::now() as u64);
        let message = UnifiedMessage {
            id: message_id.clone(),
            role: UnifiedRole::Assistant,
            content,
            function_calls,
            function_responses: Vec::new(),
            timestamp: js_sys::Date::now(),
        };
        self.conversation.messages.push(message);
        message_id
    }

    /// Add function responses (tool results)
    pub fn add_function_responses(&mut self, responses: Vec<UnifiedFunctionResponse>) -> String {
        let message_id = format!("msg_{}", js_sys::Date::now() as u64);
        let message = UnifiedMessage {
            id: message_id.clone(),
            role: UnifiedRole::Tool,
            content: None,
            function_calls: Vec::new(),
            function_responses: responses,
            timestamp: js_sys::Date::now(),
        };
        self.conversation.messages.push(message);
        message_id
    }

    /// Clear the conversation history
    pub fn clear_conversation(&mut self) {
        self.conversation.messages.clear();
    }

    /// Get the conversation history
    pub fn get_conversation(&self) -> &UnifiedConversation {
        &self.conversation
    }

    /// Get mutable conversation for external updates
    pub fn get_conversation_mut(&mut self) -> &mut UnifiedConversation {
        &mut self.conversation
    }

    /// Send message using the appropriate provider (FlexibleApiConfig)
    pub fn send_message(
        &self,
        config: &FlexibleApiConfig,
    ) -> Pin<Box<dyn Future<Output = Result<LLMResponse, String>>>> {
        // Convert unified messages to the format expected by the trait
        let trait_messages = self.convert_to_trait_messages();
        
        // Convert FlexibleApiConfig to legacy ApiConfig for provider clients
        let legacy_config = self.convert_to_legacy_config(config);
        
        // Determine provider from FlexibleApiConfig
        let (provider_name, _model_name) = config.get_current_provider_and_model();
        let is_gemini = if let Some(provider) = config.get_provider(&provider_name) {
            provider.transformer.r#use.contains(&"gemini".to_string())
        } else {
            false
        };
        
        if is_gemini {
            let mut client = GeminiClient::new();
            if let Some(prompt) = &self.conversation.system_prompt {
                client.set_system_prompt(prompt);
            }
            client.send_message(&trait_messages, &legacy_config)
        } else {
            let mut client = OpenAIClient::new();
            if let Some(prompt) = &self.conversation.system_prompt {
                client.set_system_prompt(prompt);
            }
            client.send_message(&trait_messages, &legacy_config)
        }
    }

    /// Send message with streaming
    pub fn send_message_stream(
        &self,
        config: &FlexibleApiConfig,
        callback: StreamCallback,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>>>> {
        let trait_messages = self.convert_to_trait_messages();
        let legacy_config = self.convert_to_legacy_config(config);
        
        // Determine provider from FlexibleApiConfig
        let (provider_name, _model_name) = config.get_current_provider_and_model();
        let is_gemini = if let Some(provider) = config.get_provider(&provider_name) {
            provider.transformer.r#use.contains(&"gemini".to_string())
        } else {
            false
        };
        
        if is_gemini {
            let mut client = GeminiClient::new();
            if let Some(prompt) = &self.conversation.system_prompt {
                client.set_system_prompt(prompt);
            }
            client.send_message_stream(&trait_messages, &legacy_config, callback)
        } else {
            let mut client = OpenAIClient::new();
            if let Some(prompt) = &self.conversation.system_prompt {
                client.set_system_prompt(prompt);
            }
            client.send_message_stream(&trait_messages, &legacy_config, callback)
        }
    }

    /// Convert unified messages to the trait format
    fn convert_to_trait_messages(&self) -> Vec<crate::llm_playground::api_clients::UnifiedMessage> {
        self.conversation
            .messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    UnifiedRole::User => crate::llm_playground::api_clients::UnifiedRole::User,
                    UnifiedRole::Assistant => crate::llm_playground::api_clients::UnifiedRole::Assistant,
                    UnifiedRole::Tool => crate::llm_playground::api_clients::UnifiedRole::Tool,
                };

                let function_calls = msg
                    .function_calls
                    .iter()
                    .map(|fc| crate::llm_playground::api_clients::UnifiedFunctionCall {
                        name: fc.name.clone(),
                        arguments: fc.arguments.clone(),
                    })
                    .collect();

                let function_responses = msg
                    .function_responses
                    .iter()
                    .map(|fr| crate::llm_playground::api_clients::UnifiedFunctionResponse {
                        name: fr.name.clone(),
                        content: fr.content.clone(),
                    })
                    .collect();

                crate::llm_playground::api_clients::UnifiedMessage {
                    role,
                    content: msg.content.clone(),
                    function_calls,
                    function_responses,
                }
            })
            .collect()
    }

    /// Convert LLM response function calls to unified format
    pub fn convert_function_calls_to_unified(
        &self,
        function_calls: Vec<FunctionCallRequest>,
    ) -> Vec<UnifiedFunctionCall> {
        function_calls
            .into_iter()
            .map(|fc| UnifiedFunctionCall {
                id: fc.id,
                name: fc.name,
                arguments: fc.arguments,
            })
            .collect()
    }

    /// Create unified function responses from tool execution results
    pub fn create_function_responses(
        &self,
        function_calls: &[UnifiedFunctionCall],
        results: Vec<serde_json::Value>,
    ) -> Vec<UnifiedFunctionResponse> {
        function_calls
            .iter()
            .zip(results.into_iter())
            .map(|(fc, result)| UnifiedFunctionResponse {
                id: fc.id.clone(),
                name: fc.name.clone(),
                content: result,
            })
            .collect()
    }

    /// Convert to legacy Message format for backward compatibility
    pub fn to_legacy_messages(&self) -> Vec<crate::llm_playground::Message> {
        self.conversation
            .messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    UnifiedRole::User => crate::llm_playground::MessageRole::User,
                    UnifiedRole::Assistant => crate::llm_playground::MessageRole::Assistant,
                    UnifiedRole::Tool => crate::llm_playground::MessageRole::Function,
                };

                // Convert function calls to legacy format
                let function_call = if !msg.function_calls.is_empty() {
                    let calls: Vec<serde_json::Value> = msg
                        .function_calls
                        .iter()
                        .map(|fc| {
                            serde_json::json!({
                                "id": fc.id,
                                "name": fc.name,
                                "arguments": fc.arguments
                            })
                        })
                        .collect();
                    Some(serde_json::Value::Array(calls))
                } else {
                    None
                };

                // Convert function responses to legacy format
                let function_response = if !msg.function_responses.is_empty() {
                    // For legacy compatibility, take the first response
                    msg.function_responses.first().map(|fr| {
                        serde_json::json!({
                            "id": fr.id,
                            "name": fr.name,
                            "response": fr.content
                        })
                    })
                } else {
                    None
                };

                crate::llm_playground::Message {
                    id: msg.id.clone(),
                    role,
                    content: msg.content.clone().unwrap_or_default(),
                    timestamp: msg.timestamp,
                    function_call,
                    function_response,
                }
            })
            .collect()
    }

    /// Create from legacy messages for backward compatibility
    pub fn from_legacy_messages(messages: &[crate::llm_playground::Message]) -> Self {
        let unified_messages = messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    crate::llm_playground::MessageRole::System => UnifiedRole::User, // System messages become user messages
                    crate::llm_playground::MessageRole::User => UnifiedRole::User,
                    crate::llm_playground::MessageRole::Assistant => UnifiedRole::Assistant,
                    crate::llm_playground::MessageRole::Function => UnifiedRole::Tool,
                };

                // Convert legacy function calls
                let function_calls = if let Some(fc_value) = &msg.function_call {
                    if let Ok(calls) = serde_json::from_value::<Vec<serde_json::Value>>(fc_value.clone()) {
                        calls
                            .into_iter()
                            .filter_map(|call| {
                                let id = call.get("id")?.as_str()?.to_string();
                                let name = call.get("name")?.as_str()?.to_string();
                                let arguments = call.get("arguments")?.clone();
                                Some(UnifiedFunctionCall { id, name, arguments })
                            })
                            .collect()
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                };

                // Convert legacy function responses
                let function_responses = if let Some(fr_value) = &msg.function_response {
                    if let (Some(id), Some(name), Some(response)) = (
                        fr_value.get("id").and_then(|v| v.as_str()),
                        fr_value.get("name").and_then(|v| v.as_str()),
                        fr_value.get("response"),
                    ) {
                        vec![UnifiedFunctionResponse {
                            id: id.to_string(),
                            name: name.to_string(),
                            content: response.clone(),
                        }]
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                };

                UnifiedMessage {
                    id: msg.id.clone(),
                    role,
                    content: if msg.content.is_empty() {
                        None
                    } else {
                        Some(msg.content.clone())
                    },
                    function_calls,
                    function_responses,
                    timestamp: msg.timestamp,
                }
            })
            .collect();

        Self {
            conversation: UnifiedConversation {
                messages: unified_messages,
                system_prompt: None,
            },
        }
    }
}

/// Helper functions for provider-specific adaptations
impl UnifiedLLMClient {
    /// Convert FlexibleApiConfig to legacy ApiConfig for provider clients
    fn convert_to_legacy_config(&self, config: &FlexibleApiConfig) -> crate::llm_playground::ApiConfig {
        let (provider_name, model_name) = config.get_current_provider_and_model();

        if let Some(provider) = config.get_provider(&provider_name) {
            if provider.transformer.r#use.contains(&"gemini".to_string()) {
                crate::llm_playground::ApiConfig {
                    current_provider: crate::llm_playground::ApiProvider::Gemini,
                    gemini: crate::llm_playground::GeminiConfig {
                        api_key: provider.api_key.clone(),
                        model: model_name.clone(),
                    },
                    openai: crate::llm_playground::OpenAIConfig {
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
                    function_tools: config.function_tools.clone(),
                    structured_outputs: config.structured_outputs.clone(),
                    mcp_config: crate::llm_playground::mcp_client::McpConfig::default(),
                }
            } else {
                crate::llm_playground::ApiConfig {
                    current_provider: crate::llm_playground::ApiProvider::OpenAI,
                    gemini: crate::llm_playground::GeminiConfig {
                        api_key: "".to_string(),
                        model: "".to_string(),
                    },
                    openai: crate::llm_playground::OpenAIConfig {
                        base_url: provider.api_base_url.clone(),
                        api_key: provider.api_key.clone(),
                        model: model_name.clone(),
                    },
                    shared_settings: crate::llm_playground::types::SharedSettings {
                        temperature: config.shared_settings.temperature,
                        max_tokens: config.shared_settings.max_tokens,
                        retry_delay: config.shared_settings.retry_delay,
                    },
                    system_prompt: config.system_prompt.clone(),
                    function_tools: config.function_tools.clone(),
                    structured_outputs: config.structured_outputs.clone(),
                    mcp_config: crate::llm_playground::mcp_client::McpConfig::default(),
                }
            }
        } else {
            crate::llm_playground::ApiConfig::default()
        }
    }

    /// Generate provider-appropriate function call IDs
    pub fn generate_function_call_id(&self, function_name: &str, provider: &crate::llm_playground::ApiProvider) -> String {
        match provider {
            crate::llm_playground::ApiProvider::Gemini => {
                format!("gemini-{}-{}", function_name, js_sys::Date::now() as u64)
            }
            crate::llm_playground::ApiProvider::OpenAI => {
                format!("call_{}", js_sys::Date::now() as u64)
            }
        }
    }

    /// Log conversation state for debugging
    pub fn log_conversation_state(&self) {
        log!("=== Unified Conversation State ===");
        log!(format!("System prompt: {:?}", self.conversation.system_prompt));
        log!(format!("Messages count: {}", self.conversation.messages.len()));
        for (i, msg) in self.conversation.messages.iter().enumerate() {
            log!(format!(
                "Message {}: role={:?}, content_len={}, function_calls={}, function_responses={}",
                i,
                msg.role,
                msg.content.as_ref().map(|c| c.len()).unwrap_or(0),
                msg.function_calls.len(),
                msg.function_responses.len()
            ));
        }
        log!("=== End Conversation State ===");
    }
}

impl Default for UnifiedLLMClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_client() {
        let client = UnifiedLLMClient::new();
        assert!(client.conversation.messages.is_empty());
        assert!(client.conversation.system_prompt.is_none());
    }

    #[test]
    fn test_add_user_message() {
        let mut client = UnifiedLLMClient::new();
        let id = client.add_user_message("Hello");
        
        assert_eq!(client.conversation.messages.len(), 1);
        assert_eq!(client.conversation.messages[0].id, id);
        assert_eq!(client.conversation.messages[0].content, Some("Hello".to_string()));
        assert!(matches!(client.conversation.messages[0].role, UnifiedRole::User));
    }

    #[test]
    fn test_add_assistant_message() {
        let mut client = UnifiedLLMClient::new();
        let function_calls = vec![UnifiedFunctionCall {
            id: "test_id".to_string(),
            name: "test_function".to_string(),
            arguments: serde_json::json!({"arg": "value"}),
        }];
        
        let id = client.add_assistant_message(Some("Response".to_string()), function_calls.clone());
        
        assert_eq!(client.conversation.messages.len(), 1);
        assert_eq!(client.conversation.messages[0].id, id);
        assert_eq!(client.conversation.messages[0].content, Some("Response".to_string()));
        assert_eq!(client.conversation.messages[0].function_calls, function_calls);
        assert!(matches!(client.conversation.messages[0].role, UnifiedRole::Assistant));
    }

    #[test]
    fn test_set_system_prompt() {
        let mut client = UnifiedLLMClient::new();
        client.set_system_prompt("You are helpful");
        assert_eq!(client.conversation.system_prompt, Some("You are helpful".to_string()));
    }

    #[test]
    fn test_clear_conversation() {
        let mut client = UnifiedLLMClient::new();
        client.add_user_message("Hello");
        client.clear_conversation();
        assert!(client.conversation.messages.is_empty());
    }

    #[test]
    fn test_generate_function_call_id() {
        let client = UnifiedLLMClient::new();
        
        let gemini_id = client.generate_function_call_id("test", &crate::llm_playground::ApiProvider::Gemini);
        assert!(gemini_id.starts_with("gemini-test-"));
        
        let openai_id = client.generate_function_call_id("test", &crate::llm_playground::ApiProvider::OpenAI);
        assert!(openai_id.starts_with("call_"));
    }
}