// Type definitions for LLM Playground
use serde::{Deserialize, Serialize};
use crate::llm_playground::api_clients::McpConfig;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ApiProvider {
    Gemini,
    OpenAI,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApiConfig {
    pub gemini: GeminiConfig,
    pub openai: OpenAIConfig,
    pub mcp: McpConfig,
    pub current_provider: ApiProvider,
    pub shared_settings: SharedSettings,
    pub system_prompt: String,
    pub function_tools: Vec<FunctionTool>,
    pub structured_outputs: Vec<StructuredOutput>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeminiConfig {
    pub api_key: String,
    pub model: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SharedSettings {
    pub temperature: f32,
    pub max_tokens: u32,
    pub retry_delay: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FunctionTool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub mock_response: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StructuredOutput {
    pub name: String,
    pub schema: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: f64,
    pub function_call: Option<serde_json::Value>,
    pub function_response: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Function,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub messages: Vec<Message>,
    pub created_at: f64,
    pub updated_at: f64,
    pub pinned: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            gemini: GeminiConfig {
                api_key: String::new(),
                model: "gemini-2.5-flash-lite-preview-06-17".to_string(),
            },
            openai: OpenAIConfig {
                base_url: "https://api.openai.com/v1".to_string(),
                api_key: String::new(),
                model: "gpt-4o".to_string(),
            },
            mcp: McpConfig::default(),
            current_provider: ApiProvider::Gemini,
            shared_settings: SharedSettings {
                temperature: 0.7,
                max_tokens: 2048,
                retry_delay: 2000,
            },
            system_prompt: "You are a helpful assistant that responds in markdown format. Always be concise and to the point.".to_string(),
            function_tools: vec![
                FunctionTool {
                    name: "get_weather".to_string(),
                    description: "Retrieves weather data for a specified location.".to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "location": {
                                "type": "string",
                                "description": "The location to get weather for"
                            },
                            "unit": {
                                "type": "string",
                                "enum": ["celsius", "fahrenheit"],
                                "description": "Temperature unit"
                            }
                        },
                        "required": ["location"]
                    }),
                    mock_response: r#"{"temperature": 22, "condition": "sunny", "humidity": 65}"#.to_string(),
                }
            ],
            structured_outputs: vec![],
        }
    }
}