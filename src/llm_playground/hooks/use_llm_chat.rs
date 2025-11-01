// Hook for sending messages to LLM only
use gloo_console::log;
use gloo_timers::future::TimeoutFuture;
use yew::prelude::*;

use crate::llm_playground::{
    api_clients::LLMResponse,
    components::notification::{NotificationMessage, NotificationType},
    flexible_client::FlexibleLLMClient,
    mcp_client::McpClient,
    FlexibleApiConfig, Message, MessageRole,
};

/// Hook for sending messages to LLM only
/// Returns: (send_to_llm_callback, is_loading_state)
#[hook]
pub fn use_llm_chat(
    api_config: UseStateHandle<FlexibleApiConfig>,
    llm_client: UseStateHandle<FlexibleLLMClient>,
    mcp_client: UseStateHandle<Option<McpClient>>,
    add_notification: Callback<NotificationMessage>,
    on_llm_response: Callback<LLMResponse>,
) -> (Callback<(Vec<Message>, FlexibleApiConfig)>, UseStateHandle<bool>) {
    let is_loading = use_state(|| false);

    // Helper function to check if error is retryable (429 rate limit)
    let is_retryable_error = |error: &str| -> bool {
        error.contains("429")
            || error.contains("Rate limit exceeded")
            || error.contains("rate limit")
    };

    // Helper function for exponential backoff delay
    let calculate_retry_delay = |base_delay: u32, attempt: u32| -> u32 {
        base_delay * (2_u32.pow(attempt.min(5))) // Cap at 2^5 to prevent excessive delays
    };

    let send_to_llm = {
        let is_loading = is_loading.clone();
        let api_config = api_config.clone();
        let llm_client = llm_client.clone();
        let add_notification = add_notification.clone();
        let on_llm_response = on_llm_response.clone();

        Callback::from(move |(messages, config): (Vec<Message>, FlexibleApiConfig)| {
            if messages.is_empty() {
                return;
            }

            is_loading.set(true);

            // Send to LLM
            let client = (*llm_client).clone();
            let is_loading_clone = is_loading.clone();
            let on_llm_response_clone = on_llm_response.clone();
            let add_notification_clone = add_notification.clone();

            wasm_bindgen_futures::spawn_local(async move {
                // Use the config passed directly from the playground
                let mut current_messages = messages;

                // Note: System prompt is now handled inside the API clients, not here

                // Send single request to LLM (no loop - playground will handle function call responses)
                log!("🚀 Sending {} messages to LLM", current_messages.len());
                let mut retry_attempt = 0u32;
                let max_retries = 3u32;

                log!("📤 Calling LLM API with {} messages...", current_messages.len());
                for (i, msg) in current_messages.iter().enumerate() {
                    let role_str = match msg.role {
                        MessageRole::User => "User",
                        MessageRole::Assistant => "Assistant", 
                        MessageRole::System => "System",
                        MessageRole::Function => "Function",
                    };
                    log!("  Message {}: {} - {}", i + 1, role_str, 
                        if msg.content.len() > 100 { &msg.content[..100] } else { &msg.content });
                }
                             
                let api_result = loop {
                    log!("⏳ Attempting LLM API call (attempt {})...", retry_attempt + 1);
                    
                    let (provider_name, model_name) = config.get_current_provider_and_model();
                    log!("🔍 use_llm_chat::send_message mcp_client - Provider: {}, Model: {}", &provider_name, &model_name);
                    
                    match client.send_message(&current_messages, &config).await {
                        Ok(response) => break Ok(response),
                        Err(error) => {
                            // Check if this is a retryable error (429 rate limit)
                            if is_retryable_error(&error) && retry_attempt < max_retries {
                                retry_attempt += 1;
                                let delay_ms = calculate_retry_delay(
                                    config.shared_settings.retry_delay,
                                    retry_attempt - 1,
                                );

                                // Show notification for rate limit
                                let notification = NotificationMessage::new(
                                    format!("Rate limit hit. Retrying in {}ms... (attempt {}/{})", 
                                        delay_ms, retry_attempt, max_retries + 1),
                                    NotificationType::Warning
                                ).with_duration(delay_ms + 1000);
                                add_notification_clone.emit(notification);

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
                                if is_retryable_error(&error) && retry_attempt >= max_retries {
                                    let final_error = format!("Rate limit exceeded. Max retries ({}) reached. Please wait before trying again.", max_retries + 1);
                                    let notification = NotificationMessage::new(
                                        final_error.clone(),
                                        NotificationType::Error,
                                    ).with_duration(8000);
                                    add_notification_clone.emit(notification);
                                    break Err(final_error);
                                } else {
                                    // Show notification for other errors
                                    let notification = NotificationMessage::new(
                                        format!("API Error: {}", error),
                                        NotificationType::Error,
                                    ).with_duration(6000);
                                    add_notification_clone.emit(notification);
                                    break Err(error);
                                }
                            }
                        }
                    }
                };

                match api_result {
                    Ok(response) => {
                        log!("✅ LLM API response received!");
                        log!("📊 Response details:");
                        log!("  - Function calls: {}", response.function_calls.len());
                        log!("  - Content length: {}", response.content.as_ref().map(|c| c.len()).unwrap_or(0));
                        
                        // Send response back to playground for handling
                        on_llm_response_clone.emit(response);
                    }
                    Err(_error) => {
                        // Error already handled above with notifications
                        log!("❌ API error occurred");
                    }
                }

                is_loading_clone.set(false);
            });
        })
    };

    (send_to_llm, is_loading)
}