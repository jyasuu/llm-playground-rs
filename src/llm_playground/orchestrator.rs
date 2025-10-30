// Orchestrator for coordinating the decoupled message flow
use gloo_console::log;
use std::collections::HashMap;
use yew::prelude::*;

use crate::llm_playground::{
    FlexibleApiConfig, Message, MessageRole,
    flexible_client::FlexibleLLMClient,
    mcp_client::McpClient,
    ChatSession,
    components::notification::NotificationMessage,
};
use super::{
    event_system::{PlaygroundEvent, EventBus, MessageProcessingState, use_processing_state},
    message_handler::{MessageHandler, use_message_handler},
    llm_processor::{LLMProcessor, use_llm_processor},
    function_executor::{FunctionExecutor, use_function_executor},
};

// Main orchestrator that coordinates all the decoupled components
#[derive(Debug, Clone)]
pub struct PlaygroundOrchestrator {
    message_handler: MessageHandler,
    llm_processor: LLMProcessor,
    function_executor: FunctionExecutor,
    event_bus: EventBus,
    api_config: UseStateHandle<FlexibleApiConfig>,
    on_loading_change: Option<Callback<(String, bool)>>,
}

impl PlaygroundOrchestrator {
    pub fn new(
        message_handler: MessageHandler,
        llm_processor: LLMProcessor,
        function_executor: FunctionExecutor,
        event_bus: EventBus,
        api_config: UseStateHandle<FlexibleApiConfig>,
    ) -> Self {
        Self {
            message_handler,
            llm_processor,
            function_executor,
            event_bus,
            api_config,
            on_loading_change: None,
        }
    }

    // Initialize event listeners
    pub fn setup_event_listeners(&self) {
        let orchestrator = self.clone();
        let _event_callback = Callback::from(move |event: PlaygroundEvent| {
            orchestrator.handle_event(event);
        });
        
        // Note: In a real implementation, you'd need to properly manage the event bus subscription
        // This is a simplified version for demonstration
    }

    // Handle all events in the system
    pub fn handle_event(&self, event: PlaygroundEvent) {
        match event {
            PlaygroundEvent::UserMessageSent { session_id, message: _ } => {
                self.handle_user_message_sent(session_id);
            }

            PlaygroundEvent::LLMResponseReceived { session_id, response } => {
                self.handle_llm_response_received(session_id, response);
            }

            PlaygroundEvent::AllFunctionCallsCompleted { session_id, messages } => {
                self.handle_function_calls_completed(session_id, messages);
            }

            PlaygroundEvent::LLMError { session_id, error: _, is_retryable: _ } => {
                self.handle_llm_error(session_id);
            }

            PlaygroundEvent::FunctionError { session_id, function_name, error } => {
                self.handle_function_error(session_id, function_name, error);
            }

            // Other events can be handled by individual components
            _ => {}
        }
    }

    // Handle user message sent - initiate LLM processing
    fn handle_user_message_sent(&self, session_id: String) {
        log!("Orchestrator: Handling user message sent for session {}", &session_id);
        
        let messages = self.message_handler.get_session_messages(&session_id);
        let config = (*self.api_config).clone();

        self.llm_processor.process_llm_request(session_id, messages, config);
    }

    // Handle LLM response - process content and function calls
    fn handle_llm_response_received(&self, session_id: String, response: crate::llm_playground::api_clients::traits::LLMResponse) {
        log!("Orchestrator: Handling LLM response for session {}", &session_id);

        // Add assistant message with content (if any)
        if let Some(content) = &response.content {
            if !content.trim().is_empty() {
                let assistant_message = Message {
                    id: format!("assistant_{}", js_sys::Date::now() as u64),
                    role: MessageRole::Assistant,
                    content: content.clone(),
                    timestamp: js_sys::Date::now(),
                    function_call: None,
                    function_response: None,
                };

                self.message_handler.add_message_to_session(session_id.clone(), assistant_message);
            }
        }

        // If there are function calls, add them to conversation and execute
        if !response.function_calls.is_empty() {
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

            self.message_handler.add_message_to_session(session_id.clone(), assistant_message);

            // Execute function calls
            let config = (*self.api_config).clone();
            self.function_executor.execute_function_calls(session_id, response.function_calls, config);
        } else {
            // No function calls, conversation is complete
            self.event_bus.emit(PlaygroundEvent::LoadingStateChanged {
                session_id,
                is_loading: false,
            });
        }
    }

    // Handle function calls completed - continue LLM conversation
    fn handle_function_calls_completed(&self, session_id: String, function_response_messages: Vec<Message>) {
        log!("Orchestrator: Handling function calls completed for session {}", &session_id);

        // Add all function response messages to session
        for message in function_response_messages {
            self.message_handler.add_message_to_session(session_id.clone(), message);
        }

        // Continue the conversation with LLM to get final response
        let messages = self.message_handler.get_session_messages(&session_id);
        let config = (*self.api_config).clone();

        self.llm_processor.process_llm_request(session_id, messages, config);
    }

    // Handle LLM error
    fn handle_llm_error(&self, session_id: String) {
        log!("Orchestrator: Handling LLM error for session {}", &session_id);
        
        self.event_bus.emit(PlaygroundEvent::LoadingStateChanged {
            session_id,
            is_loading: false,
        });
    }

    // Handle function error
    fn handle_function_error(&self, session_id: String, function_name: String, error: String) {
        log!("Orchestrator: Handling function error for session {}: {} - {}", session_id, function_name, error);
        
        // Could add error handling logic here, such as:
        // - Adding error message to conversation
        // - Retrying function call
        // - Continuing conversation with error context
    }

    // Public method to send a message (entry point for UI)
    pub fn send_message(&self, session_id: String, message: String) {
        // Handle user message
        self.message_handler.handle_user_message(session_id.clone(), message);
        
        // Immediately trigger LLM processing
        self.handle_user_message_sent(session_id);
    }
}

// Hook for using the orchestrator
#[hook]
pub fn use_playground_orchestrator(
    sessions: UseStateHandle<HashMap<String, ChatSession>>,
    api_config: UseStateHandle<FlexibleApiConfig>,
    llm_client: FlexibleLLMClient,
    mcp_client: Option<McpClient>,
    add_notification: Callback<NotificationMessage>,
    event_bus: EventBus,
) -> PlaygroundOrchestrator {
    let processing_state = use_processing_state();
    
    let message_handler = use_message_handler(sessions, event_bus.clone());
    let llm_processor = use_llm_processor(
        llm_client,
        event_bus.clone(),
        processing_state.clone(),
        add_notification,
    );
    let function_executor = use_function_executor(
        event_bus.clone(),
        processing_state,
        mcp_client,
    );
    
    PlaygroundOrchestrator::new(
        message_handler,
        llm_processor,
        function_executor,
        event_bus,
        api_config,
    )
}