// LLM Playground module
pub mod components;
pub mod storage;
pub mod api_clients;
pub mod types;

pub use components::*;
pub use storage::*;
pub use api_clients::*;
pub use types::*;

use yew::prelude::*;
use web_sys::HtmlTextAreaElement;
use gloo_storage::{LocalStorage, Storage};
use std::collections::HashMap;
use gloo_console::log;
use crate::llm_playground::api_clients::{GeminiClient, OpenAIClient};

#[function_component(LLMPlayground)]
pub fn llm_playground() -> Html {
    // State management
    let sessions = use_state(|| HashMap::<String, ChatSession>::new());
    let current_session_id = use_state(|| Option::<String>::None);
    let api_config = use_state(|| ApiConfig::default());
    let show_settings = use_state(|| false);
    let dark_mode = use_state(|| false);
    let current_message = use_state(|| String::new());
    let is_loading = use_state(|| false);

    // Load data from localStorage on mount
    {
        let sessions = sessions.clone();
        let api_config = api_config.clone();
        let current_session_id = current_session_id.clone();
        
        use_effect_with(
            (),
            move |_| {
                // Load sessions from localStorage
                if let Ok(stored_sessions) = StorageManager::load_sessions() {
                    sessions.set(stored_sessions);
                }

                // Load API config from localStorage
                if let Ok(stored_config) = StorageManager::load_config() {
                    api_config.set(stored_config);
                }

                // Load current session ID
                if let Ok(stored_session_id) = StorageManager::load_current_session_id() {
                    current_session_id.set(Some(stored_session_id));
                }

                || ()
            },
        );
    }

    // Current session
    let current_session = {
        let sessions = sessions.clone();
        let current_session_id = current_session_id.clone();
        
        use_memo(
            ((*sessions).clone(), (*current_session_id).clone()),
            move |(sessions, current_session_id)| {
                current_session_id.as_ref()
                    .and_then(|id| sessions.get(id))
                    .cloned()
            },
        )
    };

    // Event handlers
    let toggle_settings = {
        let show_settings = show_settings.clone();
        Callback::from(move |_| {
            show_settings.set(!*show_settings);
        })
    };

    let close_settings = {
        let show_settings = show_settings.clone();
        Callback::from(move |_| {
            show_settings.set(false);
        })
    };

    let toggle_dark_mode = {
        let dark_mode = dark_mode.clone();
        Callback::from(move |_| {
            let new_dark_mode = !*dark_mode;
            dark_mode.set(new_dark_mode);
            
            // Update the document class for Tailwind dark mode
            if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                if let Some(html_element) = document.document_element() {
                    if new_dark_mode {
                        let _ = html_element.set_class_name("dark");
                    } else {
                        let _ = html_element.set_class_name("");
                    }
                }
            }
        })
    };

    let create_new_session = {
        let sessions = sessions.clone();
        let current_session_id = current_session_id.clone();
        
        Callback::from(move |_| {
            let new_session = ChatSession {
                id: format!("session_{}", js_sys::Date::now()),
                title: "New Chat".to_string(),
                messages: vec![],
                created_at: js_sys::Date::now(),
                updated_at: js_sys::Date::now(),
                pinned: false,
            };
            
            let mut new_sessions = (*sessions).clone();
            new_sessions.insert(new_session.id.clone(), new_session.clone());
            sessions.set(new_sessions.clone());
            current_session_id.set(Some(new_session.id.clone()));
            
            // Save to localStorage
            let _ = StorageManager::save_sessions(&new_sessions);
            let _ = StorageManager::save_current_session_id(&new_session.id);
        })
    };

    let select_session = {
        let current_session_id = current_session_id.clone();
        
        Callback::from(move |session_id: String| {
            current_session_id.set(Some(session_id.clone()));
            let _ = StorageManager::save_current_session_id(&session_id);
        })
    };

    let delete_session = {
        let sessions = sessions.clone();
        let current_session_id = current_session_id.clone();
        
        Callback::from(move |session_id: String| {
            let mut new_sessions = (*sessions).clone();
            new_sessions.remove(&session_id);
            
            // If we're deleting the current session, clear the current session
            if current_session_id.as_ref() == Some(&session_id) {
                current_session_id.set(None);
                let _ = StorageManager::save_current_session_id("");
            }
            
            sessions.set(new_sessions.clone());
            let _ = StorageManager::save_sessions(&new_sessions);
        })
    };

    let update_message_input = {
        let current_message = current_message.clone();
        Callback::from(move |e: InputEvent| {
            let textarea: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
            current_message.set(textarea.value());
        })
    };

    let send_message = {
        let sessions = sessions.clone();
        let current_session_id = current_session_id.clone();
        let current_message = current_message.clone();
        let _api_config = api_config.clone();
        let is_loading = is_loading.clone();
        
        Callback::from(move |_| {
            log!("Send button clicked!");
            log!("Current message:", &*current_message);
            log!("Is loading:", *is_loading);
            if let Some(ref session_id) = *current_session_id {
                log!("Current session ID:", session_id);
            } else {
                log!("No current session ID");
            }
            
            if current_message.trim().is_empty() || *is_loading {
                log!("Message is empty or loading, returning early");
                return;
            }
            
            if let Some(session_id) = current_session_id.as_ref() {
                let mut new_sessions = (*sessions).clone();
                if let Some(session) = new_sessions.get_mut(session_id) {
                    // Add user message
                    let user_message = Message {
                        id: format!("msg_{}", js_sys::Date::now()),
                        role: MessageRole::User,
                        content: (*current_message).clone(),
                        timestamp: js_sys::Date::now(),
                        function_call: None,
                        function_response: None,
                    };
                    
                    session.messages.push(user_message);
                    session.updated_at = js_sys::Date::now();
                    
                    // Update session title if this is the first message
                    if session.messages.len() == 1 {
                        session.title = current_message.chars().take(50).collect();
                    }
                    
                    // Log the message before borrowing issues
                    log!("Would send message to API:", &session.messages.last().unwrap().content);
                }
                
                // Get the updated session before async operations
                let updated_session = new_sessions.get(session_id).unwrap().clone();
                
                // Set state after mutations
                sessions.set(new_sessions.clone());
                current_message.set(String::new());
                is_loading.set(true);
                
                // Save to localStorage
                let _ = StorageManager::save_sessions(&new_sessions);
                
                // Make real API call
                let sessions_clone = sessions.clone();
                let session_id_clone = session_id.clone();
                let is_loading_clone = is_loading.clone();
                let api_config_clone = (*_api_config).clone();
                    
                wasm_bindgen_futures::spawn_local(async move {
                    let response_content = match api_config_clone.current_provider {
                        ApiProvider::Gemini => {
                            log!("Calling Gemini API...");
                            let client = GeminiClient::new();
                            
                            // Create messages including system prompt
                            let mut api_messages = vec![];
                            
                            // Add system message if exists
                            if !api_config_clone.system_prompt.trim().is_empty() {
                                api_messages.push(Message {
                                    id: "system".to_string(),
                                    role: MessageRole::System,
                                    content: api_config_clone.system_prompt.clone(),
                                    timestamp: js_sys::Date::now(),
                                    function_call: None,
                                    function_response: None,
                                });
                            }
                            
                            // Add conversation history from the updated session
                            api_messages.extend(updated_session.messages.clone());
                                
                            // Handle function calls automatically with feedback loop
                            let mut final_response = String::new();
                            let mut current_messages = api_messages.clone();
                            let mut max_iterations = 5; // Prevent infinite loops
                            
                            loop {
                                match client.send_message(&current_messages, &api_config_clone).await {
                                    Ok(response) => {
                                        // Add any text content to final response
                                        if let Some(content) = &response.content {
                                            if !final_response.is_empty() {
                                                final_response.push_str("\n\n");
                                            }
                                            final_response.push_str(content);
                                        }
                                        
                                        // If no function calls, we're done
                                        if response.function_calls.is_empty() {
                                            break;
                                        }
                                        
                                        // Process function calls
                                        if !final_response.is_empty() {
                                            final_response.push_str("\n\n");
                                        }
                                        
                                        // Add assistant message with function calls to conversation
                                        let assistant_message = Message {
                                            id: format!("msg_fc_{}", js_sys::Date::now()),
                                            role: MessageRole::Assistant,
                                            content: response.content.unwrap_or_default(),
                                            timestamp: js_sys::Date::now(),
                                            function_call: if let Some(fc) = response.function_calls.first() {
                                                Some(serde_json::json!({
                                                    "name": fc.name,
                                                    "args": fc.arguments
                                                }))
                                            } else { None },
                                            function_response: None,
                                        };
                                        current_messages.push(assistant_message);
                                        
                                        // Execute each function call and add responses
                                        for function_call in &response.function_calls {
                                            // Find mock response from config
                                            let mock_response = api_config_clone
                                                .function_tools
                                                .iter()
                                                .find(|tool| tool.name == function_call.name)
                                                .map(|tool| tool.mock_response.clone())
                                                .unwrap_or_else(|| r#"{"result": "Function executed successfully"}"#.to_string());
                                            
                                            // Parse mock response as JSON
                                            let response_value = serde_json::from_str(&mock_response)
                                                .unwrap_or_else(|_| serde_json::json!({"result": mock_response}));
                                            
                                            // Add function response message to conversation
                                            let function_response_message = Message {
                                                id: format!("msg_fr_{}", js_sys::Date::now()),
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
                                            current_messages.push(function_response_message);
                                            
                                            // Add to display
                                            final_response.push_str(&format!(
                                                "🔧 **Function**: `{}` → {}",
                                                function_call.name,
                                                serde_json::to_string(&response_value).unwrap_or_else(|_| "Invalid response".to_string())
                                            ));
                                            if function_call != response.function_calls.last().unwrap() {
                                                final_response.push_str("\n");
                                            }
                                        }
                                        
                                        // Check iteration limit
                                        max_iterations -= 1;
                                        if max_iterations <= 0 {
                                            final_response.push_str("\n\n⚠️ Maximum function call iterations reached");
                                            break;
                                        }
                                    },
                                    Err(error) => {
                                        log!("Gemini API error:", &error);
                                        if final_response.is_empty() {
                                            final_response = format!("❌ **API Error**: {}", error);
                                        } else {
                                            final_response.push_str(&format!("\n\n❌ **API Error**: {}", error));
                                        }
                                        break;
                                    }
                                }
                            }
                            
                            if final_response.is_empty() {
                                "No response from API".to_string()
                            } else {
                                final_response
                            }
                        },
                        ApiProvider::OpenAI => {
                            log!("Calling OpenAI API...");
                            let client = OpenAIClient::new();
                            
                            // Create messages including system prompt
                            let mut api_messages = vec![];
                            
                            // Add system message if exists
                            if !api_config_clone.system_prompt.trim().is_empty() {
                                api_messages.push(Message {
                                    id: "system".to_string(),
                                    role: MessageRole::System,
                                    content: api_config_clone.system_prompt.clone(),
                                    timestamp: js_sys::Date::now(),
                                    function_call: None,
                                    function_response: None,
                                });
                            }
                            
                            // Add conversation history from the updated session
                            api_messages.extend(updated_session.messages.clone());
                            
                            // Handle function calls automatically with feedback loop
                            let mut final_response = String::new();
                            let mut current_messages = api_messages.clone();
                            let mut max_iterations = 5; // Prevent infinite loops
                            
                            loop {
                                match client.send_message(&current_messages, &api_config_clone).await {
                                    Ok(response) => {
                                        // Add any text content to final response
                                        if let Some(content) = &response.content {
                                            if !final_response.is_empty() {
                                                final_response.push_str("\n\n");
                                            }
                                            final_response.push_str(content);
                                        }
                                        
                                        // If no function calls, we're done
                                        if response.function_calls.is_empty() {
                                            break;
                                        }
                                        
                                        // Process function calls
                                        if !final_response.is_empty() {
                                            final_response.push_str("\n\n");
                                        }
                                        
                                        // Add assistant message with function calls to conversation
                                        let assistant_message = Message {
                                            id: format!("msg_fc_{}", js_sys::Date::now()),
                                            role: MessageRole::Assistant,
                                            content: response.content.unwrap_or_default(),
                                            timestamp: js_sys::Date::now(),
                                            function_call: if let Some(fc) = response.function_calls.first() {
                                                Some(serde_json::json!({
                                                    "name": fc.name,
                                                    "args": fc.arguments
                                                }))
                                            } else { None },
                                            function_response: None,
                                        };
                                        current_messages.push(assistant_message);
                                        
                                        // Execute each function call and add responses
                                        for function_call in &response.function_calls {
                                            // Find mock response from config
                                            let mock_response = api_config_clone
                                                .function_tools
                                                .iter()
                                                .find(|tool| tool.name == function_call.name)
                                                .map(|tool| tool.mock_response.clone())
                                                .unwrap_or_else(|| r#"{"result": "Function executed successfully"}"#.to_string());
                                            
                                            // Parse mock response as JSON
                                            let response_value = serde_json::from_str(&mock_response)
                                                .unwrap_or_else(|_| serde_json::json!({"result": mock_response}));
                                            
                                            // Add function response message to conversation
                                            let function_response_message = Message {
                                                id: format!("msg_fr_{}", js_sys::Date::now()),
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
                                            current_messages.push(function_response_message);
                                            
                                            // Add to display
                                            final_response.push_str(&format!(
                                                "🔧 **Function**: `{}` → {}",
                                                function_call.name,
                                                serde_json::to_string(&response_value).unwrap_or_else(|_| "Invalid response".to_string())
                                            ));
                                            if function_call != response.function_calls.last().unwrap() {
                                                final_response.push_str("\n");
                                            }
                                        }
                                        
                                        // Check iteration limit
                                        max_iterations -= 1;
                                        if max_iterations <= 0 {
                                            final_response.push_str("\n\n⚠️ Maximum function call iterations reached");
                                            break;
                                        }
                                    },
                                    Err(error) => {
                                        log!("OpenAI API error:", &error);
                                        if final_response.is_empty() {
                                            final_response = format!("❌ **API Error**: {}", error);
                                        } else {
                                            final_response.push_str(&format!("\n\n❌ **API Error**: {}", error));
                                        }
                                        break;
                                    }
                                }
                            }
                            
                            if final_response.is_empty() {
                                "No response from API".to_string()
                            } else {
                                final_response
                            }
                        }
                    };
                    
                    // Add assistant response to session
                    let mut new_sessions = (*sessions_clone).clone();
                    if let Some(session) = new_sessions.get_mut(&session_id_clone) {
                        let assistant_message = Message {
                            id: format!("msg_{}", js_sys::Date::now()),
                            role: MessageRole::Assistant,
                            content: response_content,
                            timestamp: js_sys::Date::now(),
                            function_call: None,
                            function_response: None,
                        };
                        
                        session.messages.push(assistant_message);
                        session.updated_at = js_sys::Date::now();
                        
                        sessions_clone.set(new_sessions.clone());
                        is_loading_clone.set(false);
                        
                        let _ = StorageManager::save_sessions(&new_sessions);
                    } else {
                        is_loading_clone.set(false);
                    }
                });
            } else {
                log!("No session selected! Creating a new session first...");
                // If no session is selected, create one first
                let new_session = ChatSession {
                    id: format!("session_{}", js_sys::Date::now()),
                    title: "New Chat".to_string(),
                    messages: vec![],
                    created_at: js_sys::Date::now(),
                    updated_at: js_sys::Date::now(),
                    pinned: false,
                };
                
                let mut new_sessions = (*sessions).clone();
                new_sessions.insert(new_session.id.clone(), new_session.clone());
                sessions.set(new_sessions.clone());
                current_session_id.set(Some(new_session.id.clone()));
                
                // Save to localStorage
                let _ = StorageManager::save_sessions(&new_sessions);
                let _ = StorageManager::save_current_session_id(&new_session.id);
                
                log!("Created new session:", &new_session.id);
                
                // The session will be available in the next render cycle
                // For now, just return and let the user click send again
            }
        })
    };

    let save_config = {
        let api_config = api_config.clone();
        let show_settings = show_settings.clone();
        
        Callback::from(move |config: ApiConfig| {
            api_config.set(config.clone());
            show_settings.set(false);
            let _ = StorageManager::save_config(&config);
        })
    };

    // Render
    html! {
        <div class={classes!("flex", "h-screen", if *dark_mode { "dark" } else { "" })}>
            <div class="bg-gray-50 text-gray-800 dark:bg-gray-900 dark:text-gray-200 transition-colors duration-200 flex h-screen w-full">
                // Sidebar
                <Sidebar
                    sessions={(*sessions).clone()}
                    current_session_id={(*current_session_id).clone()}
                    on_new_session={create_new_session}
                    on_select_session={select_session}
                    on_delete_session={delete_session}
                    on_toggle_settings={toggle_settings}
                />
                
                // Main Content Area
                <div class="flex-1 flex flex-col">
                    // Chat Header
                    <ChatHeader
                        current_session={(*current_session).clone()}
                        api_config={(*api_config).clone()}
                        on_toggle_dark_mode={toggle_dark_mode}
                        dark_mode={*dark_mode}
                    />
                    
                    // Chat Messages
                    <ChatRoom
                        session={(*current_session).clone()}
                        is_loading={*is_loading}
                    />
                    
                    // Input Area
                    <InputBar
                        current_message={(*current_message).clone()}
                        on_message_change={update_message_input}
                        on_send_message={send_message}
                        is_loading={*is_loading}
                    />
                </div>
                
                // Settings Panel
                {if *show_settings {
                    html! {
                        <SettingsPanel
                            config={(*api_config).clone()}
                            on_save={save_config}
                            on_close={close_settings}
                        />
                    }
                } else {
                    html! {}
                }}
            </div>
        </div>
    }
}