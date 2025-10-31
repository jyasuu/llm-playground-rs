// API client modules
pub mod gemini_client;
pub mod openai_client;
pub mod traits;

pub use gemini_client::GeminiClient;
pub use openai_client::OpenAIClient;
pub use traits::{
    ConversationManager, ConversationMessage, FunctionCallRequest, FunctionResponse, LLMClient,
    LLMResponse, StreamCallback, UnifiedMessage, UnifiedRole, UnifiedFunctionCall, UnifiedFunctionResponse,
};
