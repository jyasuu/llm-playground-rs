// Updated LLM Playground with flexible provider system
use gloo_console::log;
use gloo_storage::{LocalStorage, Storage};
use gloo_timers::future::TimeoutFuture;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::llm_playground::{
    components::notification::{use_notifications, NotificationContainer, NotificationMessage, NotificationType},
    flexible_client::FlexibleLLMClient,
    mcp_client::McpClient,
    ChatHeader, ChatRoom, ChatSession, FlexibleApiConfig, FlexibleSettingsPanel, InputBar,
    ModelSelector, Sidebar, Message, MessageRole,
};

const STORAGE_KEY_FLEXIBLE_CONFIG: &str = "llm_playground_flexible_config";
const STORAGE_KEY_SESSIONS: &str = "llm_playground_sessions";
const STORAGE_KEY_CURRENT_SESSION: &str = "llm_playground_current_session";
const STORAGE_KEY_DARK_MODE: &str = "llm_playground_dark_mode";


#[function_component(FlexibleLLMPlayground)]
pub fn flexible_llm_playground() -> Html {
    // State management
    let sessions = use_state(|| HashMap::<String, ChatSession>::new());
    let current_session_id = use_state(|| Option::<String>::None);
    let api_config = use_state(|| FlexibleApiConfig::default());
    let show_settings = use_state(|| false);
    let show_model_selector = use_state(|| false);
    let dark_mode = use_state(|| false);
    let current_message = use_state(|| String::new());
    let llm_client = use_state(|| FlexibleLLMClient::new());
    let mcp_client = use_state(|| Option::<McpClient>::None);

    // New state-driven message flow
    let is_loading = use_state(|| false);
    let send_message_trigger = use_state(|| false);
    let function_call_trigger = use_state(|| Option::<serde_json::Value>::None);

    // Notification system
    let (notifications, add_notification, dismiss_notification) = use_notifications();

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
        let sessions = sessions.clone();
        let current_session_id = current_session_id.clone();
        let api_config = api_config.clone();
        let mcp_client = mcp_client.clone();
        let send_message_trigger = send_message_trigger.clone();

        use_effect_with(function_call_trigger.clone(), move |trigger_data| {
            if let Some(function_calls_json) = trigger_data.as_ref() {
                log!("🔧 Function call trigger activated");
                function_call_trigger.set(None); // Reset trigger
                
                if let Some(session_id) = current_session_id.as_ref() {
                    if let Ok(function_calls) = serde_json::from_value::<Vec<serde_json::Value>>(function_calls_json.clone()) {
                        let sessions_clone = sessions.clone();
                        let current_session_id_clone = current_session_id.clone();
                        let api_config_clone = api_config.clone();
                        let mcp_client_clone = mcp_client.clone();
                        let send_message_trigger_clone = send_message_trigger.clone();
                        let session_id_str = session_id.clone();

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
                                    let mut updated_sessions = (*sessions_clone).clone();
                                    if let Some(session) = updated_sessions.get_mut(&session_id_str) {
                                        session.messages.push(function_response_message);
                                        session.updated_at = js_sys::Date::now();
                                    }
                                    sessions_clone.set(updated_sessions);
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
        let sessions = sessions.clone();
        let current_session_id = current_session_id.clone();
        let api_config = api_config.clone();
        let llm_client = llm_client.clone();
        let function_call_trigger = function_call_trigger.clone();
        let add_notification = add_notification.clone();

        use_effect_with(send_message_trigger.clone(), move |trigger| {
            if **trigger {
                log!("🚀 Send message trigger activated");
                send_message_trigger.set(false); // Reset trigger
                
                if let Some(session_id) = current_session_id.as_ref() {
                    if let Some(session) = sessions.get(session_id) {
                        if !session.messages.is_empty() {
                            is_loading.set(true);
                            
                            let messages = session.messages.clone();
                            let config = (*api_config).clone();
                            let client = (*llm_client).clone();
                            let is_loading_clone = is_loading.clone();
                            let add_notification_clone = add_notification.clone();
                            let sessions_clone = sessions.clone();
                            let function_call_trigger_clone = function_call_trigger.clone();
                            let session_id_str = session_id.clone();

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
                                    log!("🔍 flexible_playground::send_message - Provider: {}, Model: {}", &provider_name, &model_name);
                                    
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
                                        
                                        // Handle LLM response directly here
                                        let mut new_sessions = (*sessions_clone).clone();
                                        if let Some(session) = new_sessions.get_mut(&session_id_str) {
                                            // Add assistant message
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
                                                        session.messages.push(assistant_message);
                                                        session.updated_at = js_sys::Date::now();
                                                    }
                                                }
                                                sessions_clone.set(new_sessions);
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
                                                session.messages.push(assistant_message);
                                                session.updated_at = js_sys::Date::now();
                                                sessions_clone.set(new_sessions);

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
            }
            || ()
        });
    }

    // Auto-initialize MCP connections when MCP config changes
    // Use a separate state to track MCP config changes to avoid infinite loops
    let mcp_config_hash = use_state(|| 0u64);

    {
        let api_config = api_config.clone();
        let mcp_client = mcp_client.clone();
        let mcp_config_hash = mcp_config_hash.clone();

        use_effect_with(api_config.clone(), move |config| {
            let mcp_config = config.mcp_config.clone();

            // Calculate a simple hash of the MCP config to detect actual changes
            let config_str = serde_json::to_string(&mcp_config).unwrap_or_default();
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            config_str.hash(&mut hasher);
            let new_hash = hasher.finish();

            // Only initialize if the MCP config actually changed
            if new_hash != *mcp_config_hash {
                mcp_config_hash.set(new_hash);

                let mcp_client = mcp_client.clone();
                let has_enabled_servers = mcp_config.servers.values().any(|server| server.enabled);

                if has_enabled_servers {
                    // Check if we already have a client with the same config to avoid re-initialization
                    let needs_initialization = if let Some(current_client) = mcp_client.as_ref() {
                        // Compare current client config with new config
                        let current_config_str =
                            serde_json::to_string(current_client.get_config()).unwrap_or_default();
                        current_config_str != config_str
                    } else {
                        true
                    };

                    if needs_initialization {
                        wasm_bindgen_futures::spawn_local(async move {
                            let mut client = McpClient::new(mcp_config);
                            match client.initialize().await {
                                Ok(_) => {
                                    log!("MCP client initialized successfully in background");
                                    mcp_client.set(Some(client));
                                }
                                Err(e) => {
                                    log!("Failed to initialize MCP client:", &e);
                                    mcp_client.set(None);
                                }
                            }
                        });
                    }
                } else {
                    mcp_client.set(None);
                }
            }

            || ()
        });
    }

    // Auto-update function tools when MCP client changes
    // Use a flag to prevent infinite loops when updating tools
    let updating_mcp_tools = use_state(|| false);

    {
        let api_config = api_config.clone();
        let mcp_client = mcp_client.clone();
        let updating_mcp_tools = updating_mcp_tools.clone();

        use_effect_with(mcp_client.clone(), move |client| {
            if !*updating_mcp_tools {
                if let Some(mcp_client) = client.as_ref() {
                    updating_mcp_tools.set(true);
                    let mcp_tools = mcp_client.get_function_tools();
                    let mut new_config = (*api_config).clone();
                    new_config.add_mcp_tools(mcp_tools);
                    api_config.set(new_config);
                                
                    let (provider_name, model_name) = api_config.get_current_provider_and_model();
                    log!("🔍 playground::use_effect_with mcp_client - Provider: {}, Model: {}", &provider_name, &model_name);
                    
                    log!("Added MCP tools to function tools list");

                    // Reset the flag after a brief delay to allow config update to complete
                    wasm_bindgen_futures::spawn_local({
                        let updating_mcp_tools = updating_mcp_tools.clone();
                        async move {
                            TimeoutFuture::new(100).await;
                            updating_mcp_tools.set(false);
                        }
                    });
                }
            }
            || ()
        });
    }

    // Load data from localStorage on mount
    {
        let sessions = sessions.clone();
        let current_session_id = current_session_id.clone();
        let api_config = api_config.clone();
        let dark_mode = dark_mode.clone();

        use_effect_with((), move |_| {
            // Load API config only if not already set (to avoid overriding session-specific settings)
            if let Ok(config_str) = LocalStorage::get::<String>(STORAGE_KEY_FLEXIBLE_CONFIG) {
                if let Ok(loaded_config) = serde_json::from_str::<FlexibleApiConfig>(&config_str) {
                    // Only load if current config is still default (hasn't been modified)
                    let current_config = (*api_config).clone();
                    if current_config.current_session_provider.is_none() {
                        api_config.set(loaded_config);
                        
                        let (provider_name, model_name) = api_config.get_current_provider_and_model();
                        
                        // Debug logging
                        use gloo_console::log;
                        log!("🔍 playground::use_effect_with - Provider: {}, Model: {}", &provider_name, &model_name);
                        
                    }
                }
            }

            // Load sessions
            if let Ok(sessions_str) = LocalStorage::get::<String>(STORAGE_KEY_SESSIONS) {
                if let Ok(loaded_sessions) =
                    serde_json::from_str::<HashMap<String, ChatSession>>(&sessions_str)
                {
                    for session in loaded_sessions.iter()
                    {
                        log!(format!("🗨️ Session {} now has {} messages", session.0, session.1.messages.len()));
                    }
                    sessions.set(loaded_sessions);
                }
            }

            // Load current session
            if let Ok(session_id) = LocalStorage::get::<String>(STORAGE_KEY_CURRENT_SESSION) {
                current_session_id.set(Some(session_id));
            }

            // Load dark mode
            if let Ok(dark) = LocalStorage::get::<bool>(STORAGE_KEY_DARK_MODE) {
                dark_mode.set(dark);

                // Apply dark mode class immediately on load
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        if let Some(html_element) = document.document_element() {
                            let class_list = html_element.class_list();
                            if dark {
                                let _ = class_list.add_1("dark");
                            } else {
                                let _ = class_list.remove_1("dark");
                            }
                        }
                    }
                }
            }

            || ()
        });
    }

    // Save to localStorage when state changes
    {
        let api_config = api_config.clone();
        use_effect_with(api_config.clone(), move |config| {
            if let Ok(config_str) = serde_json::to_string(&**config) {
                let _ = LocalStorage::set(STORAGE_KEY_FLEXIBLE_CONFIG, config_str);
            }
            || ()
        });
    }

    {
        let sessions = sessions.clone();
        use_effect_with(sessions.clone(), move |sessions| {
            if let Ok(sessions_str) = serde_json::to_string(&**sessions) {
                let _ = LocalStorage::set(STORAGE_KEY_SESSIONS, sessions_str);
            }
            || ()
        });
    }

    {
        let current_session_id = current_session_id.clone();
        use_effect_with(current_session_id.clone(), move |session_id| {
            if let Some(id) = session_id.as_ref() {
                let _ = LocalStorage::set(STORAGE_KEY_CURRENT_SESSION, id.clone());
            }
            || ()
        });
    }

    {
        let dark_mode = dark_mode.clone();
        use_effect_with(dark_mode.clone(), move |dark| {
            let _ = LocalStorage::set(STORAGE_KEY_DARK_MODE, **dark);

            // Apply dark mode class to document
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(html_element) = document.document_element() {
                        let class_list = html_element.class_list();
                        if **dark {
                            let _ = class_list.add_1("dark");
                        } else {
                            let _ = class_list.remove_1("dark");
                        }
                    }
                }
            }
            || ()
        });
    }

    // Session management callbacks
    let create_new_session = {
        let show_model_selector = show_model_selector.clone();
        Callback::from(move |_| {
            show_model_selector.set(true);
        })
    };

    let on_model_selected = {
        let sessions = sessions.clone();
        let current_session_id = current_session_id.clone();
        let api_config = api_config.clone();
        let show_model_selector = show_model_selector.clone();
        Callback::from(move |(provider_name, model_name): (String, String)| {
            log!("🆕 Creating new session with provider: {}, model: {}", &provider_name, &model_name);
            
            let session_id = format!("session_{}", js_sys::Date::now() as u64);
            let session_title = format!("{} - {}", &provider_name, &model_name);

            let new_session = ChatSession {
                id: session_id.clone(),
                title: session_title,
                messages: vec![],
                created_at: js_sys::Date::now(),
                updated_at: js_sys::Date::now(),
                pinned: false,
            };

            // Update API config with selected provider/model for this session
            let mut new_config = (*api_config).clone();
            log!("🔧 Setting session provider to: {},{}", &provider_name, &model_name);
            new_config.set_session_provider(&provider_name, &model_name);
            log!("🔧 New config session provider:", format!("{:?}", &new_config.current_session_provider));
            
            // Set the config state and ensure it's applied immediately
            api_config.set(new_config.clone());
    
            let (provider_name, model_name) = api_config.get_current_provider_and_model();
            log!("🔍 playground::on_model_selected - Provider: {}, Model: {}", &provider_name, &model_name);
            
            
            // Force a re-render to ensure the config update propagates
            // This ensures the hook gets the updated config
            log!("🔧 Config updated with session provider");

            // Add session and set as current
            let mut new_sessions = (*sessions).clone();
            new_sessions.insert(session_id.clone(), new_session);
            for session in new_sessions.iter()
            {
                log!(format!("🗨️ Session {} now has {} messages", session.0, session.1.messages.len()));
            }
            sessions.set(new_sessions);
            current_session_id.set(Some(session_id.clone()));
            log!("🔧 Set current session to: {}", &session_id);
            show_model_selector.set(false);
        })
    };

    let on_model_selector_cancel = {
        let show_model_selector = show_model_selector.clone();
        Callback::from(move |_: ()| {
            show_model_selector.set(false);
        })
    };

    let switch_session = {
        let current_session_id = current_session_id.clone();
        Callback::from(move |session_id: String| {
            current_session_id.set(Some(session_id));
        })
    };

    let delete_session = {
        let sessions = sessions.clone();
        let current_session_id = current_session_id.clone();
        Callback::from(move |session_id: String| {
            let mut new_sessions = (*sessions).clone();
            new_sessions.remove(&session_id);
            for session in new_sessions.iter()
            {
                log!(format!("🗨️ Session {} now has {} messages", session.0, session.1.messages.len()));
            }
            sessions.set(new_sessions);

            // If we're deleting the current session, clear current session
            if current_session_id.as_ref() == Some(&session_id) {
                current_session_id.set(None);
            }
        })
    };

    let toggle_pin_session = {
        let sessions = sessions.clone();
        Callback::from(move |session_id: String| {
            let mut new_sessions = (*sessions).clone();
            if let Some(session) = new_sessions.get_mut(&session_id) {
                session.pinned = !session.pinned;
                for session in new_sessions.iter()
                {
                    log!(format!("🗨️ Session {} now has {} messages", session.0, session.1.messages.len()));
                }
                sessions.set(new_sessions);
            }
        })
    };

    let clear_current_session = {
        let sessions = sessions.clone();
        let current_session_id = current_session_id.clone();
        Callback::from(move |_: ()| {
            if let Some(session_id) = current_session_id.as_ref() {
                let mut new_sessions = (*sessions).clone();
                if let Some(session) = new_sessions.get_mut(session_id) {
                    session.messages.clear();
                    session.updated_at = js_sys::Date::now();
                    for session in new_sessions.iter()
                    {
                        log!(format!("🗨️ Session {} now has {} messages", session.0, session.1.messages.len()));
                    }
                    sessions.set(new_sessions);
                }
            }
        })
    };

    // Settings management
    let toggle_settings = {
        let show_settings = show_settings.clone();
        Callback::from(move |_| {
            show_settings.set(!*show_settings);
        })
    };

    let save_settings = {
        let api_config = api_config.clone();
        let show_settings = show_settings.clone();
        Callback::from(move |config: FlexibleApiConfig| {
            api_config.set(config);
    
            let (provider_name, model_name) = api_config.get_current_provider_and_model();
            
            // Debug logging
            use gloo_console::log;
            log!("🔍 playground::save_settings - Provider: {}, Model: {}", &provider_name, &model_name);
            
            show_settings.set(false);
        })
    };

    let on_mcp_client_change = {
        let mcp_client = mcp_client.clone();
        Callback::from(move |client: Option<McpClient>| {
            mcp_client.set(client);
        })
    };

    let close_settings = {
        let show_settings = show_settings.clone();
        Callback::from(move |_| {
            show_settings.set(false);
        })
    };

    // Dark mode toggle
    let toggle_dark_mode = {
        let dark_mode = dark_mode.clone();
        Callback::from(move |_| {
            dark_mode.set(!*dark_mode);
        })
    };


    // Handle user message submission
    let send_message_ui = {
        let sessions = sessions.clone();
        let current_session_id = current_session_id.clone();
        let current_message = current_message.clone();
        let send_message_trigger = send_message_trigger.clone();
        Callback::from(move |_: ()| {
            let message_content = (*current_message).clone();
            if !message_content.trim().is_empty() && current_session_id.is_some() {
                let session_id = current_session_id.as_ref().unwrap();
                
                // Create user message
                let user_message = Message {
                    id: format!("user_{}", js_sys::Date::now() as u64),
                    role: MessageRole::User,
                    content: message_content.clone(),
                    timestamp: js_sys::Date::now(),
                    function_call: None,
                    function_response: None,
                };

                log!("🔍 playground::send_message_ui - Adding user message and triggering send");
                
                // Add user message to session
                let mut new_sessions = (*sessions).clone();
                if let Some(session) = new_sessions.get_mut(session_id) {
                    session.messages.push(user_message);
                    session.updated_at = js_sys::Date::now();
                }
                for session in new_sessions.iter()
                {
                    log!(format!("🗨️ Session {} now has {} messages", session.0, session.1.messages.len()));
                }
                sessions.set(new_sessions);
                current_message.set(String::new());
                
                // Trigger LLM send
                send_message_trigger.set(true);
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

    // Create a legacy API config for components that still need it
    let create_legacy_api_config = |flexible_config: &FlexibleApiConfig| {
        let (provider_name, model_name) = flexible_config.get_current_provider_and_model();

        if let Some(provider) = flexible_config.get_provider(&provider_name) {
            if provider.transformer.r#use.contains(&"gemini".to_string()) {
                crate::llm_playground::ApiConfig {
                    current_provider: crate::llm_playground::ApiProvider::Gemini,
                    gemini: crate::llm_playground::GeminiConfig {
                        api_key: provider.api_key.clone(),
                        model: model_name.clone(),
                    },
                    openai: crate::llm_playground::OpenAIConfig {
                        base_url: "".to_string(),
                        api_key: "".to_string(),
                        model: "".to_string(),
                    },
                    shared_settings: crate::llm_playground::types::SharedSettings {
                        temperature: flexible_config.shared_settings.temperature,
                        max_tokens: flexible_config.shared_settings.max_tokens,
                        retry_delay: flexible_config.shared_settings.retry_delay,
                    },
                    system_prompt: flexible_config.system_prompt.clone(),
                    function_tools: flexible_config.function_tools.clone(),
                    structured_outputs: flexible_config.structured_outputs.clone(),
                    mcp_config: crate::llm_playground::mcp_client::McpConfig::default(),
                }
            } else {
                crate::llm_playground::ApiConfig {
                    current_provider: crate::llm_playground::ApiProvider::OpenAI,
                    gemini: crate::llm_playground::GeminiConfig {
                        api_key: "".to_string(),
                        model: "".to_string(),
                    },
                    openai: crate::llm_playground::OpenAIConfig {
                        base_url: provider.api_base_url.clone(),
                        api_key: provider.api_key.clone(),
                        model: model_name.clone(),
                    },
                    shared_settings: crate::llm_playground::types::SharedSettings {
                        temperature: flexible_config.shared_settings.temperature,
                        max_tokens: flexible_config.shared_settings.max_tokens,
                        retry_delay: flexible_config.shared_settings.retry_delay,
                    },
                    system_prompt: flexible_config.system_prompt.clone(),
                    function_tools: flexible_config.function_tools.clone(),
                    structured_outputs: flexible_config.structured_outputs.clone(),
                    mcp_config: crate::llm_playground::mcp_client::McpConfig::default(),
                }
            }
        } else {
            crate::llm_playground::ApiConfig::default()
        }
    };

    // Get current session
    let current_session = current_session_id
        .as_ref()
        .and_then(|id| sessions.get(id))
        .cloned();

    // Get current provider info for display
    let current_provider_info = {
        let (provider_name, model_name) = api_config.get_current_provider_and_model();
        format!("{} - {}", provider_name, model_name)
    };

    html! {
        <div class={classes!("flex", "h-screen", "overflow-hidden", if *dark_mode { "dark" } else { "" })}>
            <div class="flex h-full w-full bg-gray-50 dark:bg-gray-900">
                // Sidebar
                <Sidebar
                    sessions={(*sessions).clone()}
                    current_session_id={(*current_session_id).clone()}
                    on_new_session={create_new_session.clone()}
                    on_select_session={switch_session}
                    on_delete_session={delete_session}
                    on_toggle_settings={toggle_settings}
                />

                // Main content area
                <div class="flex-1 flex flex-col min-w-0">
                    {if let Some(session) = current_session {
                        html! {
                            <>
                                <ChatHeader
                                    current_session={Some(session.clone())}
                                    api_config={create_legacy_api_config(&*api_config)}
                                    on_toggle_dark_mode={toggle_dark_mode}
                                    dark_mode={*dark_mode}
                                />
                                <ChatRoom
                                    session={Some(session.clone())}
                                    is_loading={*is_loading}
                                />
                                <InputBar
                                    current_message={(*current_message).clone()}
                                    is_loading={*is_loading}
                                    on_send_message={send_message_ui}
                                    on_message_change={create_input_event_callback(update_message)}
                                />
                            </>
                        }
                    } else {
                        html! {
                            <div class="flex-1 flex items-center justify-center bg-white dark:bg-gray-800">
                                <div class="text-center">
                                    <div class="mb-4">
                                        <i class="fas fa-comments text-6xl text-gray-300 dark:text-gray-600"></i>
                                    </div>
                                    <h2 class="text-2xl font-semibold text-gray-600 dark:text-gray-300 mb-2">
                                        {"Welcome to LLM Playground"}
                                    </h2>
                                    <p class="text-gray-500 dark:text-gray-400 mb-6">
                                        {"Start a new conversation by selecting a model"}
                                    </p>
                                    <button
                                        onclick={
                                            let create_new_session = create_new_session.clone();
                                            Callback::from(move |_| create_new_session.emit(()))
                                        }
                                        class="bg-primary-600 hover:bg-primary-700 text-white px-6 py-3 rounded-lg font-medium transition-colors"
                                    >
                                        {"New Session"}
                                    </button>
                                </div>
                            </div>
                        }
                    }}
                </div>

                // Settings panel
                {if *show_settings {
                    html! {
                        <FlexibleSettingsPanel
                            config={(*api_config).clone()}
                            on_save={save_settings}
                            on_close={close_settings}
                            mcp_client={(*mcp_client).clone()}
                            on_mcp_client_change={on_mcp_client_change}
                        />
                    }
                } else {
                    html! {}
                }}

                // Model selector modal
                <ModelSelector
                    config={(*api_config).clone()}
                    on_select={on_model_selected}
                    on_cancel={on_model_selector_cancel}
                    show={*show_model_selector}
                />

                // Notification container
                <NotificationContainer
                    notifications={notifications}
                    on_dismiss={dismiss_notification}
                />
            </div>
        </div>
    }
}
