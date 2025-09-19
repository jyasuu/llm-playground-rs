// OpenAI-compatible API client for WASM
use crate::llm_playground::{Message, ApiConfig, MessageRole};
use crate::llm_playground::api_clients::{LLMClient, ConversationManager, ConversationMessage, FunctionResponse, StreamCallback, FunctionCallRequest, LLMResponse};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use gloo_console::log;
use std::future::Future;
use std::pin::Pin;

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

pub struct OpenAIClient {
    conversation_history: Vec<ConversationMessage>,
    system_prompt: Option<String>,
}

impl OpenAIClient {
    pub fn new() -> Self {
        Self {
            conversation_history: Vec::new(),
            system_prompt: None,
        }
    }

    fn convert_messages_to_openai(&self, messages: &[Message]) -> Vec<OpenAIMessage> {
        let mut openai_messages = Vec::new();

        // Add system message if available
        if let Some(system_prompt) = &self.system_prompt {
            openai_messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: system_prompt.clone(),
            });
        }

        // Add conversation history
        for conv_msg in &self.conversation_history {
            if !conv_msg.content.is_empty() || conv_msg.function_call.is_some() || conv_msg.function_response.is_some() {
                let role = match conv_msg.role.as_str() {
                    "user" => "user",
                    "assistant" | "model" => "assistant",
                    _ => "user",
                };

                openai_messages.push(OpenAIMessage {
                    role: role.to_string(),
                    content: conv_msg.content.clone(),
                });

                // Handle function calls and responses
                if let Some(_fc) = &conv_msg.function_call {
                    // Function calls would be handled differently in a full implementation
                    // For now, we'll include them as text content
                }

                if let Some(fr) = &conv_msg.function_response {
                    openai_messages.push(OpenAIMessage {
                        role: "tool".to_string(),
                        content: serde_json::to_string(fr).unwrap_or_default(),
                    });
                }
            }
        }

        // Add new messages
        for message in messages {
            let role = match message.role {
                MessageRole::System => {
                    if openai_messages.is_empty() || openai_messages[0].role != "system" {
                        "system"
                    } else {
                        continue; // Skip if we already have a system message
                    }
                }
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::Function => "tool",
            };

            openai_messages.push(OpenAIMessage {
                role: role.to_string(),
                content: message.content.clone(),
            });
        }

        openai_messages
    }

    fn build_tools(&self, config: &ApiConfig) -> Option<Vec<serde_json::Value>> {
        if config.function_tools.is_empty() {
            return None;
        }

        Some(
            config
                .function_tools
                .iter()
                .map(|tool| {
                    serde_json::json!({
                        "type": "function",
                        "function": {
                            "name": tool.name,
                            "description": tool.description,
                            "parameters": tool.parameters
                        }
                    })
                })
                .collect(),
        )
    }

    async fn send_message_internal(
        &self,
        messages: &[Message],
        config: &ApiConfig,
    ) -> Result<String, String> {
        log!("OpenAI API call started");
        
        if config.openai.api_key.trim().is_empty() {
            return Err("Please configure your OpenAI API key in Settings".to_string());
        }

        let openai_messages = self.convert_messages_to_openai(messages);
        let tools = self.build_tools(config);

        let mut request_body = serde_json::json!({
            "model": config.openai.model,
            "messages": openai_messages,
            "temperature": config.shared_settings.temperature,
            "max_tokens": config.shared_settings.max_tokens,
        });

        if let Some(tools_array) = tools {
            request_body["tools"] = serde_json::Value::Array(tools_array);
        }

        let url = format!("{}/chat/completions", config.openai.base_url);

        let response = Request::post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", &format!("Bearer {}", config.openai.api_key))
            .json(&request_body)
            .map_err(|e| format!("Failed to create request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Network error - Check your internet connection and API key: {}", e))?;

        if !response.ok() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            
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

impl LLMClient for OpenAIClient {
    fn send_message(
        &self,
        messages: &[Message],
        config: &ApiConfig,
    ) -> Pin<Box<dyn Future<Output = Result<LLMResponse, String>>>> {
        let self_clone = self.clone();
        let messages_clone = messages.to_vec();
        let config_clone = config.clone();

        Box::pin(async move {
            match self_clone.send_message_internal(&messages_clone, &config_clone).await {
                Ok(content) => Ok(LLMResponse {
                    content: Some(content),
                    function_calls: Vec::new(), // TODO: Implement function call parsing for OpenAI
                    finish_reason: Some("stop".to_string()),
                }),
                Err(e) => Err(e),
            }
        })
    }

    fn send_message_stream(
        &self,
        messages: &[Message],
        config: &ApiConfig,
        callback: StreamCallback,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>>>> {
        let openai_messages = self.convert_messages_to_openai(messages);
        let tools = self.build_tools(config);
        let api_key = config.openai.api_key.clone();
        let base_url = config.openai.base_url.clone();
        let model = config.openai.model.clone();
        let temperature = config.shared_settings.temperature;
        let max_tokens = config.shared_settings.max_tokens;

        Box::pin(async move {
            if api_key.trim().is_empty() {
                return Err("Please configure your OpenAI API key in Settings".to_string());
            }

            let mut request_body = serde_json::json!({
                "model": model,
                "messages": openai_messages,
                "temperature": temperature,
                "max_tokens": max_tokens,
                "stream": true,
            });

            if let Some(tools_array) = tools {
                request_body["tools"] = serde_json::Value::Array(tools_array);
            }

            let url = format!("{}/chat/completions", base_url);

            // For WASM, we'll simulate streaming like we did with Gemini
            let response = Request::post(&url)
                .header("Content-Type", "application/json")
                .header("Authorization", &format!("Bearer {}", api_key))
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
            callback(response_text, None);
            
            Ok(())
        })
    }

    fn client_name(&self) -> &str {
        "OpenAI"
    }

    fn get_available_models(
        &self,
        config: &ApiConfig,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, String>>>> {
        let api_key = config.openai.api_key.clone();
        let base_url = config.openai.base_url.clone();

        Box::pin(async move {
            if api_key.trim().is_empty() {
                return Err("Please configure your OpenAI API key to fetch models".to_string());
            }

            let url = format!("{}/models", base_url);

            let response = Request::get(&url)
                .header("Authorization", &format!("Bearer {}", api_key))
                .send()
                .await
                .map_err(|e| format!("Failed to fetch models: {}", e))?;

            if !response.ok() {
                let status = response.status();
                return Err(format!("Failed to fetch models, status: {}", status));
            }

            #[derive(Deserialize)]
            struct ModelsResponse {
                data: Vec<ModelInfo>,
            }

            #[derive(Deserialize)]
            struct ModelInfo {
                id: String,
                object: String,
            }

            let models_response: ModelsResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse models response: {}", e))?;

            // Filter for chat completion models
            let model_names: Vec<String> = models_response
                .data
                .into_iter()
                .filter_map(|model| {
                    if model.object == "model" && (
                        model.id.starts_with("gpt-") ||
                        model.id.starts_with("claude-") ||
                        model.id.contains("chat") ||
                        model.id.contains("instruct")
                    ) {
                        Some(model.id)
                    } else {
                        None
                    }
                })
                .collect();

            Ok(model_names)
        })
    }
}

impl ConversationManager for OpenAIClient {
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
            role: "assistant".to_string(),
            content: message.to_string(),
            function_call,
            function_response: None,
        });
    }

    fn add_function_response(&mut self, function_response: &FunctionResponse) {
        self.conversation_history.push(ConversationMessage {
            role: "tool".to_string(),
            content: serde_json::to_string(&function_response.response).unwrap_or_default(),
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

// We need Clone for the OpenAI client to work with the async trait
impl Clone for OpenAIClient {
    fn clone(&self) -> Self {
        Self {
            conversation_history: self.conversation_history.clone(),
            system_prompt: self.system_prompt.clone(),
        }
    }
}