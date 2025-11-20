// Segregated interfaces following ISP principle
use super::traits::{LLMResponse, StreamCallback, UnifiedMessage};
use crate::llm_playground::ApiConfig;
use std::future::Future;
use std::pin::Pin;

/// Core message sending capability (ISP compliance)
pub trait MessageSender {
    fn send_message<'a>(
        &'a self,
        messages: &'a [UnifiedMessage],
        config: &'a ApiConfig,
        system_prompt: Option<&'a str>,
    ) -> Pin<Box<dyn Future<Output = Result<LLMResponse, String>> + 'a>>;
}

/// Streaming message capability (ISP compliance)
pub trait StreamingSender {
    fn send_message_stream<'a>(
        &'a self,
        messages: &'a [UnifiedMessage],
        config: &'a ApiConfig,
        system_prompt: Option<&'a str>,
        callback: StreamCallback,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + 'a>>;
}

/// Model information provider (ISP compliance)
pub trait ModelProvider {
    fn get_available_models<'a>(
        &'a self,
        config: &'a ApiConfig,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, String>> + 'a>>;
}

/// Client identification (ISP compliance)
pub trait NamedClient {
    fn client_name(&self) -> &str;
}

/// Function calling capability (ISP compliance)
pub trait FunctionCaller {
    fn supports_function_calling(&self) -> bool;
    fn prepare_function_tools(&self, tools: &[serde_json::Value]) -> Result<serde_json::Value, String>;
}

/// Basic client interface that all clients must implement
pub trait BasicClient: MessageSender + NamedClient {}

/// Full-featured client that implements all capabilities
pub trait FullFeaturedClient: 
    MessageSender + 
    StreamingSender + 
    ModelProvider + 
    NamedClient + 
    FunctionCaller 
{}

/// Streaming-enabled client
pub trait StreamingClient: MessageSender + StreamingSender + NamedClient {}

/// Function-enabled client
pub trait FunctionEnabledClient: MessageSender + FunctionCaller + NamedClient {}

// Blanket implementations for composed traits
impl<T> BasicClient for T where T: MessageSender + NamedClient {}

impl<T> FullFeaturedClient for T where 
    T: MessageSender + StreamingSender + ModelProvider + NamedClient + FunctionCaller 
{}

impl<T> StreamingClient for T where T: MessageSender + StreamingSender + NamedClient {}

impl<T> FunctionEnabledClient for T where T: MessageSender + FunctionCaller + NamedClient {}