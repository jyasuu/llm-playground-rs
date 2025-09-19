// Gemini API client for WASM
use crate::llm_playground::{Message, ApiConfig, MessageRole};
use crate::llm_playground::api_clients::{LLMClient, ConversationManager, ConversationMessage, FunctionResponse, StreamCallback};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use gloo_console::log;
use std::future::Future;
use std::pin::Pin;
use wasm_bindgen_futures::spawn_local;
use web_sys::{ReadableStream, Response as WebResponse};
use js_sys::{Uint8Array, Promise};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Debug, Serialize, Deserialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(rename = "generationConfig")]
    generation_config: Option<GenerationConfig>,
    #[serde(rename = "systemInstruction", skip_serializing_if = "Option::is_none")]
    system_instruction: Option<SystemInstruction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Content {
    parts: Vec<Part>,
    role: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Part {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(rename = "functionCall", skip_serializing_if = "Option::is_none")]
    function_call: Option<serde_json::Value>,
    #[serde(rename = "functionResponse", skip_serializing_if = "Option::is_none")]
    function_response: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SystemInstruction {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Tool {
    #[serde(rename = "functionDeclarations")]
    function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FunctionDeclaration {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct GenerationConfig {
    temperature: f32,
    #[serde(rename = "topP")]
    top_p: f32,
    #[serde(rename = "topK")]
    top_k: i32,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
    #[serde(rename = "usageMetadata", skip_serializing_if = "Option::is_none")]
    usage_metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Candidate {
    content: Content,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
}

pub struct GeminiClient {
    conversation_history: Vec<ConversationMessage>,
    system_prompt: Option<String>,
}

impl GeminiClient {
    pub fn new() -> Self {
        Self {
            conversation_history: Vec::new(),
            system_prompt: None,
        }
    }

    fn convert_messages_to_contents(&self, messages: &[Message]) -> (Vec<Content>, Option<SystemInstruction>) {
        let mut contents = Vec::new();
        let mut system_instruction = None;

        // Use system prompt if available
        if let Some(system_prompt) = &self.system_prompt {
            system_instruction = Some(SystemInstruction {
                parts: vec![Part {
                    text: Some(system_prompt.clone()),
                    function_call: None,
                    function_response: None,
                }],
            });
        }

        // Add conversation history
        for conv_msg in &self.conversation_history {
            let mut parts = Vec::new();
            
            if !conv_msg.content.is_empty() {
                parts.push(Part {
                    text: Some(conv_msg.content.clone()),
                    function_call: None,
                    function_response: None,
                });
            }
            
            if let Some(fc) = &conv_msg.function_call {
                parts.push(Part {
                    text: None,
                    function_call: Some(fc.clone()),
                    function_response: None,
                });
            }
            
            if let Some(fr) = &conv_msg.function_response {
                parts.push(Part {
                    text: None,
                    function_call: None,
                    function_response: Some(fr.clone()),
                });
            }

            if !parts.is_empty() {
                contents.push(Content {
                    parts,
                    role: conv_msg.role.clone(),
                });
            }
        }

        // Add new messages
        for message in messages {
            match message.role {
                MessageRole::System => {
                    if system_instruction.is_none() {
                        system_instruction = Some(SystemInstruction {
                            parts: vec![Part {
                                text: Some(message.content.clone()),
                                function_call: None,
                                function_response: None,
                            }],
                        });
                    }
                }
                MessageRole::User => {
                    if !message.content.trim().is_empty() {
                        let mut parts = vec![Part {
                            text: Some(message.content.clone()),
                            function_call: None,
                            function_response: None,
                        }];
                        
                        if let Some(fc) = &message.function_call {
                            parts.push(Part {
                                text: None,
                                function_call: Some(fc.clone()),
                                function_response: None,
                            });
                        }
                        
                        if let Some(fr) = &message.function_response {
                            parts.push(Part {
                                text: None,
                                function_call: None,
                                function_response: Some(fr.clone()),
                            });
                        }

                        contents.push(Content {
                            parts,
                            role: "user".to_string(),
                        });
                    }
                }
                MessageRole::Assistant => {
                    if !message.content.trim().is_empty() {
                        let mut parts = vec![Part {
                            text: Some(message.content.clone()),
                            function_call: None,
                            function_response: None,
                        }];
                        
                        if let Some(fc) = &message.function_call {
                            parts.push(Part {
                                text: None,
                                function_call: Some(fc.clone()),
                                function_response: None,
                            });
                        }

                        contents.push(Content {
                            parts,
                            role: "model".to_string(),
                        });
                    }
                }
                MessageRole::Function => {
                    // Handle function messages as user messages with function responses
                    if let Some(fr) = &message.function_response {
                        contents.push(Content {
                            parts: vec![Part {
                                text: None,
                                function_call: None,
                                function_response: Some(fr.clone()),
                            }],
                            role: "user".to_string(),
                        });
                    }
                }
            }
        }

        (contents, system_instruction)
    }

    fn build_tools(&self, config: &ApiConfig) -> Option<Vec<Tool>> {
        if config.function_tools.is_empty() {
            return None;
        }

        Some(vec![Tool {
            function_declarations: config
                .function_tools
                .iter()
                .map(|tool| FunctionDeclaration {
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                    parameters: tool.parameters.clone(),
                })
                .collect(),
        }])
    }
}

impl LLMClient for GeminiClient {
    fn send_message(
        &self,
        messages: &[Message],
        config: &ApiConfig,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>>>> {
        
        let (contents, system_instruction) = self.convert_messages_to_contents(messages);
        let tools = self.build_tools(config);
        let api_key = config.gemini.api_key.clone();
        let model = config.gemini.model.clone();
        let temperature = config.shared_settings.temperature;
        let max_tokens = config.shared_settings.max_tokens;
        let config_clone = config.clone();

        Box::pin(async move {
            log!("Gemini API call started");
            
            if api_key.trim().is_empty() {
                return Err("Please configure your Gemini API key in Settings".to_string());
            }

            let request_body = GeminiRequest {
                contents,
                generation_config: Some(GenerationConfig {
                    temperature,
                    top_p: 0.95,
                    top_k: 40,
                    max_output_tokens: max_tokens as i32,
                }),
                system_instruction,
                tools,
            };

            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                model, api_key
            );

            let response = Request::post(&url)
                .header("Content-Type", "application/json")
                .json(&request_body)
                .map_err(|e| format!("Failed to create request: {}", e))?
                .send()
                .await
                .map_err(|e| format!("Network error - Check your internet connection and API key: {}", e))?;

            if !response.ok() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                
                let error_message = if status == 400 {
                    if error_text.contains("API_KEY_INVALID") {
                        "Invalid Gemini API key. Please check your API key in Settings."
                    } else if error_text.contains("quota") || error_text.contains("QUOTA") {
                        "API quota exceeded. Please check your Gemini API usage limits."
                    } else {
                        "Bad request to Gemini API. Please check your configuration."
                    }
                } else if status == 403 {
                    "Access denied. Please verify your Gemini API key has proper permissions."
                } else if status == 429 {
                    "Rate limit exceeded. Please wait a moment before trying again."
                } else {
                    "Gemini API error occurred. Please try again."
                };
                
                return Err(format!("{}\n\nDetailed error: {}", error_message, error_text));
            }

            let gemini_response: GeminiResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            if gemini_response.candidates.is_empty() {
                return Err("No response from Gemini API".to_string());
            }

            let candidate = &gemini_response.candidates[0];
            if candidate.content.parts.is_empty() {
                return Err("Empty response from Gemini API".to_string());
            }

            // Check for text content first
            for part in &candidate.content.parts {
                if let Some(text) = &part.text {
                    return Ok(text.clone());
                }
            }

            // Check for function calls if no text content
            for part in &candidate.content.parts {
                if let Some(function_call) = &part.function_call {
                    
                    // Extract function name and arguments
                    if let (Some(name), Some(args)) = (
                        function_call.get("name").and_then(|v| v.as_str()),
                        function_call.get("args")
                    ) {
                        // Find the mock response from config
                        let mock_response = config_clone
                            .function_tools
                            .iter()
                            .find(|tool| tool.name == name)
                            .map(|tool| tool.mock_response.clone())
                            .unwrap_or_else(|| r#"{"result": "Function executed successfully"}"#.to_string());
                        
                        // Format the function call display
                        let function_display = format!(
                            "🔧 **Function Call**: `{}`\n\n**Arguments**: ```json\n{}\n```\n\n**Response**: ```json\n{}\n```",
                            name,
                            serde_json::to_string_pretty(args).unwrap_or_else(|_| args.to_string()),
                            mock_response
                        );
                        
                        log!("Returning function call display");
                        return Ok(function_display);
                    }
                }
            }

            Err("No text or function call content in response".to_string())
        })
    }

    fn send_message_stream(
        &self,
        messages: &[Message],
        config: &ApiConfig,
        callback: StreamCallback,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>>>> {
        let (contents, system_instruction) = self.convert_messages_to_contents(messages);
        let tools = self.build_tools(config);
        let api_key = config.gemini.api_key.clone();
        let model = config.gemini.model.clone();
        let temperature = config.shared_settings.temperature;
        let max_tokens = config.shared_settings.max_tokens;
        let config_clone = config.clone();

        Box::pin(async move {
            log!("Gemini streaming API call started");
            
            if api_key.trim().is_empty() {
                return Err("Please configure your Gemini API key in Settings".to_string());
            }

            let request_body = GeminiRequest {
                contents,
                generation_config: Some(GenerationConfig {
                    temperature,
                    top_p: 0.95,
                    top_k: 40,
                    max_output_tokens: max_tokens as i32,
                }),
                system_instruction,
                tools,
            };

            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
                model, api_key
            );

            // For WASM, we'll use a simpler approach since we can't do proper SSE streaming
            // We'll make a regular request and simulate streaming by sending the response in chunks
            let response = Request::post(&url)
                .header("Content-Type", "application/json")
                .json(&request_body)
                .map_err(|e| format!("Failed to create request: {}", e))?
                .send()
                .await
                .map_err(|e| format!("Network error: {}", e))?;

            if !response.ok() {
                let status = response.status();
                let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                return Err(format!("API error {}: {}", status, error_text));
            }

            let response_text = response.text().await.map_err(|e| format!("Failed to read response: {}", e))?;
            
            // For now, send the full response as a single chunk
            // In a real implementation, we'd parse SSE events
            callback(response_text, None);
            
            Ok(())
        })
    }

    fn client_name(&self) -> &str {
        "Gemini"
    }
}

impl ConversationManager for GeminiClient {
    fn add_user_message(&mut self, message: &str) {
        self.conversation_history.push(ConversationMessage {
            role: "user".to_string(),
            content: message.to_string(),
            function_call: None,
            function_response: None,
        });
    }

    fn add_assistant_message(&mut self, message: &str, function_call: Option<serde_json::Value>) {
        self.conversation_history.push(ConversationMessage {
            role: "model".to_string(),
            content: message.to_string(),
            function_call,
            function_response: None,
        });
    }

    fn add_function_response(&mut self, function_response: &FunctionResponse) {
        self.conversation_history.push(ConversationMessage {
            role: "user".to_string(),
            content: String::new(),
            function_call: None,
            function_response: Some(serde_json::json!({
                "id": function_response.id,
                "name": function_response.name,
                "response": function_response.response
            })),
        });
    }

    fn clear_conversation(&mut self) {
        self.conversation_history.clear();
    }

    fn set_system_prompt(&mut self, prompt: &str) {
        self.system_prompt = Some(prompt.to_string());
    }

    fn get_conversation_history(&self) -> &[ConversationMessage] {
        &self.conversation_history
    }
}