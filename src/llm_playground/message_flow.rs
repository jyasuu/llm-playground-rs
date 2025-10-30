// Event-driven message flow system for decoupling user input from LLM interactions
use std::collections::HashMap;
use yew::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

use crate::llm_playground::{
    types::{Message, MessageRole, ChatSession},
    flexible_client::FlexibleLLMClient,
    provider_config::FlexibleApiConfig,
    mcp_client::McpClient,
    api_clients::traits::LLMResponse,
};
use gloo_timers::future::TimeoutFuture;

/// Events that can occur in the message flow system
#[derive(Debug, Clone)]
pub enum MessageFlowEvent {
    /// User submitted a new message
    UserMessageSubmitted {
        session_id: String,
        content: String,
    },
    /// Message was added to a session
    MessageAdded {
        session_id: String,
        message: Message,
    },
    /// LLM API call should be initiated
    LLMCallRequested {
        session_id: String,
        messages: Vec<Message>,
    },
    /// LLM API call completed successfully
    LLMCallCompleted {
        session_id: String,
        response_content: Option<String>,
        function_calls: Vec<FunctionCall>,
    },
    /// Function call should be executed
    FunctionCallRequested {
        session_id: String,
        function_call: FunctionCall,
        call_index: usize,
        total_calls: usize,
    },
    /// Function call completed
    FunctionCallCompleted {
        session_id: String,
        function_call: FunctionCall,
        response: serde_json::Value,
        call_index: usize,
        total_calls: usize,
    },
    /// All function calls in a batch completed
    FunctionCallBatchCompleted {
        session_id: String,
    },
    /// An error occurred
    ErrorOccurred {
        session_id: String,
        error: String,
        is_retryable: bool,
    },
    /// Loading state changed
    LoadingStateChanged {
        is_loading: bool,
    },
}

/// Function call data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Event bus for managing message flow events
#[derive(Debug, Clone)]
pub struct MessageFlowEventBus {
    listeners: HashMap<String, Vec<Callback<MessageFlowEvent>>>,
}

impl Default for MessageFlowEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageFlowEventBus {
    pub fn new() -> Self {
        Self {
            listeners: HashMap::new(),
        }
    }

    /// Subscribe to specific event types
    pub fn subscribe(&mut self, event_type: &str, callback: Callback<MessageFlowEvent>) {
        self.listeners
            .entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(callback);
    }

    /// Emit an event to all subscribers
    pub fn emit(&self, event: MessageFlowEvent) {
        let event_type = match &event {
            MessageFlowEvent::UserMessageSubmitted { .. } => "user_message_submitted",
            MessageFlowEvent::MessageAdded { .. } => "message_added",
            MessageFlowEvent::LLMCallRequested { .. } => "llm_call_requested",
            MessageFlowEvent::LLMCallCompleted { .. } => "llm_call_completed",
            MessageFlowEvent::FunctionCallRequested { .. } => "function_call_requested",
            MessageFlowEvent::FunctionCallCompleted { .. } => "function_call_completed",
            MessageFlowEvent::FunctionCallBatchCompleted { .. } => "function_call_batch_completed",
            MessageFlowEvent::ErrorOccurred { .. } => "error_occurred",
            MessageFlowEvent::LoadingStateChanged { .. } => "loading_state_changed",
        };

        if let Some(listeners) = self.listeners.get(event_type) {
            for callback in listeners {
                callback.emit(event.clone());
            }
        }
    }
}

/// Message flow coordinator that orchestrates the entire flow
#[derive(Clone)]
pub struct MessageFlowCoordinator {
    event_bus: MessageFlowEventBus,
    llm_client: FlexibleLLMClient,
    api_config: FlexibleApiConfig,
    mcp_client: Option<McpClient>,
    sessions: UseStateHandle<HashMap<String, ChatSession>>,
    is_loading: UseStateHandle<bool>,
}

impl MessageFlowCoordinator {
    pub fn new(
        llm_client: FlexibleLLMClient,
        api_config: FlexibleApiConfig,
        mcp_client: Option<McpClient>,
        sessions: UseStateHandle<HashMap<String, ChatSession>>,
        is_loading: UseStateHandle<bool>,
    ) -> Self {
        let mut coordinator = Self {
            event_bus: MessageFlowEventBus::new(),
            llm_client,
            api_config,
            mcp_client,
            sessions,
            is_loading,
        };

        coordinator.setup_event_handlers();
        coordinator
    }

    /// Get the event bus for external subscriptions
    pub fn get_event_bus(&self) -> &MessageFlowEventBus {
        &self.event_bus
    }

    /// Emit an event through the coordinator's event bus
    pub fn emit_event(&self, event: MessageFlowEvent) {
        self.event_bus.emit(event);
    }

    /// Setup internal event handlers for the message flow
    fn setup_event_handlers(&mut self) {
        // Handle user message submission
        let sessions = self.sessions.clone();
        let event_bus = self.event_bus.clone();
        self.event_bus.subscribe(
            "user_message_submitted",
            Callback::from(move |event| {
                if let MessageFlowEvent::UserMessageSubmitted { session_id, content } = event {
                    Self::handle_user_message_submitted(
                        &sessions,
                        &event_bus,
                        session_id,
                        content,
                    );
                }
            }),
        );

        // Handle LLM call requests
        let llm_client = self.llm_client.clone();
        let api_config = self.api_config.clone();
        let event_bus = self.event_bus.clone();
        self.event_bus.subscribe(
            "llm_call_requested",
            Callback::from(move |event| {
                if let MessageFlowEvent::LLMCallRequested { session_id, messages } = event {
                    Self::handle_llm_call_requested(
                        llm_client.clone(),
                        api_config.clone(),
                        event_bus.clone(),
                        session_id,
                        messages,
                    );
                }
            }),
        );

        // Handle function call requests
        let api_config = self.api_config.clone();
        let mcp_client = self.mcp_client.clone();
        let event_bus = self.event_bus.clone();
        let sessions = self.sessions.clone();
        self.event_bus.subscribe(
            "function_call_requested",
            Callback::from(move |event| {
                if let MessageFlowEvent::FunctionCallRequested {
                    session_id,
                    function_call,
                    call_index,
                    total_calls,
                } = event
                {
                    Self::handle_function_call_requested(
                        api_config.clone(),
                        mcp_client.clone(),
                        event_bus.clone(),
                        sessions.clone(),
                        session_id,
                        function_call,
                        call_index,
                        total_calls,
                    );
                }
            }),
        );

        // Handle function call batch completion
        let sessions = self.sessions.clone();
        let event_bus = self.event_bus.clone();
        self.event_bus.subscribe(
            "function_call_batch_completed",
            Callback::from(move |event| {
                if let MessageFlowEvent::FunctionCallBatchCompleted { session_id } = event {
                    Self::handle_function_call_batch_completed(
                        &sessions,
                        &event_bus,
                        session_id,
                    );
                }
            }),
        );

        // Handle loading state changes
        let is_loading = self.is_loading.clone();
        self.event_bus.subscribe(
            "loading_state_changed",
            Callback::from(move |event| {
                if let MessageFlowEvent::LoadingStateChanged { is_loading: loading } = event {
                    is_loading.set(loading);
                }
            }),
        );
    }

    /// Handle user message submission - add to session and trigger LLM call
    fn handle_user_message_submitted(
        sessions: &UseStateHandle<HashMap<String, ChatSession>>,
        event_bus: &MessageFlowEventBus,
        session_id: String,
        content: String,
    ) {
        if content.trim().is_empty() {
            return;
        }

        // Create user message
        let user_message = Message {
            id: format!("user_{}", js_sys::Date::now() as u64),
            role: MessageRole::User,
            content: content.clone(),
            timestamp: js_sys::Date::now(),
            function_call: None,
            function_response: None,
        };

        // Add message to session
        let mut updated_sessions: HashMap<String, ChatSession> = (**sessions).clone();
        if let Some(session) = updated_sessions.get_mut(&session_id) {
            session.messages.push(user_message.clone());
            session.updated_at = js_sys::Date::now();
            
            // Clone the messages before moving updated_sessions
            let session_messages = session.messages.clone();
            
            sessions.set(updated_sessions);

            // Emit message added event
            event_bus.emit(MessageFlowEvent::MessageAdded {
                session_id: session_id.clone(),
                message: user_message,
            });

            // Emit loading state change
            event_bus.emit(MessageFlowEvent::LoadingStateChanged { is_loading: true });

            // Request LLM call with updated messages
            event_bus.emit(MessageFlowEvent::LLMCallRequested {
                session_id,
                messages: session_messages,
            });
        }
    }

    /// Handle LLM call request - make API call and process response
    fn handle_llm_call_requested(
        llm_client: FlexibleLLMClient,
        api_config: FlexibleApiConfig,
        event_bus: MessageFlowEventBus,
        session_id: String,
        messages: Vec<Message>,
    ) {
        spawn_local(async move {
            // Prepare messages with system prompt if needed
            let mut current_messages = messages;
            if !api_config.system_prompt.trim().is_empty() {
                current_messages.insert(
                    0,
                    Message {
                        id: "system".to_string(),
                        role: MessageRole::System,
                        content: api_config.system_prompt.clone(),
                        timestamp: js_sys::Date::now(),
                        function_call: None,
                        function_response: None,
                    },
                );
            }

            // Make LLM API call with retry logic
            match Self::make_llm_call_with_retry(&llm_client, &current_messages, &api_config, &event_bus, &session_id).await {
                Ok(response) => {
                    // Convert response function calls to our format
                    let function_calls: Vec<FunctionCall> = response
                        .function_calls
                        .into_iter()
                        .map(|fc| FunctionCall {
                            id: fc.id,
                            name: fc.name,
                            arguments: fc.arguments,
                        })
                        .collect();

                    event_bus.emit(MessageFlowEvent::LLMCallCompleted {
                        session_id,
                        response_content: response.content,
                        function_calls,
                    });
                }
                Err(error) => {
                    event_bus.emit(MessageFlowEvent::ErrorOccurred {
                        session_id,
                        error,
                        is_retryable: false,
                    });
                    event_bus.emit(MessageFlowEvent::LoadingStateChanged { is_loading: false });
                }
            }
        });
    }

    /// Make LLM call with retry logic for rate limits
    async fn make_llm_call_with_retry(
        llm_client: &FlexibleLLMClient,
        messages: &[Message],
        api_config: &FlexibleApiConfig,
        event_bus: &MessageFlowEventBus,
        session_id: &str,
    ) -> Result<LLMResponse, String> {
        let max_retries = 3u32;
        let mut retry_attempt = 0u32;

        loop {
            match llm_client.send_message(messages, api_config).await {
                Ok(response) => return Ok(response),
                Err(error) => {
                    let is_retryable = Self::is_retryable_error(&error);
                    
                    if is_retryable && retry_attempt < max_retries {
                        retry_attempt += 1;
                        let delay_ms = Self::calculate_retry_delay(
                            api_config.shared_settings.retry_delay,
                            retry_attempt - 1,
                        );

                        // Emit retry event
                        event_bus.emit(MessageFlowEvent::ErrorOccurred {
                            session_id: session_id.to_string(),
                            error: format!("Rate limit hit. Retrying in {}ms... (attempt {}/{})", 
                                delay_ms, retry_attempt, max_retries + 1),
                            is_retryable: true,
                        });

                        // Wait before retry
                        TimeoutFuture::new(delay_ms).await;
                        continue;
                    } else {
                        return Err(if is_retryable && retry_attempt >= max_retries {
                            format!("Rate limit exceeded. Max retries ({}) reached.", max_retries + 1)
                        } else {
                            error
                        });
                    }
                }
            }
        }
    }

    /// Handle function call request - execute function and emit result
    fn handle_function_call_requested(
        api_config: FlexibleApiConfig,
        mcp_client: Option<McpClient>,
        event_bus: MessageFlowEventBus,
        sessions: UseStateHandle<HashMap<String, ChatSession>>,
        session_id: String,
        function_call: FunctionCall,
        call_index: usize,
        total_calls: usize,
    ) {
        spawn_local(async move {
            // Execute the function call
            let response_value = if let Some(tool) = api_config
                .function_tools
                .iter()
                .find(|tool| tool.name == function_call.name)
            {
                if tool.is_builtin {
                    // Execute built-in tool
                    match crate::llm_playground::builtin_tools::execute_builtin_tool(
                        &function_call.name,
                        &function_call.arguments,
                        mcp_client.as_ref(),
                    )
                    .await
                    {
                        Ok(result) => result,
                        Err(error) => serde_json::json!({"error": error}),
                    }
                } else {
                    // Use mock response for regular tools
                    serde_json::from_str(&tool.mock_response)
                        .unwrap_or_else(|_| serde_json::json!({"result": tool.mock_response.clone()}))
                }
            } else {
                serde_json::json!({"error": "Unknown function tool"})
            };

            // Add function response message to session
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

            // Update session
            let mut new_sessions = (*sessions).clone();
            if let Some(session) = new_sessions.get_mut(&session_id) {
                session.messages.push(function_response_message.clone());
                session.updated_at = js_sys::Date::now();
                sessions.set(new_sessions);
            }

            // Emit function call completion
            event_bus.emit(MessageFlowEvent::FunctionCallCompleted {
                session_id: session_id.clone(),
                function_call,
                response: response_value,
                call_index,
                total_calls,
            });

            // If this was the last function call, emit batch completion
            if call_index + 1 >= total_calls {
                event_bus.emit(MessageFlowEvent::FunctionCallBatchCompleted { session_id });
            }
        });
    }

    /// Handle function call batch completion - trigger next LLM call
    fn handle_function_call_batch_completed(
        sessions: &UseStateHandle<HashMap<String, ChatSession>>,
        event_bus: &MessageFlowEventBus,
        session_id: String,
    ) {
        if let Some(session) = sessions.get(&session_id) {
            // Request another LLM call with updated messages (including function responses)
            event_bus.emit(MessageFlowEvent::LLMCallRequested {
                session_id,
                messages: session.messages.clone(),
            });
        }
    }

    /// Helper function to check if error is retryable
    fn is_retryable_error(error: &str) -> bool {
        error.contains("429")
            || error.contains("Rate limit exceeded")
            || error.contains("rate limit")
    }

    /// Helper function for exponential backoff delay
    fn calculate_retry_delay(base_delay: u32, attempt: u32) -> u32 {
        base_delay * (2_u32.pow(attempt.min(5)))
    }
}

/// Hook for using the message flow system in components
#[hook]
pub fn use_message_flow(
    llm_client: FlexibleLLMClient,
    api_config: FlexibleApiConfig,
    mcp_client: Option<McpClient>,
    sessions: UseStateHandle<HashMap<String, ChatSession>>,
    is_loading: UseStateHandle<bool>,
) -> MessageFlowCoordinator {
    let coordinator = use_state(|| {
        MessageFlowCoordinator::new(
            llm_client.clone(),
            api_config.clone(),
            mcp_client.clone(),
            sessions.clone(),
            is_loading.clone(),
        )
    });

    (*coordinator).clone()
}