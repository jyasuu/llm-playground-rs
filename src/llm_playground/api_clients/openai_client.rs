// OpenAI-compatible API client for WASM
use crate::llm_playground::{Message, ApiConfig, MessageRole};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Choice {
    message: OpenAIMessage,
}

pub struct OpenAIClient;

impl OpenAIClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn send_message(
        &self,
        messages: &[Message],
        config: &ApiConfig,
    ) -> Result<String, String> {
        if config.openai.api_key.trim().is_empty() {
            return Err("OpenAI API key is required".to_string());
        }

        // Convert messages to OpenAI format
        let openai_messages: Vec<OpenAIMessage> = messages
            .iter()
            .map(|msg| OpenAIMessage {
                role: match msg.role {
                    MessageRole::System => "system".to_string(),
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    MessageRole::Function => "user".to_string(), // Treat function as user for now
                },
                content: msg.content.clone(),
            })
            .collect();

        let request_body = OpenAIRequest {
            model: config.openai.model.clone(),
            messages: openai_messages,
            temperature: config.shared_settings.temperature,
            max_tokens: config.shared_settings.max_tokens,
        };

        let url = format!("{}/chat/completions", config.openai.base_url);

        let response = Request::post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", config.openai.api_key))
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

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if openai_response.choices.is_empty() {
            return Err("No response from OpenAI API".to_string());
        }

        Ok(openai_response.choices[0].message.content.clone())
    }
}