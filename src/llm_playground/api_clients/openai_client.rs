// OpenAI-compatible API client for WASM
use crate::llm_playground::{Message, ApiConfig, MessageRole};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use gloo_console::log;

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
        log!("OpenAI API call started");
        
        if config.openai.api_key.trim().is_empty() {
            log!("OpenAI API key is missing");
            return Err("Please configure your OpenAI API key in Settings".to_string());
        }
        
        log!("API key present, processing messages...");

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
        
        log!("Making request to:", &url);
        log!("Request body:", &serde_json::to_string(&request_body).unwrap_or_default());

        let response = Request::post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", config.openai.api_key))
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
                "Bad request to OpenAI API. Please check your model selection and message format."
            } else if status == 401 {
                "Invalid OpenAI API key. Please check your API key in Settings."
            } else if status == 403 {
                "Access denied. Your API key may not have permission for this model."
            } else if status == 429 {
                "Rate limit exceeded. Please wait a moment before trying again."
            } else if status == 500 {
                "OpenAI server error. Please try again in a moment."
            } else {
                "OpenAI API error occurred. Please try again."
            };
            
            return Err(format!("{}\n\nDetailed error: {}", error_message, error_text));
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