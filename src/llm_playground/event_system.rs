// Event system for decoupling message handling and LLM processing
use std::collections::HashMap;
use yew::prelude::*;

use crate::llm_playground::{FlexibleApiConfig, Message};
use crate::llm_playground::api_clients::traits::{FunctionCallRequest, LLMResponse};

// Event types for the system
#[derive(Debug, Clone, PartialEq)]
pub enum PlaygroundEvent {
    // User interaction events
    UserMessageSent {
        session_id: String,
        message: String,
    },
    
    // LLM processing events
    LLMRequestInitiated {
        session_id: String,
        messages: Vec<Message>,
        config: FlexibleApiConfig,
    },
    
    LLMResponseReceived {
        session_id: String,
        response: LLMResponse,
    },
    
    // Function execution events
    FunctionCallRequested {
        session_id: String,
        function_calls: Vec<FunctionCallRequest>,
        config: FlexibleApiConfig,
    },
    
    FunctionCallCompleted {
        session_id: String,
        function_call: FunctionCallRequest,
        result: serde_json::Value,
    },
    
    AllFunctionCallsCompleted {
        session_id: String,
        messages: Vec<Message>, // Updated message history
    },
    
    // Error events
    LLMError {
        session_id: String,
        error: String,
        is_retryable: bool,
    },
    
    FunctionError {
        session_id: String,
        function_name: String,
        error: String,
    },
    
    // UI state events
    LoadingStateChanged {
        session_id: String,
        is_loading: bool,
    },
    
    MessageAdded {
        session_id: String,
        message: Message,
    },
}

// Event bus for managing event subscriptions and emissions
#[derive(Debug, Clone, PartialEq)]
pub struct EventBus {
    listeners: Vec<Callback<PlaygroundEvent>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }

    pub fn subscribe(&mut self, callback: Callback<PlaygroundEvent>) {
        self.listeners.push(callback);
    }

    pub fn emit(&self, event: PlaygroundEvent) {
        for listener in &self.listeners {
            listener.emit(event.clone());
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

// Hook for using the event bus in components
#[hook]
pub fn use_event_bus() -> (EventBus, Callback<PlaygroundEvent>) {
    let event_bus = use_state(|| EventBus::new());
    
    let emit_event = {
        let event_bus = event_bus.clone();
        Callback::from(move |event: PlaygroundEvent| {
            event_bus.emit(event);
        })
    };
    
    ((*event_bus).clone(), emit_event)
}

// Message processing state
#[derive(Debug, Clone)]
pub struct MessageProcessingState {
    pub is_processing: bool,
    pub current_step: String,
    pub retry_count: u32,
    pub function_calls_pending: usize,
    pub function_calls_completed: usize,
}

impl Default for MessageProcessingState {
    fn default() -> Self {
        Self {
            is_processing: false,
            current_step: String::new(),
            retry_count: 0,
            function_calls_pending: 0,
            function_calls_completed: 0,
        }
    }
}

// Hook for managing processing state per session
#[hook]
pub fn use_processing_state() -> UseStateHandle<HashMap<String, MessageProcessingState>> {
    use_state(|| HashMap::new())
}