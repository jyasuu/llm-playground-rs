use gloo_console::log;
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::llm_playground::{
    components::notification::{NotificationMessage, NotificationType},
    flexible_client::FlexibleLLMClient,
    mcp_client::McpClient,
    ChatSession, FlexibleApiConfig, Message, MessageRole,
};

use super::{ChatRoom as ChatRoomDisplay, InputBar};

#[derive(Properties, PartialEq)]
pub struct ChatroomProps {
    /// Current session being displayed
    pub session: Option<ChatSession>,
    /// API configuration for LLM calls
    pub api_config: FlexibleApiConfig,
    /// LLM client instance
    pub llm_client: FlexibleLLMClient,
    /// Optional MCP client for function tools
    pub mcp_client: Option<McpClient>,
    /// Callback when session is updated (for persistence)
    pub on_session_update: Callback<ChatSession>,
    /// Callback for notifications
    pub on_notification: Callback<NotificationMessage>,
}

#[function_component(Chatroom)]
pub fn chatroom(props: &ChatroomProps) -> Html {
    // Local state for current message input
    let current_message = use_state(|| String::new());
    let is_loading = use_state(|| false);
    
    // State-driven message flow triggers
    let send_message_trigger = use_state(|| false);
    let function_call_trigger = use_state(|| Option::<serde_json::Value>::None);

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

    // Function call execution effect
    {
        let function_call_trigger = function_call_trigger.clone();
        let session = props.session.clone();
        let api_config = props.api_config.clone();
        let mcp_client = props.mcp_client.clone();
        let send_message_trigger = send_message_trigger.clone();
        let on_session_update = props.on_session_update.clone();

        use_effect_with(function_call_trigger.clone(), move |trigger_data| {
            if let Some(function_calls_json) = trigger_data.as_ref() {
                log!("🔧 Function call trigger activated");
                function_call_trigger.set(None); // Reset trigger
                
                if let Some(mut current_session) = session {
                    if let Ok(function_calls) = serde_json::from_value::<Vec<serde_json::Value>>(function_calls_json.clone()) {
                        let on_session_update_clone = on_session_update.clone();
                        let api_config_clone = api_config.clone();
                        let mcp_client_clone = mcp_client.clone();
                        let send_message_trigger_clone = send_message_trigger.clone();

                        wasm_bindgen_futures::spawn_local(async move {
                            // Execute all function calls
                            for function_call_json in &function_calls {
                                if let (Some(name), Some(id), Some(arguments)) = (
                                    function_call_json.get("name").and_then(|v| v.as_str()),
                                    function_call_json.get("id").and_then(|v| v.as_str()),
                                    function_call_json.get("arguments")
                                ) {
                                    log!("🔧 Executing function: {} (ID: {})", name, id);
                                    
                                    // Execute function call
                                    let response_value = if let Some(tool) = api_config_clone
                                        .function_tools
                                        .iter()
                                        .find(|tool| tool.name == name)
                                    {
                                        if tool.is_builtin {
                                            // Execute built-in tool
                                            match crate::llm_playground::builtin_tools::execute_builtin_tool(
                                                name, 
                                                arguments, 
                                                mcp_client_clone.as_ref()
                                            ).await {
                                                Ok(result) => result,
                                                Err(error) => serde_json::json!({"error": error}),
                                            }
                                        } else {
                                            // Use mock response
                                            serde_json::from_str(&tool.mock_response)
                                                .unwrap_or_else(|_| serde_json::json!({"result": tool.mock_response.clone()}))
                                        }
                                    } else {
                                        serde_json::json!({"error": "Unknown function tool"})
                                    };

                                    // Add function response message
                                    let function_response_message = Message {
                                        id: format!("msg_fr_{}", js_sys::Date::now() as u64),
                                        role: MessageRole::Function,
                                        content: format!("Function {} executed", name),
                                        timestamp: js_sys::Date::now(),
                                        function_call: None,
                                        function_response: Some(serde_json::json!({
                                            "id": id,
                                            "name": name,
                                            "response": response_value
                                        })),
                                    };
                                    
                                    // Update session with function response
                                    current_session.messages.push(function_response_message);
                                    current_session.updated_at = js_sys::Date::now();
                                    
                                    // Notify parent of session update
                                    on_session_update_clone.emit(current_session.clone());
                                }
                            }
                            
                            // Trigger next LLM call after all function executions are complete
                            log!("🔄 All functions executed, triggering next LLM call");
                            send_message_trigger_clone.set(true);
                        });
                    }
                }
            }
            || ()
        });
    }

    // State-driven LLM message sending effect
    {
        let send_message_trigger = send_message_trigger.clone();
        let is_loading = is_loading.clone();
        let session = props.session.clone();
        let api_config = props.api_config.clone();
        let llm_client = props.llm_client.clone();
        let function_call_trigger = function_call_trigger.clone();
        let on_notification = props.on_notification.clone();
        let on_session_update = props.on_session_update.clone();

        use_effect_with(send_message_trigger.clone(), move |trigger| {
            if **trigger {
                log!("🚀 Send message trigger activated");
                send_message_trigger.set(false); // Reset trigger
                
                if let Some(mut current_session) = session {
                    if !current_session.messages.is_empty() {
                        is_loading.set(true);
                        
                        let messages = current_session.messages.clone();
                        let config = api_config.clone();
                        let client = llm_client.clone();
                        let is_loading_clone = is_loading.clone();
                        let on_notification_clone = on_notification.clone();
                        let function_call_trigger_clone = function_call_trigger.clone();
                        let on_session_update_clone = on_session_update.clone();

                        wasm_bindgen_futures::spawn_local(async move {
                            log!("📤 Calling LLM API with {} messages...", messages.len());
                            for (i, msg) in messages.iter().enumerate() {
                                let role_str = match msg.role {
                                    MessageRole::User => "User",
                                    MessageRole::Assistant => "Assistant", 
                                    MessageRole::System => "System",
                                    MessageRole::Function => "Function",
                                };
                                log!("  Message {}: {} - {}", i + 1, role_str, 
                                    if msg.content.len() > 100 { &msg.content[..100] } else { &msg.content });
                            }

                            let mut retry_attempt = 0u32;
                            let max_retries = 3u32;

                            let api_result = loop {
                                log!("⏳ Attempting LLM API call (attempt {})...", retry_attempt + 1);
                                
                                let (provider_name, model_name) = config.get_current_provider_and_model();
                                log!("🔍 chatroom::send_message - Provider: {}, Model: {}", &provider_name, &model_name);
                                
                                match client.send_message(&messages, &config).await {
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
                                            on_notification_clone.emit(notification);

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
                                                on_notification_clone.emit(notification);
                                                break Err(final_error);
                                            } else {
                                                // Show notification for other errors
                                                let notification = NotificationMessage::new(
                                                    format!("API Error: {}", error),
                                                    NotificationType::Error,
                                                ).with_duration(6000);
                                                on_notification_clone.emit(notification);
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
                                    
                                    // Handle LLM response directly here
                                    if response.function_calls.is_empty() {
                                        // Regular text response - conversation ends here
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
                                                current_session.messages.push(assistant_message);
                                                current_session.updated_at = js_sys::Date::now();
                                            }
                                        }
                                        on_session_update_clone.emit(current_session);
                                    } else {
                                        // Function call response - trigger function execution
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
                                        current_session.messages.push(assistant_message);
                                        current_session.updated_at = js_sys::Date::now();
                                        on_session_update_clone.emit(current_session);

                                        // Trigger function call execution
                                        let function_calls_json = serde_json::json!(response
                                            .function_calls
                                            .iter()
                                            .map(|fc| {
                                                serde_json::json!({
                                                    "id": fc.id,
                                                    "name": fc.name,
                                                    "arguments": fc.arguments
                                                })
                                            })
                                            .collect::<Vec<_>>());
                                        
                                        log!("🔄 Triggering function call execution");
                                        function_call_trigger_clone.set(Some(function_calls_json));
                                    }
                                }
                                Err(_error) => {
                                    // Error already handled above with notifications
                                    log!("❌ API error occurred");
                                }
                            }

                            is_loading_clone.set(false);
                        });
                    }
                }
            }
            || ()
        });
    }

    // Handle user message submission
    let send_message = {
        let current_message = current_message.clone();
        let send_message_trigger = send_message_trigger.clone();
        let session = props.session.clone();
        let on_session_update = props.on_session_update.clone();
        
        Callback::from(move |_: ()| {
            let message_content = (*current_message).clone();
            if !message_content.trim().is_empty() {
                if let Some(mut current_session) = session.clone() {
                    // Create user message
                    let user_message = Message {
                        id: format!("user_{}", js_sys::Date::now() as u64),
                        role: MessageRole::User,
                        content: message_content.clone(),
                        timestamp: js_sys::Date::now(),
                        function_call: None,
                        function_response: None,
                    };

                    log!("🔍 chatroom::send_message - Adding user message and triggering send");
                    
                    // Add user message to session
                    current_session.messages.push(user_message);
                    current_session.updated_at = js_sys::Date::now();
                    
                    // Notify parent of session update
                    on_session_update.emit(current_session);
                    
                    // Clear input
                    current_message.set(String::new());
                    
                    // Trigger LLM send
                    send_message_trigger.set(true);
                }
            }
        })
    };

    let update_message = {
        let current_message = current_message.clone();
        Callback::from(move |message: String| {
            current_message.set(message);
        })
    };

    let create_input_event_callback = {
        let update_message = update_message.clone();
        move |callback: Callback<String>| {
            Callback::from(move |e: InputEvent| {
                if let Some(target) = e.target() {
                    if let Ok(input) = target.dyn_into::<web_sys::HtmlTextAreaElement>() {
                        callback.emit(input.value());
                    }
                }
            })
        }
    };

    html! {
        <>
            <ChatRoomDisplay
                session={props.session.clone()}
                is_loading={*is_loading}
            />
            <InputBar
                current_message={(*current_message).clone()}
                is_loading={*is_loading}
                on_send_message={send_message}
                on_message_change={create_input_event_callback(update_message)}
            />
        </>
    }
}