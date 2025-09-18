// Gemini API client for WASM
use crate::llm_playground::{Message, ApiConfig, MessageRole};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use gloo_console::log;

#[derive(Debug, Serialize, Deserialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig")]
    generation_config: GenerationConfig,
    #[serde(rename = "systemInstruction", skip_serializing_if = "Option::is_none")]
    system_instruction: Option<SystemInstruction>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
    role: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct SystemInstruction {
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GenerationConfig {
    temperature: f32,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Candidate {
    content: GeminiContent,
}

pub struct GeminiClient;

impl GeminiClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn send_message(
        &self,
        messages: &[Message],
        config: &ApiConfig,
    ) -> Result<String, String> {
        log!("Gemini API call started");
        
        if config.gemini.api_key.trim().is_empty() {
            log!("Gemini API key is missing");
            return Err("Please configure your Gemini API key in Settings".to_string());
        }
        
        log!("API key present, processing messages...");

        // Convert messages to Gemini format
        let mut contents = Vec::new();
        let mut system_instruction = None;

        for message in messages {
            match message.role {
                MessageRole::System => {
                    system_instruction = Some(SystemInstruction {
                        parts: vec![GeminiPart {
                            text: message.content.clone(),
                        }],
                    });
                }
                MessageRole::User => {
                    if !message.content.trim().is_empty() {
                        contents.push(GeminiContent {
                            parts: vec![GeminiPart {
                                text: message.content.clone(),
                            }],
                            role: "user".to_string(),
                        });
                    }
                }
                MessageRole::Assistant => {
                    if !message.content.trim().is_empty() {
                        contents.push(GeminiContent {
                            parts: vec![GeminiPart {
                                text: message.content.clone(),
                            }],
                            role: "model".to_string(),
                        });
                    }
                }
                MessageRole::Function => {
                    // Skip function messages for now
                }
            }
        }

        // Ensure we have at least one content item and conversation ends with user message
        if contents.is_empty() {
            log!("No valid contents found, creating default user message");
            contents.push(GeminiContent {
                parts: vec![GeminiPart {
                    text: "Hello".to_string(),
                }],
                role: "user".to_string(),
            });
        } else {
            // Gemini API requires conversations to end with a user message
            // If the last message is from the model, we need to ensure there's a user message
            if let Some(last_content) = contents.last() {
                if last_content.role == "model" {
                    log!("Last message is from model, this should not happen in proper flow");
                    // This shouldn't happen in normal flow since we're adding user message first
                    // But let's handle it gracefully
                }
            }
        }
        
        log!("Final contents count:", contents.len());
        if let Some(last) = contents.last() {
            log!("Last message role:", &last.role);
        }

        let request_body = GeminiRequest {
            contents,
            generation_config: GenerationConfig {
                temperature: config.shared_settings.temperature,
                max_output_tokens: config.shared_settings.max_tokens,
            },
            system_instruction,
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            config.gemini.model, config.gemini.api_key
        );
        
        log!("Making request to:", &url);
        log!("Request body:", &serde_json::to_string(&request_body).unwrap_or_default());

        let response = Request::post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .map_err(|e| format!("Failed to create request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error - Check your internet connection and API key: {}", e))?;
            
        log!("Response status:", response.status());

        if !response.ok() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            
            log!("API error response:", &error_text);
            
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

        Ok(candidate.content.parts[0].text.clone())
    }
}