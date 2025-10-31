// Common traits for API clients
use crate::llm_playground::ApiConfig;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

// Stream callback type for handling streaming responses
pub type StreamCallback = Box<dyn Fn(String, Option<serde_json::Value>) + 'static>;

// Function call handler type for UI layer to handle function calls
pub type FunctionCallHandler =
    Box<dyn Fn(FunctionCallRequest) -> Pin<Box<dyn Future<Output = FunctionResponse>>> + 'static>;

// Unified message structure for LLM communication
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnifiedMessage {
    pub role: UnifiedRole,
    pub content: Option<String>,
    pub function_calls: Vec<UnifiedFunctionCall>,
    pub function_responses: Vec<UnifiedFunctionResponse>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnifiedRole {
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnifiedFunctionCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnifiedFunctionResponse {
    pub name: String,
    pub content: serde_json::Value,
}

// Represents a function call request from the LLM
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionCallRequest {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

// Response from LLM that may contain text and/or function calls
#[derive(Debug, Clone, PartialEq)]
pub struct LLMResponse {
    pub content: Option<String>,
    pub function_calls: Vec<FunctionCallRequest>,
    pub finish_reason: Option<String>,
}

pub trait LLMClient {
    fn send_message(
        &self,
        messages: &[UnifiedMessage],
        config: &ApiConfig,
    ) -> Pin<Box<dyn Future<Output = Result<LLMResponse, String>>>>;

    fn send_message_stream(
        &self,
        messages: &[UnifiedMessage],
        config: &ApiConfig,
        callback: StreamCallback,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>>>>;

    fn client_name(&self) -> &str;

    // Get available models from the API
    fn get_available_models(
        &self,
        config: &ApiConfig,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, String>>>>;

    // Set system prompt - clients will handle this internally
    fn set_system_prompt(&mut self, prompt: &str);

    // Legacy compatibility methods
    fn send_message_legacy(
        &self,
        messages: &[crate::llm_playground::Message],
        config: &ApiConfig,
    ) -> Pin<Box<dyn Future<Output = Result<LLMResponse, String>>>> {
        let unified_messages = UnifiedMessage::from_legacy_messages(messages);
        self.send_message(&unified_messages, config)
    }

    fn send_message_stream_legacy(
        &self,
        messages: &[crate::llm_playground::Message],
        config: &ApiConfig,
        callback: StreamCallback,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>>>> {
        let unified_messages = UnifiedMessage::from_legacy_messages(messages);
        self.send_message_stream(&unified_messages, config, callback)
    }
}

// Trait for conversation management
pub trait ConversationManager {
    fn add_user_message(&mut self, message: &str);
    fn add_assistant_message(&mut self, message: &str, function_call: Option<serde_json::Value>);
    fn add_function_response(&mut self, function_response: &FunctionResponse);
    fn clear_conversation(&mut self);
    fn set_system_prompt(&mut self, prompt: &str);
    fn get_conversation_history(&self) -> &[ConversationMessage];
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
    pub function_call: Option<serde_json::Value>,
    pub function_response: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct FunctionResponse {
    pub id: String,
    pub name: String,
    pub response: serde_json::Value,
}

// Helper functions to convert between old Message type and new UnifiedMessage type
impl UnifiedMessage {
    /// Convert from the old Message type to UnifiedMessage
    pub fn from_legacy_message(message: &crate::llm_playground::Message) -> Self {
        use crate::llm_playground::MessageRole;
        
        let role = match message.role {
            MessageRole::System => UnifiedRole::User, // System messages are handled by clients now
            MessageRole::User => UnifiedRole::User,
            MessageRole::Assistant => UnifiedRole::Assistant,
            MessageRole::Function => UnifiedRole::Tool,
        };

        let mut function_calls = vec![];
        let mut function_responses = vec![];

        // Convert function_call to function_calls
        if let Some(fc) = &message.function_call {
            if let Ok(calls) = serde_json::from_value::<Vec<serde_json::Value>>(fc.clone()) {
                for call in calls {
                    if let (Some(name), Some(args)) = (
                        call.get("name").and_then(|v| v.as_str()),
                        call.get("arguments"),
                    ) {
                        function_calls.push(UnifiedFunctionCall {
                            name: name.to_string(),
                            arguments: args.clone(),
                        });
                    }
                }
            }
        }

        // Convert function_response to function_responses
        if let Some(fr) = &message.function_response {
            if let (Some(name), Some(response)) = (
                fr.get("name").and_then(|v| v.as_str()),
                fr.get("response"),
            ) {
                function_responses.push(UnifiedFunctionResponse {
                    name: name.to_string(),
                    content: response.clone(),
                });
            }
        }

        Self {
            role,
            content: if message.content.is_empty() { 
                None 
            } else { 
                Some(message.content.clone()) 
            },
            function_calls,
            function_responses,
        }
    }

    /// Convert a slice of legacy Messages to UnifiedMessages
    pub fn from_legacy_messages(messages: &[crate::llm_playground::Message]) -> Vec<Self> {
        messages.iter().map(Self::from_legacy_message).collect()
    }
}
