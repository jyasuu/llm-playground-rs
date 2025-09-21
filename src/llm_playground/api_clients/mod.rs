// API client modules
pub mod gemini_client;
pub mod openai_client;
pub mod mcp_client;
pub mod traits;

pub use gemini_client::GeminiClient;
pub use openai_client::OpenAIClient;
pub use mcp_client::{McpClient, McpConfig};
pub use traits::{LLMClient, ConversationManager, ConversationMessage, FunctionResponse, StreamCallback, FunctionCallRequest, LLMResponse};