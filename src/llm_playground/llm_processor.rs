// LLM processor for handling all LLM API interactions
use gloo_console::log;
use gloo_timers::future::TimeoutFuture;
use std::collections::HashMap;
use yew::prelude::*;

use crate::llm_playground::{
    FlexibleApiConfig, Message, MessageRole,
    flexible_client::FlexibleLLMClient,
    components::notification::{NotificationMessage, NotificationType},
    api_clients::traits::LLMResponse,
};
use super::event_system::{PlaygroundEvent, EventBus, MessageProcessingState};

// LLM processor for decoupled LLM handling
#[derive(Debug, Clone)]
pub struct LLMProcessor {
    client: FlexibleLLMClient,
    event_bus: EventBus,
    processing_state: UseStateHandle<HashMap<String, MessageProcessingState>>,
    add_notification: Callback<NotificationMessage>,
    on_response: Option<Callback<(String, crate::llm_playground::api_clients::traits::LLMResponse)>>,
    on_loading_change: Option<Callback<(String, bool)>>,
}

impl LLMProcessor {
    pub fn new(
        client: FlexibleLLMClient,
        event_bus: EventBus,
        processing_state: UseStateHandle<HashMap<String, MessageProcessingState>>,
        add_notification: Callback<NotificationMessage>,
    ) -> Self {
        Self {
            client,
            event_bus,
            processing_state,
            add_notification,
            on_response: None,
            on_loading_change: None,
        }
    }

    // Process LLM request
    pub fn process_llm_request(&self, session_id: String, messages: Vec<Message>, config: FlexibleApiConfig) {
        log!("LLMProcessor: Processing LLM request for session {}", &session_id);

        // Update processing state
        self.update_processing_state(&session_id, |state| {
            state.is_processing = true;
            state.current_step = "Sending request to LLM".to_string();
        });

        // Emit loading state change
        self.event_bus.emit(PlaygroundEvent::LoadingStateChanged {
            session_id: session_id.clone(),
            is_loading: true,
        });

        let client = self.client.clone();
        let event_bus = self.event_bus.clone();
        let add_notification = self.add_notification.clone();
        let processing_state = self.processing_state.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let mut prepared_messages = messages.clone();

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

            // Retry logic
            let mut retry_attempt = 0u32;
            let max_retries = 3u32;

            let api_result: Result<LLMResponse, String> = loop {
                match client.send_message(&prepared_messages, &config).await {
                    Ok(response) => break Ok(response),
                    Err(error) => {
                        let is_retryable = Self::is_retryable_error(&error);
                        
                        if is_retryable && retry_attempt < max_retries {
                            retry_attempt += 1;
                            let delay_ms = Self::calculate_retry_delay(
                                config.shared_settings.retry_delay,
                                retry_attempt - 1,
                            );

                            // Update processing state
                            {
                                let mut state_map = (*processing_state).clone();
                                if let Some(state) = state_map.get_mut(&session_id) {
                                    state.retry_count = retry_attempt;
                                    state.current_step = format!("Retrying... (attempt {}/{})", retry_attempt, max_retries + 1);
                                }
                                processing_state.set(state_map);
                            }

                            // Show notification for rate limit
                            let notification = NotificationMessage::new(
                                format!("Rate limit hit. Retrying in {}ms... (attempt {}/{})", 
                                    delay_ms, retry_attempt, max_retries + 1),
                                NotificationType::Warning
                            ).with_duration(delay_ms + 1000);
                            add_notification.emit(notification);

                            log!(
                                "Rate limit hit, retrying in {}ms (attempt {})",
                                delay_ms,
                                retry_attempt
                            );

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
                                if is_retryable { NotificationType::Error } else { NotificationType::Error },
                            ).with_duration(if is_retryable { 8000 } else { 6000 });
                            add_notification.emit(notification);

                            event_bus.emit(PlaygroundEvent::LLMError {
                                session_id: session_id.clone(),
                                error: final_error,
                                is_retryable,
                            });

                            // Reset processing state
                            {
                                let mut state_map = (*processing_state).clone();
                                if let Some(state) = state_map.get_mut(&session_id) {
                                    *state = MessageProcessingState::default();
                                }
                                processing_state.set(state_map);
                            }

                            event_bus.emit(PlaygroundEvent::LoadingStateChanged {
                                session_id,
                                is_loading: false,
                            });

                            return;
                        }
                    }
                }
            };

            match api_result {
                Ok(response) => {
                    log!("LLMProcessor: Received response from LLM");
                    
                    // Update processing state
                    {
                        let mut state_map = (*processing_state).clone();
                        if let Some(state) = state_map.get_mut(&session_id) {
                            state.current_step = "Processing LLM response".to_string();
                            state.function_calls_pending = response.function_calls.len();
                            state.function_calls_completed = 0;
                        }
                        processing_state.set(state_map);
                    }

                    event_bus.emit(PlaygroundEvent::LLMResponseReceived {
                        session_id,
                        response,
                    });
                }
                Err(_) => {
                    // Error already handled above
                }
            }
        });
    }

    // Helper function to check if error is retryable (429 rate limit)
    fn is_retryable_error(error: &str) -> bool {
        error.contains("429")
            || error.contains("Rate limit exceeded")
            || error.contains("rate limit")
    }

    // Helper function for exponential backoff delay
    fn calculate_retry_delay(base_delay: u32, attempt: u32) -> u32 {
        base_delay * (2_u32.pow(attempt.min(5))) // Cap at 2^5 to prevent excessive delays
    }

    // Update processing state helper
    fn update_processing_state<F>(&self, session_id: &str, updater: F)
    where
        F: FnOnce(&mut MessageProcessingState),
    {
        let mut state_map = (*self.processing_state).clone();
        let state = state_map.entry(session_id.to_string()).or_default();
        updater(state);
        self.processing_state.set(state_map);
    }
}

// Hook for using the LLM processor
#[hook]
pub fn use_llm_processor(
    client: FlexibleLLMClient,
    event_bus: EventBus,
    processing_state: UseStateHandle<HashMap<String, MessageProcessingState>>,
    add_notification: Callback<NotificationMessage>,
) -> LLMProcessor {
    LLMProcessor::new(client, event_bus, processing_state, add_notification)
}