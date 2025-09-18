// Gemini API client for WASM
use crate::llm_playground::{Message, ApiConfig, MessageRole};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

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
        if config.gemini.api_key.trim().is_empty() {
            return Err("Gemini API key is required".to_string());
        }

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
                    contents.push(GeminiContent {
                        parts: vec![GeminiPart {
                            text: message.content.clone(),
                        }],
                        role: "user".to_string(),
                    });
                }
                MessageRole::Assistant => {
                    contents.push(GeminiContent {
                        parts: vec![GeminiPart {
                            text: message.content.clone(),
                        }],
                        role: "model".to_string(),
                    });
                }
                MessageRole::Function => {
                    // Skip function messages for now
                }
            }
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

        let response = Request::post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .map_err(|e| format!("Failed to create request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        if !response.ok() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("API request failed: {}", error_text));
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