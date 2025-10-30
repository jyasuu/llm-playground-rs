// Message handler for managing user input and message state
use gloo_console::log;
use std::collections::HashMap;
use yew::prelude::*;

use crate::llm_playground::{ChatSession, Message, MessageRole};
use super::event_system::{PlaygroundEvent, EventBus};

// Message handler component for decoupled message management
#[derive(Debug, Clone)]
pub struct MessageHandler {
    sessions: UseStateHandle<HashMap<String, ChatSession>>,
    event_bus: EventBus,
}

impl MessageHandler {
    pub fn new(
        sessions: UseStateHandle<HashMap<String, ChatSession>>,
        event_bus: EventBus,
    ) -> Self {
        Self {
            sessions,
            event_bus,
        }
    }

    // Handle user message input
    pub fn handle_user_message(&self, session_id: String, message_content: String) {
        if message_content.trim().is_empty() {
            return;
        }

        log!("MessageHandler: Processing user message for session {}", &session_id);

        // Create user message
        let user_message = Message {
            id: format!("user_{}", js_sys::Date::now() as u64),
            role: MessageRole::User,
            content: message_content.clone(),
            timestamp: js_sys::Date::now(),
            function_call: None,
            function_response: None,
        };

        // Add user message to session
        let mut new_sessions = (*self.sessions).clone();
        if let Some(session) = new_sessions.get_mut(&session_id) {
            session.messages.push(user_message.clone());
            session.updated_at = js_sys::Date::now();
            self.sessions.set(new_sessions);

            // Emit events
            self.event_bus.emit(PlaygroundEvent::MessageAdded {
                session_id: session_id.clone(),
                message: user_message,
            });

            self.event_bus.emit(PlaygroundEvent::UserMessageSent {
                session_id,
                message: message_content,
            });
        }
    }

    // Handle adding any message to session
    pub fn add_message_to_session(&self, session_id: String, message: Message) {
        let mut new_sessions = (*self.sessions).clone();
        if let Some(session) = new_sessions.get_mut(&session_id) {
            session.messages.push(message.clone());
            session.updated_at = js_sys::Date::now();
            self.sessions.set(new_sessions);

            self.event_bus.emit(PlaygroundEvent::MessageAdded {
                session_id,
                message,
            });
        }
    }

    // Get current messages for a session
    pub fn get_session_messages(&self, session_id: &str) -> Vec<Message> {
        self.sessions
            .get(session_id)
            .map(|session| session.messages.clone())
            .unwrap_or_default()
    }

    // Update session with new messages
    pub fn update_session_messages(&self, session_id: String, messages: Vec<Message>) {
        let mut new_sessions = (*self.sessions).clone();
        if let Some(session) = new_sessions.get_mut(&session_id) {
            session.messages = messages;
            session.updated_at = js_sys::Date::now();
            self.sessions.set(new_sessions);
        }
    }
}

// Hook for using the message handler
#[hook]
pub fn use_message_handler(
    sessions: UseStateHandle<HashMap<String, ChatSession>>,
    event_bus: EventBus,
) -> MessageHandler {
    MessageHandler::new(sessions, event_bus)
}