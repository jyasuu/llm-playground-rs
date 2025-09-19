// API client modules
pub mod gemini_client;
pub mod openai_client;
pub mod traits;

pub use gemini_client::GeminiClient;
pub use openai_client::OpenAIClient;
pub use traits::{LLMClient, ConversationManager, ConversationMessage, FunctionResponse, StreamCallback, FunctionCallHandler, FunctionCallRequest, LLMResponse};