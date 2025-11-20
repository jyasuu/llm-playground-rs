// Common traits for API clients
use crate::llm_playground::{ApiConfig, Message};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

// Stream callback type for handling streaming responses
pub type StreamCallback = Box<dyn Fn(String, Option<serde_json::Value>) + 'static>;

// Function call handler type for UI layer to handle function calls
pub type FunctionCallHandler =
    Box<dyn Fn(FunctionCallRequest) -> Pin<Box<dyn Future<Output = FunctionResponse>>> + 'static>;

// Represents a function call request from the LLM
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

// Unified message structure for internal LLM client communication
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnifiedMessage {
    pub id: String,
    pub role: UnifiedMessageRole,
    pub content: Option<String>,
    pub timestamp: f64,
    pub function_calls: Vec<FunctionCallRequest>,
    pub function_responses: Vec<FunctionResponse>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnifiedMessageRole {
    System,
    User,
    Assistant,
}

// Represents a client that can send non-streaming messages
pub trait MessageSender {
    fn send_message(
        &self,
        messages: &[UnifiedMessage],
        config: &ApiConfig,
        system_prompt: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<LLMResponse, String>> + '_>>;
}

// Represents a client that can send streaming messages
pub trait StreamingSender {
    fn send_message_stream(
        &self,
        messages: &[UnifiedMessage],
        config: &ApiConfig,
        system_prompt: Option<&str>,
        callback: StreamCallback,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + '_>>;
}

// Represents a client that can provide a list of available models
pub trait ModelProvider {
    fn get_available_models(
        &self,
        config: &ApiConfig,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, String>> + '_>>;
}

// Represents a utility for converting message formats
pub trait MessageConverter {
    fn convert_legacy_messages(&self, messages: &[Message]) -> Vec<UnifiedMessage>;
}

// Represents a client that has a name
pub trait NamedClient {
    fn client_name(&self) -> &str;
}

// Updated LLMClient trait composed of smaller, focused traits.
// Any client that implements all the smaller traits automatically implements LLMClient.
pub trait LLMClient:
    MessageSender + StreamingSender + ModelProvider + MessageConverter + NamedClient
{
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
    pub function_call: Option<serde_json::Value>,
    pub function_response: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionResponse {
    pub id: String,
    pub name: String,
    pub response: serde_json::Value,
}
