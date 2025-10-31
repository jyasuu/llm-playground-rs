// Hook for managing LLM chat interactions
use gloo_console::log;
use gloo_timers::future::TimeoutFuture;
use std::collections::HashMap;
use yew::prelude::*;

use crate::llm_playground::{
    components::notification::{NotificationMessage, NotificationType},
    unified_client::UnifiedLLMClient,
    mcp_client::McpClient,
    ChatSession, FlexibleApiConfig, Message, MessageRole,
};

/// Hook for managing LLM chat interactions
/// Returns: (send_message_callback, is_loading_state)
#[hook]
pub fn use_llm_chat(
    sessions: UseStateHandle<HashMap<String, ChatSession>>,
    current_session_id: UseStateHandle<Option<String>>,
    api_config: UseStateHandle<FlexibleApiConfig>,
    llm_client: UseStateHandle<UnifiedLLMClient>,
    mcp_client: UseStateHandle<Option<McpClient>>,
    add_notification: Callback<NotificationMessage>,
) -> (Callback<String>, UseStateHandle<bool>) {
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

    let send_message = {
        let sessions = sessions.clone();
        let current_session_id = current_session_id.clone();
        let is_loading = is_loading.clone();
        let api_config = api_config.clone();
        let llm_client = llm_client.clone();
        let mcp_client = mcp_client.clone();
        let add_notification = add_notification.clone();

        Callback::from(move |message_content: String| {
            let sessions = sessions.clone();
            let add_notification = add_notification.clone();
            
            if let Some(session_id) = current_session_id.as_ref() {
                if message_content.trim().is_empty() {
                    return;
                }

                let user_message = Message {
                    id: format!("user_{}", js_sys::Date::now() as u64),
                    role: MessageRole::User,
                    content: message_content.clone(),
                    timestamp: js_sys::Date::now(),
                    function_call: None,
                    function_response: None,
                };

                // Add user message to session
                let mut new_sessions = (*sessions).clone();
                if let Some(session) = new_sessions.get_mut(session_id) {
                    session.messages.push(user_message);
                    session.updated_at = js_sys::Date::now();
                }
                sessions.set(new_sessions.clone());
                is_loading.set(true);

                // Send to LLM
                let session_id_clone = session_id.clone();
                let config = (*api_config).clone();
                let client = (*llm_client).clone();
                let mcp_client = (*mcp_client).clone();
                let is_loading_clone = is_loading.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    if let Some(session) = new_sessions.get(&session_id_clone) {
                        // Create unified client from session messages
                        let mut unified_client = UnifiedLLMClient::from_legacy_messages(&session.messages);
                        
                        // Set system prompt in the unified client (it will handle provider-specific placement internally)
                        if !config.system_prompt.trim().is_empty() {
                            unified_client.set_system_prompt(&config.system_prompt);
                        }

                        // Handle function calls automatically with feedback loop
                        log!("🚀 Starting LLM conversation loop for session: {}", &session_id_clone);
                        let mut loop_iteration = 0;
                        loop {
                            loop_iteration += 1;
                            let conversation = unified_client.get_conversation();
                            log!("🔄 Loop iteration #{} - Unified client has {} messages", loop_iteration, conversation.messages.len());
                            let mut retry_attempt = 0u32;
                            let max_retries = 3u32;

                            log!("📤 Calling LLM API with unified client...");
                            unified_client.log_conversation_state();
                            
                            let api_result = loop {
                                log!("⏳ Attempting LLM API call (attempt {})...", retry_attempt + 1);
                                match unified_client.send_message(&config).await {
                                    Ok(response) => break Ok(response),
                                    Err(error) => {
                                        // Check if this is a retryable error (429 rate limit)
                                        if is_retryable_error(&error) && retry_attempt < max_retries
                                        {
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
                                            if is_retryable_error(&error)
                                                && retry_attempt >= max_retries
                                            {
                                                let final_error = format!("Rate limit exceeded. Max retries ({}) reached. Please wait before trying again.", max_retries + 1);
                                                let notification = NotificationMessage::new(
                                                    final_error.clone(),
                                                    NotificationType::Error,
                                                )
                                                .with_duration(8000);
                                                add_notification.emit(notification);
                                                break Err(final_error);
                                            } else {
                                                // Show notification for other errors
                                                let notification = NotificationMessage::new(
                                                    format!("API Error: {}", error),
                                                    NotificationType::Error,
                                                )
                                                .with_duration(6000);
                                                add_notification.emit(notification);
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
                                    if let Some(content) = &response.content {
                                        log!("  - Content preview: {:?}", &content[..50.min(content.len())]);
                                    }
                                    
                                    // If no function calls, this is a regular text response - add it and break
                                    if response.function_calls.is_empty() {
                                        log!("🏁 No function calls - this is a final text response, ending loop");
                                        if let Some(content) = &response.content {
                                            if !content.trim().is_empty() {
                                                // Add assistant message to unified client
                                                unified_client.add_assistant_message(response.content.clone(), Vec::new());

                                                // Update session with the unified conversation
                                                if let Some(session) = new_sessions.get_mut(&session_id_clone) {
                                                    session.messages = unified_client.to_legacy_messages();
                                                    session.updated_at = js_sys::Date::now();
                                                }
                                                sessions.set(new_sessions.clone());
                                            }
                                        }
                                        break;
                                    }

                                    log!("🔧 LLM requested {} function calls - processing them now...", response.function_calls.len());
                                    for (i, fc) in response.function_calls.iter().enumerate() {
                                        log!("  Function {}: {} with id: {}", i + 1, &fc.name, &fc.id);
                                    }

                                    // Convert function calls to unified format and add to client
                                    let unified_calls = unified_client.convert_function_calls_to_unified(response.function_calls.clone());
                                    unified_client.add_assistant_message(response.content.clone(), unified_calls.clone());

                                    // Update session immediately for display
                                    {
                                        if let Some(session) = new_sessions.get_mut(&session_id_clone) {
                                            session.messages = unified_client.to_legacy_messages();
                                            session.updated_at = js_sys::Date::now();
                                        }
                                        sessions.set(new_sessions.clone());
                                    }

                                    // Execute ALL function calls, then continue loop for LLM response
                                    log!("🛠️ Starting execution of {} function calls...", unified_calls.len());
                                    let mut function_results = Vec::new();
                                    
                                    for (func_index, function_call) in unified_calls.iter().enumerate() {
                                        log!("🔧 Executing function {}/{}: {} (ID: {})", 
                                            func_index + 1, 
                                            unified_calls.len(),
                                            &function_call.name, 
                                            &function_call.id);
                                        log!("📋 Function arguments: {}", 
                                            serde_json::to_string(&function_call.arguments).unwrap_or_else(|_| "invalid_args".to_string()));
                                        
                                        // Check if this is a built-in tool and execute it properly
                                        let response_value = if let Some(tool) = config
                                            .function_tools
                                            .iter()
                                            .find(|tool| tool.name == function_call.name)
                                        {
                                            if tool.is_builtin {
                                                // Execute built-in tool with real functionality (including MCP tools)
                                                log!("Executing built-in tool: {}", &function_call.name);
                                                match crate::llm_playground::builtin_tools::execute_builtin_tool(&function_call.name, &function_call.arguments, mcp_client.as_ref()).await {
                                                    Ok(result) => result,
                                                    Err(error) => serde_json::json!({"error": error}),
                                                }
                                            } else {
                                                // Use mock response for regular tools
                                                serde_json::from_str(&tool.mock_response)
                                                    .unwrap_or_else(|_| serde_json::json!({"result": tool.mock_response.clone()}))
                                            }
                                        } else {
                                            // Unknown tool
                                            serde_json::json!({"error": "Unknown function tool"})
                                        };

                                        log!("✅ Function {} execution completed", &function_call.name);
                                        log!("📤 Function result: {}", 
                                            serde_json::to_string(&response_value).unwrap_or_else(|_| "invalid_result".to_string()));

                                        function_results.push(response_value);
                                    }
                                    
                                    // Add all function responses to unified client
                                    let unified_responses = unified_client.create_function_responses(&unified_calls, function_results);
                                    unified_client.add_function_responses(unified_responses);

                                    // Update session with all the function responses
                                    {
                                        if let Some(session) = new_sessions.get_mut(&session_id_clone) {
                                            session.messages = unified_client.to_legacy_messages();
                                            session.updated_at = js_sys::Date::now();
                                        }
                                        sessions.set(new_sessions.clone());
                                    }

                                    log!("📝 All function responses added to unified client");
                                    
                                    log!("🔄 ALL function calls completed! Now continuing loop to trigger LLM response...");
                                    let conversation = unified_client.get_conversation();
                                    log!("📨 Next LLM call will include {} messages (including {} function responses)", 
                                        conversation.messages.len(), 
                                        unified_calls.len());
                                    // Continue the loop to send updated messages back to LLM
                                    // This will trigger another LLM call with the function result
                                }
                                Err(_error) => {
                                    // Error already handled above with notifications
                                    // Don't add error messages to chat history for retryable errors
                                    log!("❌ API error occurred, breaking out of loop");
                                    break;
                                }
                            }
                        }

                        log!("🏁 LLM conversation loop completed after {} iterations", loop_iteration);
                        is_loading_clone.set(false);
                    }
                });
            }
        })
    };

    (send_message, is_loading)
}