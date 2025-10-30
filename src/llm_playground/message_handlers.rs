// Specialized handlers for different types of message flow events
use yew::prelude::*;
use std::collections::HashMap;

use crate::llm_playground::{
    types::{Message, MessageRole, ChatSession},
    message_flow::{MessageFlowEvent, MessageFlowEventBus, FunctionCall},
};

/// Handler for LLM responses that may contain function calls
pub struct LLMResponseHandler {
    event_bus: MessageFlowEventBus,
    sessions: UseStateHandle<HashMap<String, ChatSession>>,
}

impl LLMResponseHandler {
    pub fn new(
        event_bus: MessageFlowEventBus,
        sessions: UseStateHandle<HashMap<String, ChatSession>>,
    ) -> Self {
        let mut handler = Self { event_bus, sessions };
        handler.setup_handlers();
        handler
    }

    fn setup_handlers(&mut self) {
        let event_bus = self.event_bus.clone();
        let sessions = self.sessions.clone();

        // Handle LLM call completion
        self.event_bus.subscribe(
            "llm_call_completed",
            Callback::from(move |event| {
                if let MessageFlowEvent::LLMCallCompleted {
                    session_id,
                    response_content,
                    function_calls,
                } = event
                {
                    Self::handle_llm_response(
                        &event_bus,
                        &sessions,
                        session_id,
                        response_content,
                        function_calls,
                    );
                }
            }),
        );
    }

    fn handle_llm_response(
        event_bus: &MessageFlowEventBus,
        sessions: &UseStateHandle<HashMap<String, ChatSession>>,
        session_id: String,
        response_content: Option<String>,
        function_calls: Vec<FunctionCall>,
    ) {
        // Add assistant message if there's content
        if let Some(content) = response_content {
            let assistant_message = Message {
                id: format!("msg_fc_{}", js_sys::Date::now() as u64),
                role: MessageRole::Assistant,
                content: content.clone(),
                timestamp: js_sys::Date::now(),
                function_call: if !function_calls.is_empty() {
                    Some(serde_json::json!(function_calls
                        .iter()
                        .map(|fc| {
                            serde_json::json!({
                                "id": fc.id,
                                "name": fc.name,
                                "arguments": fc.arguments
                            })
                        })
                        .collect::<Vec<_>>()))
                } else {
                    None
                },
                function_response: None,
            };

            // Update session with assistant message
            let mut updated_sessions: HashMap<String, ChatSession> = (**sessions).clone();
            if let Some(session) = updated_sessions.get_mut(&session_id) {
                session.messages.push(assistant_message.clone());
                session.updated_at = js_sys::Date::now();
                sessions.set(updated_sessions);

                event_bus.emit(MessageFlowEvent::MessageAdded {
                    session_id: session_id.clone(),
                    message: assistant_message,
                });
            }
        }

        // Process function calls if any
        if !function_calls.is_empty() {
            let total_calls = function_calls.len();
            for (index, function_call) in function_calls.into_iter().enumerate() {
                event_bus.emit(MessageFlowEvent::FunctionCallRequested {
                    session_id: session_id.clone(),
                    function_call,
                    call_index: index,
                    total_calls,
                });
            }
        } else {
            // No function calls, we're done with this conversation turn
            event_bus.emit(MessageFlowEvent::LoadingStateChanged { is_loading: false });
        }
    }
}

/// Handler for creating human-readable function call summaries
pub struct FunctionCallDisplayHandler {
    event_bus: MessageFlowEventBus,
    sessions: UseStateHandle<HashMap<String, ChatSession>>,
    function_call_summaries: UseStateHandle<HashMap<String, String>>, // session_id -> summary
}

impl FunctionCallDisplayHandler {
    pub fn new(
        event_bus: MessageFlowEventBus,
        sessions: UseStateHandle<HashMap<String, ChatSession>>,
        function_call_summaries: UseStateHandle<HashMap<String, String>>,
    ) -> Self {
        let mut handler = Self {
            event_bus,
            sessions,
            function_call_summaries,
        };
        handler.setup_handlers();
        handler
    }

    fn setup_handlers(&mut self) {
        let sessions = self.sessions.clone();
        let summaries = self.function_call_summaries.clone();
        let event_bus = self.event_bus.clone();

        // Handle function call completion to build summary
        self.event_bus.subscribe(
            "function_call_completed",
            Callback::from(move |event| {
                if let MessageFlowEvent::FunctionCallCompleted {
                    session_id,
                    function_call,
                    response,
                    call_index,
                    total_calls,
                } = event
                {
                    Self::handle_function_call_completed(
                        &sessions,
                        &summaries,
                        session_id,
                        function_call,
                        response,
                        call_index,
                        total_calls,
                    );
                }
            }),
        );

        // Handle batch completion to add final summary message
        let sessions2 = self.sessions.clone();
        let summaries2 = self.function_call_summaries.clone();
        self.event_bus.subscribe(
            "function_call_batch_completed",
            Callback::from(move |event| {
                if let MessageFlowEvent::FunctionCallBatchCompleted { session_id } = event {
                    Self::handle_batch_completed(&sessions2, &summaries2, session_id);
                }
            }),
        );
    }

    fn handle_function_call_completed(
        sessions: &UseStateHandle<HashMap<String, ChatSession>>,
        summaries: &UseStateHandle<HashMap<String, String>>,
        session_id: String,
        function_call: FunctionCall,
        response: serde_json::Value,
        call_index: usize,
        total_calls: usize,
    ) {
        let mut current_summaries_map: HashMap<String, String> = (**summaries).clone();
        let summary = current_summaries_map
            .entry(session_id.clone())
            .or_insert_with(|| {
                if total_calls == 1 {
                    "## 🔧 Function Execution\n\n".to_string()
                } else {
                    format!("## 🔧 Function Execution Sequence ({} calls)\n\n", total_calls)
                }
            });

        // Get function description from session's config (we'll need to pass this through events)
        let description = "Execute function".to_string(); // Placeholder

        summary.push_str(&format!(
            "### Step {}: Calling `{}`\n\n**Function**: `{}()`\n**Purpose**: {}\n\n**📤 Request Parameters**:\n```json\n{}\n```\n\n**📥 Response Received**:\n```json\n{}\n```\n\n**✅ Function call completed**\n\n",
            call_index + 1,
            function_call.name,
            function_call.name,
            description,
            serde_json::to_string_pretty(&function_call.arguments).unwrap_or_else(|_| "{}".to_string()),
            serde_json::to_string_pretty(&response).unwrap_or_else(|_| "Invalid response".to_string())
        ));

        summaries.set(current_summaries_map);
    }

    fn handle_batch_completed(
        sessions: &UseStateHandle<HashMap<String, ChatSession>>,
        summaries: &UseStateHandle<HashMap<String, String>>,
        session_id: String,
    ) {
        if let Some(summary) = summaries.get(&session_id) {
            let mut final_summary = summary.clone();
            final_summary.push_str("---\n\n**🔄 Function Execution Summary**: All function calls completed successfully.\n\n");

            // Add summary as assistant message
            let summary_message = Message {
                id: format!("assistant_summary_{}", js_sys::Date::now() as u64),
                role: MessageRole::Assistant,
                content: final_summary,
                timestamp: js_sys::Date::now(),
                function_call: None,
                function_response: None,
            };

            let mut updated_sessions: HashMap<String, ChatSession> = (**sessions).clone();
            if let Some(session) = updated_sessions.get_mut(&session_id) {
                session.messages.push(summary_message);
                session.updated_at = js_sys::Date::now();
                sessions.set(updated_sessions);
            }

            // Clear the summary for this session
            let mut updated_summaries: HashMap<String, String> = (**summaries).clone();
            updated_summaries.remove(&session_id);
            summaries.set(updated_summaries);
        }
    }
}

/// Error handler for displaying notifications and managing error states
pub struct ErrorHandler {
    add_notification: Callback<crate::llm_playground::notification::NotificationMessage>,
}

impl ErrorHandler {
    pub fn new(
        event_bus: &mut MessageFlowEventBus,
        add_notification: Callback<crate::llm_playground::notification::NotificationMessage>,
    ) -> Self {
        let handler = Self { add_notification };
        handler.setup_handlers(event_bus);
        handler
    }

    fn setup_handlers(&self, event_bus: &mut MessageFlowEventBus) {
        let add_notification = self.add_notification.clone();

        event_bus.subscribe(
            "error_occurred",
            Callback::from(move |event| {
                if let MessageFlowEvent::ErrorOccurred {
                    session_id: _,
                    error,
                    is_retryable,
                } = event
                {
                    let notification_type = if is_retryable {
                        crate::llm_playground::notification::NotificationType::Warning
                    } else {
                        crate::llm_playground::notification::NotificationType::Error
                    };

                    let duration = if is_retryable { 3000 } else { 6000 };

                    let notification = crate::llm_playground::notification::NotificationMessage::new(
                        if is_retryable {
                            error
                        } else {
                            format!("API Error: {}", error)
                        },
                        notification_type,
                    )
                    .with_duration(duration);

                    add_notification.emit(notification);
                }
            }),
        );
    }
}