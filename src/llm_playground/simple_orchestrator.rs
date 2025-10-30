// Simplified orchestrator that directly handles the message flow
use gloo_console::log;
use gloo_timers::future::TimeoutFuture;
use std::collections::HashMap;
use yew::prelude::*;

use crate::llm_playground::{
    FlexibleApiConfig, Message, MessageRole, ChatSession,
    flexible_client::FlexibleLLMClient,
    mcp_client::McpClient,
    components::notification::{NotificationMessage, NotificationType},
};

// Simple orchestrator that handles the message flow directly
#[derive(Debug, Clone)]
pub struct SimpleOrchestrator {
    sessions: UseStateHandle<HashMap<String, ChatSession>>,
    api_config: UseStateHandle<FlexibleApiConfig>,
    llm_client: FlexibleLLMClient,
    mcp_client: Option<McpClient>,
    add_notification: Callback<NotificationMessage>,
    on_loading_change: Callback<(String, bool)>,
}

impl SimpleOrchestrator {
    pub fn new(
        sessions: UseStateHandle<HashMap<String, ChatSession>>,
        api_config: UseStateHandle<FlexibleApiConfig>,
        llm_client: FlexibleLLMClient,
        mcp_client: Option<McpClient>,
        add_notification: Callback<NotificationMessage>,
        on_loading_change: Callback<(String, bool)>,
    ) -> Self {
        Self {
            sessions,
            api_config,
            llm_client,
            mcp_client,
            add_notification,
            on_loading_change,
        }
    }

    // Send a message and handle the complete flow
    pub fn send_message(&self, session_id: String, message_content: String) {
        if message_content.trim().is_empty() {
            return;
        }

        log!("SimpleOrchestrator: Processing message for session {}", &session_id);

        // 1. Add user message to session
        let user_message = Message {
            id: format!("user_{}", js_sys::Date::now() as u64),
            role: MessageRole::User,
            content: message_content,
            timestamp: js_sys::Date::now(),
            function_call: None,
            function_response: None,
        };

        self.add_message_to_session(&session_id, user_message.clone());

        // 2. Set loading state
        self.on_loading_change.emit((session_id.clone(), true));

        // 3. Get current messages INCLUDING the just-added user message
        let mut current_messages = self.get_session_messages(&session_id);
        
        // If for some reason the message isn't there yet, add it manually
        if current_messages.is_empty() || !current_messages.iter().any(|m| m.id == user_message.id) {
            current_messages.push(user_message);
        }

        // 4. Process with LLM using the messages we know are correct
        self.process_with_llm_direct(session_id, current_messages);
    }

    // Process messages with LLM
    fn process_with_llm(&self, session_id: String) {
        let messages = self.get_session_messages(&session_id);
        self.process_with_llm_direct(session_id, messages);
    }

    // Process messages with LLM using provided messages
    fn process_with_llm_direct(&self, session_id: String, messages: Vec<Message>) {
        let config = (*self.api_config).clone();
        let llm_client = self.llm_client.clone();
        let add_notification = self.add_notification.clone();
        let on_loading_change = self.on_loading_change.clone();
        let sessions = self.sessions.clone();
        let mcp_client = self.mcp_client.clone();
        let api_config = self.api_config.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let mut prepared_messages = messages.clone();
            
            log!("Original messages count: {}", messages.len());
            for (i, msg) in messages.iter().enumerate() {
                log!("Message {}: {} - {}", i, format!("{:?}", msg.role), &msg.content);
            }

            // Add system message if exists
            if !config.system_prompt.trim().is_empty() {
                log!("Adding system prompt: {}", &config.system_prompt);
                prepared_messages.insert(
                    0,
                    Message {
                        id: "system".to_string(),
                        role: MessageRole::System,
                        content: config.system_prompt.clone(),
                        timestamp: js_sys::Date::now(),
                        function_call: None,
                        function_response: None,
                    },
                );
            }

            log!("Prepared messages count: {}", prepared_messages.len());
            for (i, msg) in prepared_messages.iter().enumerate() {
                log!("Prepared message {}: {} - {}", i, format!("{:?}", msg.role), &msg.content);
            }

            // Send to LLM with retry logic
            let mut retry_attempt = 0u32;
            let max_retries = 3u32;

            let api_result: Result<crate::llm_playground::api_clients::traits::LLMResponse, String> = loop {
                log!("Sending {} messages to LLM", prepared_messages.len());
                match llm_client.send_message(&prepared_messages, &config).await {
                    Ok(response) => break Ok(response),
                    Err(error) => {
                        let is_retryable = error.contains("429") || error.contains("Rate limit");
                        
                        if is_retryable && retry_attempt < max_retries {
                            retry_attempt += 1;
                            let delay_ms = config.shared_settings.retry_delay * (2_u32.pow(retry_attempt.min(5)));

                            // Show notification for rate limit
                            let notification = NotificationMessage::new(
                                format!("Rate limit hit. Retrying in {}ms... (attempt {}/{})", 
                                    delay_ms, retry_attempt, max_retries + 1),
                                NotificationType::Warning
                            ).with_duration(delay_ms + 1000);
                            add_notification.emit(notification);

                            log!("Rate limit hit, retrying in {}ms (attempt {})", delay_ms, retry_attempt);

                            // Wait before retry
                            TimeoutFuture::new(delay_ms).await;
                            continue;
                        } else {
                            // Non-retryable error or max retries exceeded
                            let final_error = if is_retryable && retry_attempt >= max_retries {
                                format!("Rate limit exceeded. Max retries ({}) reached. Please wait before trying again.", max_retries + 1)
                            } else {
                                error
                            };

                            let notification = NotificationMessage::new(
                                format!("API Error: {}", final_error),
                                NotificationType::Error,
                            ).with_duration(8000);
                            add_notification.emit(notification);

                            // Set loading to false
                            on_loading_change.emit((session_id, false));
                            return;
                        }
                    }
                }
            };

            match api_result {
                Ok(response) => {
                    log!("SimpleOrchestrator: Received response from LLM");
                    log!("Response content: {}", format!("{:?}", &response.content));
                    log!("Response function calls: {}", response.function_calls.len());
                    
                    // Add assistant message with content (if any)
                    if let Some(content) = &response.content {
                        if !content.trim().is_empty() {
                            log!("Adding assistant message with content: {}", content);
                            let assistant_message = Message {
                                id: format!("assistant_{}", js_sys::Date::now() as u64),
                                role: MessageRole::Assistant,
                                content: content.clone(),
                                timestamp: js_sys::Date::now(),
                                function_call: None,
                                function_response: None,
                            };

                            Self::add_message_to_session_static(&sessions, &session_id, assistant_message);
                        } else {
                            log!("Response content is empty - adding debug message");
                            // Add a debug message to show what happened
                            let debug_message = Message {
                                id: format!("debug_{}", js_sys::Date::now() as u64),
                                role: MessageRole::Assistant,
                                content: "[DEBUG] LLM returned empty content. This might indicate an API configuration issue or the model didn't generate a response.".to_string(),
                                timestamp: js_sys::Date::now(),
                                function_call: None,
                                function_response: None,
                            };
                            Self::add_message_to_session_static(&sessions, &session_id, debug_message);
                        }
                    } else {
                        log!("Response content is None - adding debug message");
                        // Add a debug message to show what happened
                        let debug_message = Message {
                            id: format!("debug_none_{}", js_sys::Date::now() as u64),
                            role: MessageRole::Assistant,
                            content: "[DEBUG] LLM returned no content at all. This indicates an API configuration or connection issue.".to_string(),
                            timestamp: js_sys::Date::now(),
                            function_call: None,
                            function_response: None,
                        };
                        Self::add_message_to_session_static(&sessions, &session_id, debug_message);
                    }

                    // Handle function calls if any
                    if !response.function_calls.is_empty() {
                        log!("SimpleOrchestrator: Processing {} function calls", response.function_calls.len());
                        
                        // Add assistant message with function calls
                        let assistant_message = Message {
                            id: format!("msg_fc_{}", js_sys::Date::now() as u64),
                            role: MessageRole::Assistant,
                            content: response.content.unwrap_or_default(),
                            timestamp: js_sys::Date::now(),
                            function_call: Some(serde_json::json!(response
                                .function_calls
                                .iter()
                                .map(|fc| {
                                    serde_json::json!({
                                        "id": fc.id,
                                        "name": fc.name,
                                        "arguments": fc.arguments
                                    })
                                })
                                .collect::<Vec<_>>())),
                            function_response: None,
                        };

                        Self::add_message_to_session_static(&sessions, &session_id, assistant_message);

                        // Execute function calls
                        let mut function_responses = Vec::new();
                        for function_call in &response.function_calls {
                            log!("Executing function: {}", &function_call.name);

                            let result = Self::execute_function_call(
                                function_call,
                                &config,
                                mcp_client.as_ref(),
                            ).await;

                            match result {
                                Ok(response_value) => {
                                    let function_response_message = Message {
                                        id: format!("msg_fr_{}", js_sys::Date::now() as u64),
                                        role: MessageRole::Function,
                                        content: format!("Function {} executed", function_call.name),
                                        timestamp: js_sys::Date::now(),
                                        function_call: None,
                                        function_response: Some(serde_json::json!({
                                            "id": function_call.id,
                                            "name": function_call.name,
                                            "response": response_value
                                        })),
                                    };

                                    function_responses.push(function_response_message);
                                }
                                Err(error) => {
                                    log!("Function execution error: {}", &error);
                                    let error_response_message = Message {
                                        id: format!("msg_fr_err_{}", js_sys::Date::now() as u64),
                                        role: MessageRole::Function,
                                        content: format!("Function {} failed", function_call.name),
                                        timestamp: js_sys::Date::now(),
                                        function_call: None,
                                        function_response: Some(serde_json::json!({
                                            "id": function_call.id,
                                            "name": function_call.name,
                                            "error": "Function execution failed"
                                        })),
                                    };

                                    function_responses.push(error_response_message);
                                }
                            }
                        }

                        // Add all function response messages
                        for message in function_responses {
                            Self::add_message_to_session_static(&sessions, &session_id, message);
                        }

                        // Continue the conversation with LLM to get final response
                        let final_messages = Self::get_session_messages_static(&sessions, &session_id);
                        let orchestrator = SimpleOrchestrator {
                            sessions: sessions.clone(),
                            api_config: api_config.clone(),
                            llm_client: llm_client.clone(),
                            mcp_client: mcp_client.clone(),
                            add_notification: add_notification.clone(),
                            on_loading_change: on_loading_change.clone(),
                        };
                        orchestrator.process_with_llm_continuation(session_id, final_messages);
                    } else {
                        // No function calls, conversation is complete
                        on_loading_change.emit((session_id, false));
                    }
                }
                Err(_) => {
                    // Error already handled above
                }
            }
        });
    }

    // Continue LLM processing after function calls
    fn process_with_llm_continuation(&self, session_id: String, messages: Vec<Message>) {
        let config = (*self.api_config).clone();
        let llm_client = self.llm_client.clone();
        let on_loading_change = self.on_loading_change.clone();
        let sessions = self.sessions.clone();
        let add_notification = self.add_notification.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let mut prepared_messages = messages;

            // Add system message if exists
            if !config.system_prompt.trim().is_empty() {
                prepared_messages.insert(
                    0,
                    Message {
                        id: "system".to_string(),
                        role: MessageRole::System,
                        content: config.system_prompt.clone(),
                        timestamp: js_sys::Date::now(),
                        function_call: None,
                        function_response: None,
                    },
                );
            }

            match llm_client.send_message(&prepared_messages, &config).await {
                Ok(response) => {
                    log!("SimpleOrchestrator: Received final response from LLM");
                    
                    // Add final assistant response
                    if let Some(content) = response.content {
                        if !content.trim().is_empty() {
                            let assistant_message = Message {
                                id: format!("assistant_final_{}", js_sys::Date::now() as u64),
                                role: MessageRole::Assistant,
                                content,
                                timestamp: js_sys::Date::now(),
                                function_call: None,
                                function_response: None,
                            };

                            Self::add_message_to_session_static(&sessions, &session_id, assistant_message);
                        }
                    }

                    // Set loading to false
                    on_loading_change.emit((session_id, false));
                }
                Err(error) => {
                    log!("Final LLM call error: {}", &error);
                    let notification = NotificationMessage::new(
                        format!("Final API Error: {}", error),
                        NotificationType::Error,
                    ).with_duration(6000);
                    add_notification.emit(notification);

                    // Set loading to false
                    on_loading_change.emit((session_id, false));
                }
            }
        });
    }

    // Execute a single function call
    async fn execute_function_call(
        function_call: &crate::llm_playground::api_clients::traits::FunctionCallRequest,
        config: &FlexibleApiConfig,
        mcp_client: Option<&McpClient>,
    ) -> Result<serde_json::Value, String> {
        // Check if this is a built-in tool and execute it properly
        if let Some(tool) = config
            .function_tools
            .iter()
            .find(|tool| tool.name == function_call.name)
        {
            if tool.is_builtin {
                // Execute built-in tool with real functionality (including MCP tools)
                log!("Executing built-in tool: {}", &function_call.name);
                match crate::llm_playground::builtin_tools::execute_builtin_tool(
                    &function_call.name,
                    &function_call.arguments,
                    mcp_client,
                ).await {
                    Ok(result) => Ok(result),
                    Err(error) => Ok(serde_json::json!({"error": error})),
                }
            } else {
                // Use mock response for regular tools
                Ok(serde_json::from_str(&tool.mock_response)
                    .unwrap_or_else(|_| serde_json::json!({"result": tool.mock_response.clone()})))
            }
        } else {
            // Unknown tool
            Ok(serde_json::json!({"error": "Unknown function tool"}))
        }
    }

    // Helper methods
    fn add_message_to_session(&self, session_id: &str, message: Message) {
        Self::add_message_to_session_static(&self.sessions, session_id, message);
    }

    fn add_message_to_session_static(
        sessions: &UseStateHandle<HashMap<String, ChatSession>>,
        session_id: &str,
        message: Message,
    ) {
        let mut new_sessions = (**sessions).clone();
        if let Some(session) = new_sessions.get_mut(session_id) {
            session.messages.push(message);
            session.updated_at = js_sys::Date::now();
            sessions.set(new_sessions);
        }
    }

    fn get_session_messages(&self, session_id: &str) -> Vec<Message> {
        Self::get_session_messages_static(&self.sessions, session_id)
    }

    fn get_session_messages_static(
        sessions: &UseStateHandle<HashMap<String, ChatSession>>,
        session_id: &str,
    ) -> Vec<Message> {
        sessions
            .get(session_id)
            .map(|session| session.messages.clone())
            .unwrap_or_default()
    }
}

// Hook for using the simple orchestrator
#[hook]
pub fn use_simple_orchestrator(
    sessions: UseStateHandle<HashMap<String, ChatSession>>,
    api_config: UseStateHandle<FlexibleApiConfig>,
    llm_client: FlexibleLLMClient,
    mcp_client: Option<McpClient>,
    add_notification: Callback<NotificationMessage>,
    on_loading_change: Callback<(String, bool)>,
) -> SimpleOrchestrator {
    SimpleOrchestrator::new(
        sessions,
        api_config,
        llm_client,
        mcp_client,
        add_notification,
        on_loading_change,
    )
}