// Common traits for API clients
use crate::llm_playground::{Message, ApiConfig};
use std::future::Future;
use std::pin::Pin;

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
    ) -> Pin<Box<dyn Future<Output = Result<(), String>>>>;
    
    fn client_name(&self) -> &str;
}