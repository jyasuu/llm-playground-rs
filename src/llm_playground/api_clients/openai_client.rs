// OpenAI-compatible API client for WASM
use crate::llm_playground::api_clients::{
    ConversationManager, ConversationMessage, FunctionCallRequest, FunctionResponse, LLMClient,
    LLMResponse, StreamCallback, UnifiedMessage, UnifiedMessageRole,
};
use crate::llm_playground::{ApiConfig, Message, MessageRole};
use gloo_console::log;
use gloo_net::http::Request;
use js_sys::Promise;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use wasm_bindgen_futures::JsFuture;
use web_sys;

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

    // Helper function to sleep for a specified duration
    async fn sleep(ms: i32) {
        let promise = Promise::new(&mut |resolve, _| {
            let window = web_sys::window().unwrap();
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
        });
        let _ = JsFuture::from(promise).await;
    }

    fn convert_unified_messages_to_openai(&self, messages: &[UnifiedMessage], system_prompt: Option<&str>) -> Vec<OpenAIMessage> {
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

        // Add conversation history
        for conv_msg in &self.conversation_history {
            log!("conv_msg_json");
            let conv_msg_json = serde_json::json!(conv_msg.clone());
            log!(conv_msg_json.to_string());

            if !conv_msg.content.is_empty()
                || conv_msg.function_call.is_some()
                || conv_msg.function_response.is_some()
            {
                let role = match conv_msg.role.as_str() {
                    "user" => "user",
                    "assistant" | "model" => "assistant",
                    "tool" => "tool",
                    _ => "user",
                };

                let mut openai_msg = OpenAIMessage {
                    role: role.to_string(),
                    content: if conv_msg.content.is_empty() && conv_msg.function_call.is_some() {
                        None
                    } else {
                        Some(conv_msg.content.clone())
                    },
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                };

                // Handle function calls from assistant messages
                if let Some(fc) = &conv_msg.function_call {
                    if role == "assistant" {
                        // Convert function call JSON to proper tool_calls format
                        if let Ok(func_calls) =
                            serde_json::from_value::<Vec<serde_json::Value>>(fc.clone())
                        {
                            let mut tool_calls = Vec::new();
                            for func_call in func_calls {
                                if let (Some(id), Some(name), Some(args)) = (
                                    func_call.get("id").and_then(|v| v.as_str()),
                                    func_call.get("name").and_then(|v| v.as_str()),
                                    func_call.get("arguments"),
                                ) {
                                    tool_calls.push(ToolCall {
                                        id: id.to_string(),
                                        call_type: "function".to_string(),
                                        function: FunctionCall {
                                            name: name.to_string(),
                                            arguments: serde_json::to_string(args)
                                                .unwrap_or_default(),
                                        },
                                    });
                                }
                            }
                            if !tool_calls.is_empty() {
                                openai_msg.tool_calls = Some(tool_calls);
                            }
                        }
                    }
                }

                // Handle function responses
                if let Some(fr) = &conv_msg.function_response {
                    if role == "tool" {
                        if let Some(call_id) = fr.get("id").and_then(|v| v.as_str()) {
                            openai_msg.tool_call_id = Some(call_id.to_string());
                        }
                        if let Some(func_name) = fr.get("name").and_then(|v| v.as_str()) {
                            openai_msg.name = Some(func_name.to_string());
                        }
                        if let Some(response_data) = fr.get("response") {
                            openai_msg.content =
                                Some(serde_json::to_string(response_data).unwrap_or_default());
                        }
                    }
                }

                log!("openai_msg_json");
                let openai_msg_json = serde_json::json!(openai_msg.clone());
                log!(openai_msg_json.to_string());

                openai_messages.push(openai_msg);
            }
        }

        // Add new unified messages
        for message in messages {
            log!("unified_message_json");
            let message_json = serde_json::json!(message.clone());
            log!(message_json.to_string());

            let role = match message.role {
                UnifiedMessageRole::System => {
                    if openai_messages.is_empty() || openai_messages[0].role != "system" {
                        "system"
                    } else {
                        continue; // Skip if we already have a system message
                    }
                }
                UnifiedMessageRole::User => "user",
                UnifiedMessageRole::Assistant => "assistant",
            };

            // Handle assistant messages with function calls
            if message.role == UnifiedMessageRole::Assistant && !message.function_calls.is_empty() {
                let mut openai_msg = OpenAIMessage {
                    role: "assistant".to_string(),
                    content: message.content.clone(),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                };

                // Convert function calls to OpenAI format
                let mut tool_calls = Vec::new();
                for func_call in &message.function_calls {
                    tool_calls.push(ToolCall {
                        id: func_call.id.clone(),
                        call_type: "function".to_string(),
                        function: FunctionCall {
                            name: func_call.name.clone(),
                            arguments: serde_json::to_string(&func_call.arguments).unwrap_or_default(),
                        },
                    });
                }
                if !tool_calls.is_empty() {
                    openai_msg.tool_calls = Some(tool_calls);
                }

                openai_messages.push(openai_msg);
            } else {
                // Regular message
                let mut openai_msg = OpenAIMessage {
                    role: role.to_string(),
                    content: message.content.clone(),
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                };

                openai_messages.push(openai_msg);
            }

            // Add function response messages as separate tool messages
            for func_response in &message.function_responses {
                let tool_msg = OpenAIMessage {
                    role: "tool".to_string(),
                    content: Some(serde_json::to_string(&func_response.response).unwrap_or_default()),
                    name: Some(func_response.name.clone()),
                    tool_calls: None,
                    tool_call_id: Some(func_response.id.clone()),
                };
                openai_messages.push(tool_msg);
            }
        }

        openai_messages
    }

    fn build_tools(&self, config: &ApiConfig) -> Option<Vec<serde_json::Value>> {
        let enabled_tools = config.get_enabled_function_tools();
        if enabled_tools.is_empty() {
            return None;
        }

        Some(
            enabled_tools
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
        messages: &[UnifiedMessage],
        config: &ApiConfig,
        system_prompt: Option<&str>,
    ) -> Result<String, String> {
        log!("OpenAI API call started");

        if config.openai.api_key.trim().is_empty() {
            return Err("Please configure your OpenAI API key in Settings".to_string());
        }

        let openai_messages = self.convert_unified_messages_to_openai(messages, system_prompt);
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
            .header(
                "Authorization",
                &format!("Bearer {}", config.openai.api_key),
            )
            .json(&request_body)
            .map_err(|e| format!("Failed to create request: {}", e))?
            .send()
            .await
            .map_err(|e| {
                format!(
                    "Network error - Check your internet connection and API key: {}",
                    e
                )
            })?;

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

            return Err(format!(
                "{}\n\nDetailed error: {}",
                error_message, error_text
            ));
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        if openai_response.choices.is_empty() {
            return Err("No response from OpenAI API".to_string());
        }

        Ok(openai_response.choices[0]
            .message
            .content
            .clone()
            .unwrap_or_default())
    }
}

impl LLMClient for OpenAIClient {
    fn send_message(
        &self,
        messages: &[UnifiedMessage],
        config: &ApiConfig,
        system_prompt: Option<&str>,
    ) -> Pin<Box<dyn Future<Output = Result<LLMResponse, String>>>> {
        let self_clone = self.clone();
        let messages_clone = messages.to_vec();
        let config_clone = config.clone();
        let system_prompt_clone = system_prompt.map(|s| s.to_string());

        log!("unified_messages_json");
        let messages_json = serde_json::json!(messages_clone.clone());
        log!(messages_json.to_string());

        Box::pin(async move {
            // Use the full response parsing instead of just the internal method
            if config_clone.openai.api_key.trim().is_empty() {
                return Err("Please configure your OpenAI API key in Settings".to_string());
            }

            let openai_messages = self_clone.convert_unified_messages_to_openai(&messages_clone, system_prompt_clone.as_deref());

            log!("openai_messages_json");
            let openai_messages_json = serde_json::json!(openai_messages.clone());
            log!(openai_messages_json.to_string());

            let tools = self_clone.build_tools(&config_clone);

            let request_body = OpenAIRequest {
                model: config_clone.openai.model,
                messages: openai_messages,
                temperature: config_clone.shared_settings.temperature,
                max_tokens: config_clone.shared_settings.max_tokens,
                tools: tools.clone(),
                tool_choice: if tools.is_some() {
                    Some("auto".to_string())
                } else {
                    None
                },
            };

            let url = format!("{}/chat/completions", config_clone.openai.base_url);

            // Add sleep/delay before sending the request (500ms)
            log!("Sleeping for 500ms before sending OpenAI request...");
            OpenAIClient::sleep(500).await;
            log!("Sleep completed, sending request now");

            let response = Request::post(&url)
                .header("Content-Type", "application/json")
                .header(
                    "Authorization",
                    &format!("Bearer {}", config_clone.openai.api_key),
                )
                .json(&request_body)
                .map_err(|e| format!("Failed to create request: {}", e))?
                .send()
                .await
                .map_err(|e| {
                    format!(
                        "Network error - Check your internet connection and API key: {}",
                        e
                    )
                })?;

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

                return Err(format!(
                    "{}\n\nDetailed error: {}",
                    error_message, error_text
                ));
            }

            let openai_response: OpenAIResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            log!("response_json");
            let response_json = serde_json::json!(openai_response.clone());
            log!(response_json.to_string());

            if openai_response.choices.is_empty() {
                return Err("No response from OpenAI API".to_string());
            }

            let choice = &openai_response.choices[0];
            let message = &choice.message;

            // Extract content
            let content = message.content.clone();

            // Extract function calls
            let mut function_calls = Vec::new();
            if let Some(tool_calls) = &message.tool_calls {
                for tool_call in tool_calls {
                    // Parse the arguments JSON string
                    let args = if tool_call.function.arguments.is_empty() {
                        serde_json::json!({})
                    } else {
                        match serde_json::from_str::<serde_json::Value>(
                            &tool_call.function.arguments,
                        ) {
                            Ok(parsed) => parsed,
                            Err(_) => serde_json::json!({}),
                        }
                    };

                    function_calls.push(FunctionCallRequest {
                        id: tool_call.id.clone(),
                        name: tool_call.function.name.clone(),
                        arguments: args,
                    });
                }
            }

            Ok(LLMResponse {
                content,
                function_calls,
                finish_reason: Some("stop".to_string()),
            })
        })
    }

    fn send_message_stream(
        &self,
        messages: &[UnifiedMessage],
        config: &ApiConfig,
        system_prompt: Option<&str>,
        callback: StreamCallback,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>>>> {
        let openai_messages = self.convert_unified_messages_to_openai(messages, system_prompt);
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
                request_body["tool_choice"] = serde_json::Value::String("auto".to_string());
            }

            let url = format!("{}/chat/completions", base_url);

            // Add sleep/delay before sending the streaming request (500ms)
            log!("Sleeping for 500ms before sending OpenAI streaming request...");
            OpenAIClient::sleep(500).await;
            log!("Sleep completed, sending streaming request now");

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
                let error_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                return Err(format!("API error {}: {}", status, error_text));
            }

            let response_text = response
                .text()
                .await
                .map_err(|e| format!("Failed to read response: {}", e))?;

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
                    if model.object == "model"
                        && (model.id.starts_with("gpt-")
                            || model.id.starts_with("claude-")
                            || model.id.contains("chat")
                            || model.id.contains("instruct"))
                    {
                        Some(model.id)
                    } else {
                        None
                    }
                })
                .collect();

            Ok(model_names)
        })
    }

    fn convert_legacy_messages(&self, messages: &[Message]) -> Vec<UnifiedMessage> {
        let mut unified_messages = Vec::new();
        let mut function_call_id_counter = 0u32;

        for message in messages {
            let role = match message.role {
                MessageRole::System => UnifiedMessageRole::System,
                MessageRole::User => UnifiedMessageRole::User,
                MessageRole::Assistant => UnifiedMessageRole::Assistant,
                MessageRole::Function => UnifiedMessageRole::User, // Function responses become user messages
            };

            let mut function_calls = Vec::new();
            let mut function_responses = Vec::new();

            // Convert legacy function calls
            if let Some(fc) = &message.function_call {
                if let Ok(func_calls) = serde_json::from_value::<Vec<serde_json::Value>>(fc.clone()) {
                    for func_call in func_calls {
                        if let (Some(name), Some(args)) = (
                            func_call.get("name").and_then(|v| v.as_str()),
                            func_call.get("arguments"),
                        ) {
                            let id = func_call
                                .get("id")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| {
                                    function_call_id_counter += 1;
                                    format!("call_{}", function_call_id_counter)
                                });

                            function_calls.push(FunctionCallRequest {
                                id,
                                name: name.to_string(),
                                arguments: args.clone(),
                            });
                        }
                    }
                }
            }

            // Convert legacy function responses
            if let Some(fr) = &message.function_response {
                if let (Some(id), Some(name), Some(response)) = (
                    fr.get("id").and_then(|v| v.as_str()),
                    fr.get("name").and_then(|v| v.as_str()),
                    fr.get("response"),
                ) {
                    function_responses.push(FunctionResponse {
                        id: id.to_string(),
                        name: name.to_string(),
                        response: response.clone(),
                    });
                }
            }

            unified_messages.push(UnifiedMessage {
                id: message.id.clone(),
                role,
                content: if message.content.is_empty() { None } else { Some(message.content.clone()) },
                timestamp: message.timestamp,
                function_calls,
                function_responses,
            });
        }

        unified_messages
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm_playground::{ApiConfig, FunctionTool, Message, MessageRole, OpenAIConfig};
    use serde_json::json;

    // Helper to create a default config for tests
    fn create_test_config() -> ApiConfig {
        ApiConfig {
            openai: OpenAIConfig {
                base_url: "https://api.openai.com/v1".to_string(),
                api_key: "test_key".to_string(),
                model: "gpt-4".to_string(),
            },
            ..Default::default()
        }
    }

    fn create_test_message(role: MessageRole, content: &str) -> Message {
        Message {
            id: "test_id".to_string(),
            role,
            content: content.to_string(),
            timestamp: 0.0,
            function_call: None,
            function_response: None,
        }
    }

    #[test]
    fn test_new_openai_client() {
        let client = OpenAIClient::new();
        assert!(client.conversation_history.is_empty());
        assert!(client.system_prompt.is_none());
    }

    #[test]
    fn test_set_system_prompt() {
        let mut client = OpenAIClient::new();
        let prompt = "You are a helpful assistant.";
        client.set_system_prompt(prompt);
        assert_eq!(client.system_prompt, Some(prompt.to_string()));
    }

    #[test]
    fn test_add_user_message() {
        let mut client = OpenAIClient::new();
        client.add_user_message("Hello");
        assert_eq!(client.conversation_history.len(), 1);
        assert_eq!(client.conversation_history[0].role, "user");
        assert_eq!(client.conversation_history[0].content, "Hello");
    }

    #[test]
    fn test_add_assistant_message() {
        let mut client = OpenAIClient::new();
        client.add_assistant_message("Hi there!", None);
        assert_eq!(client.conversation_history.len(), 1);
        assert_eq!(client.conversation_history[0].role, "assistant");
        assert_eq!(client.conversation_history[0].content, "Hi there!");
    }

    #[test]
    fn test_clear_conversation() {
        let mut client = OpenAIClient::new();
        client.add_user_message("Hello");
        client.clear_conversation();
        assert!(client.conversation_history.is_empty());
    }

    #[test]
    fn test_convert_messages_to_openai_simple() {
        let client = OpenAIClient::new();
        let messages = vec![create_test_message(MessageRole::User, "Hello")];
        let openai_messages = client.convert_messages_to_openai(&messages);

        assert_eq!(openai_messages.len(), 1);
        assert_eq!(openai_messages[0].role, "user");
        assert_eq!(openai_messages[0].content, Some("Hello".to_string()));
    }

    #[test]
    fn test_convert_messages_with_system_prompt_from_client() {
        let mut client = OpenAIClient::new();
        client.set_system_prompt("Be concise.");
        let messages = vec![create_test_message(MessageRole::User, "Hello")];
        let openai_messages = client.convert_messages_to_openai(&messages);

        assert_eq!(openai_messages.len(), 2);
        assert_eq!(openai_messages[0].role, "system");
        assert_eq!(openai_messages[0].content, Some("Be concise.".to_string()));
        assert_eq!(openai_messages[1].role, "user");
    }

    #[test]
    fn test_convert_messages_with_system_prompt_from_message() {
        let client = OpenAIClient::new();
        let messages = vec![
            create_test_message(MessageRole::System, "Be verbose."),
            create_test_message(MessageRole::User, "Hello"),
        ];
        let openai_messages = client.convert_messages_to_openai(&messages);

        assert_eq!(openai_messages.len(), 2);
        assert_eq!(openai_messages[0].role, "system");
        assert_eq!(openai_messages[0].content, Some("Be verbose.".to_string()));
        assert_eq!(openai_messages[1].role, "user");
    }

    #[test]
    fn test_convert_messages_with_history() {
        let mut client = OpenAIClient::new();
        client.add_user_message("First message");
        client.add_assistant_message("First response", None);

        let messages = vec![create_test_message(MessageRole::User, "Second message")];

        let openai_messages = client.convert_messages_to_openai(&messages);

        assert_eq!(openai_messages.len(), 3);
        assert_eq!(openai_messages[0].role, "user");
        assert_eq!(
            openai_messages[0].content,
            Some("First message".to_string())
        );
        assert_eq!(openai_messages[1].role, "assistant");
        assert_eq!(
            openai_messages[1].content,
            Some("First response".to_string())
        );
        assert_eq!(openai_messages[2].role, "user");
        assert_eq!(
            openai_messages[2].content,
            Some("Second message".to_string())
        );
    }

    #[test]
    fn test_build_tools_empty() {
        let client = OpenAIClient::new();
        let mut config = ApiConfig::default();
        config.function_tools = vec![];
        let tools = client.build_tools(&config);
        assert!(tools.is_none());
    }

    #[test]
    fn test_build_tools_with_one_tool() {
        let client = OpenAIClient::new();
        let mut config = create_test_config();
        config.function_tools.clear(); // Clear default tools
        config.function_tools.push(FunctionTool {
            name: "get_weather".to_string(),
            description: "Get the current weather".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The city and state, e.g. San Francisco, CA"
                    }
                }
            }),
            mock_response: "".to_string(),
            enabled: true,
            category: "Weather".to_string(),
        });

        let tools = client.build_tools(&config);
        assert!(tools.is_some());
        let tool_vec = tools.unwrap();
        assert_eq!(tool_vec.len(), 1);
        let tool_json = &tool_vec[0];
        assert_eq!(tool_json["type"], "function");
        assert_eq!(tool_json["function"]["name"], "get_weather");
    }
}
