// Refactored OpenAI client following SOLID principles
use super::interfaces::{FunctionCaller, MessageSender, ModelProvider, NamedClient, StreamingSender};
use super::traits::{FunctionCallRequest, LLMResponse, StreamCallback, UnifiedMessage, UnifiedMessageRole};
use crate::llm_playground::ApiConfig;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

// Keep the existing request/response structures
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OpenAIMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Choice {
    message: OpenAIMessage,
}

/// Refactored OpenAI client with single responsibilities
pub struct RefactoredOpenAIClient {
    // Client configuration could be injected here
}

impl RefactoredOpenAIClient {
    pub fn new() -> Self {
        Self {}
    }

    // Helper method for message conversion (SRP - single responsibility)
    fn convert_unified_to_openai(&self, messages: &[UnifiedMessage], system_prompt: Option<&str>) -> Vec<OpenAIMessage> {
        let mut openai_messages = Vec::new();

        // Add system message if provided
        if let Some(prompt) = system_prompt {
            openai_messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: Some(prompt.to_string()),
                name: None,
                tool_calls: None,
                tool_call_id: None,
            });
        }

        // Convert unified messages
        for message in messages {
            let role = match message.role {
                UnifiedMessageRole::System => "system",
                UnifiedMessageRole::User => "user",
                UnifiedMessageRole::Assistant => "assistant",
            };

            let tool_calls = if !message.function_calls.is_empty() {
                Some(
                    message
                        .function_calls
                        .iter()
                        .map(|fc| ToolCall {
                            id: fc.id.clone(),
                            call_type: "function".to_string(),
                            function: FunctionCall {
                                name: fc.name.clone(),
                                arguments: serde_json::to_string(&fc.arguments).unwrap_or_default(),
                            },
                        })
                        .collect(),
                )
            } else {
                None
            };

            openai_messages.push(OpenAIMessage {
                role: role.to_string(),
                content: message.content.clone(),
                name: None,
                tool_calls,
                tool_call_id: None,
            });

            // Add function responses as separate messages
            for response in &message.function_responses {
                openai_messages.push(OpenAIMessage {
                    role: "tool".to_string(),
                    content: Some(serde_json::to_string(&response.response).unwrap_or_default()),
                    name: Some(response.name.clone()),
                    tool_calls: None,
                    tool_call_id: Some(response.id.clone()),
                });
            }
        }

        openai_messages
    }

    // Helper method for API request (SRP)
    async fn make_request(&self, request: OpenAIRequest, config: &ApiConfig) -> Result<OpenAIResponse, String> {
        let api_key = &config.openai.api_key;
        let base_url = if config.openai.base_url.is_empty() {
            "https://api.openai.com/v1".to_string()
        } else {
            config.openai.base_url.clone()
        };

        let url = format!("{}/chat/completions", base_url);

        let response = Request::post(&url)
            .header("Authorization", &format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.ok() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("API error {}: {}", status, error_text));
        }

        response
            .json::<OpenAIResponse>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }
}

// Implement segregated interfaces (ISP compliance)

impl MessageSender for RefactoredOpenAIClient {
    fn send_message<'a>(
        &'a self,
        messages: &'a [UnifiedMessage],
        config: &'a ApiConfig,
        system_prompt: Option<&'a str>,
    ) -> Pin<Box<dyn Future<Output = Result<LLMResponse, String>> + 'a>> {
        Box::pin(async move {
            let openai_messages = self.convert_unified_to_openai(messages, system_prompt);
            
            let request = OpenAIRequest {
                model: config.openai.model.clone(),
                messages: openai_messages,
                temperature: config.shared_settings.temperature,
                max_tokens: config.shared_settings.max_tokens,
                tools: if !config.function_tools.is_empty() {
                    Some(config.function_tools.iter().map(|tool| serde_json::json!(tool)).collect())
                } else {
                    None
                },
                tool_choice: None,
            };

            let response = self.make_request(request, config).await?;
            
            if let Some(choice) = response.choices.first() {
                let function_calls = choice.message.tool_calls
                    .as_ref()
                    .map(|calls| {
                        calls
                            .iter()
                            .map(|call| FunctionCallRequest {
                                id: call.id.clone(),
                                name: call.function.name.clone(),
                                arguments: serde_json::from_str(&call.function.arguments)
                                    .unwrap_or(serde_json::Value::Null),
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                Ok(LLMResponse {
                    content: choice.message.content.clone(),
                    function_calls,
                    finish_reason: None,
                })
            } else {
                Err("No response from API".to_string())
            }
        })
    }
}

impl NamedClient for RefactoredOpenAIClient {
    fn client_name(&self) -> &str {
        "OpenAI"
    }
}

impl FunctionCaller for RefactoredOpenAIClient {
    fn supports_function_calling(&self) -> bool {
        true
    }

    fn prepare_function_tools(&self, tools: &[serde_json::Value]) -> Result<serde_json::Value, String> {
        Ok(serde_json::json!(tools))
    }
}

impl ModelProvider for RefactoredOpenAIClient {
    fn get_available_models<'a>(
        &'a self,
        _config: &'a ApiConfig,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, String>> + 'a>> {
        Box::pin(async move {
            // Simplified implementation - in reality, you'd fetch from API
            Ok(vec![
                "gpt-4".to_string(),
                "gpt-3.5-turbo".to_string(),
                "gpt-4-turbo".to_string(),
            ])
        })
    }
}

impl StreamingSender for RefactoredOpenAIClient {
    fn send_message_stream<'a>(
        &'a self,
        messages: &'a [UnifiedMessage],
        config: &'a ApiConfig,
        system_prompt: Option<&'a str>,
        callback: StreamCallback,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + 'a>> {
        Box::pin(async move {
            // Simplified streaming implementation
            // In a real implementation, you'd handle SSE streaming
            let response = self.send_message(messages, config, system_prompt).await?;
            
            if let Some(content) = response.content {
                callback(content, None);
            }
            
            Ok(())
        })
    }
}

impl Default for RefactoredOpenAIClient {
    fn default() -> Self {
        Self::new()
    }
}