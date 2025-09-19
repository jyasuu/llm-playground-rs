// Common traits for API clients
use crate::llm_playground::{Message, ApiConfig};
use std::future::Future;
use std::pin::Pin;
use wasm_bindgen_futures::spawn_local;
use gloo_timers::callback::Timeout;
use web_sys::js_sys;

// Stream callback type for handling streaming responses
pub type StreamCallback = Box<dyn Fn(String, Option<serde_json::Value>) + 'static>;

pub trait LLMClient {
    fn send_message(
        &self,
        messages: &[Message],
        config: &ApiConfig,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>>>>;
    
    fn send_message_stream(
        &self,
        messages: &[Message],
        config: &ApiConfig,
        callback: StreamCallback,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>>>>;
    
    fn client_name(&self) -> &str;
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

#[derive(Debug, Clone)]
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