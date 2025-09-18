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
use web_sys::HtmlInputElement;
use gloo_storage::{LocalStorage, Storage};
use std::collections::HashMap;
use gloo_console::log;

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

    let update_message_input = {
        let current_message = current_message.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            current_message.set(input.value());
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
                
                // Set state after mutations
                sessions.set(new_sessions.clone());
                current_message.set(String::new());
                is_loading.set(true);
                
                // Save to localStorage
                let _ = StorageManager::save_sessions(&new_sessions);
                
                // Simulate API response for now
                let sessions_clone = sessions.clone();
                let session_id_clone = session_id.clone();
                let is_loading_clone = is_loading.clone();
                    
                wasm_bindgen_futures::spawn_local(async move {
                    gloo_timers::future::TimeoutFuture::new(1000).await;
                    
                    let mut new_sessions = (*sessions_clone).clone();
                    if let Some(session) = new_sessions.get_mut(&session_id_clone) {
                        let assistant_message = Message {
                            id: format!("msg_{}", js_sys::Date::now()),
                            role: MessageRole::Assistant,
                            content: "This is a mock response. The actual API integration will be implemented next.".to_string(),
                            timestamp: js_sys::Date::now(),
                            function_call: None,
                            function_response: None,
                        };
                        
                        session.messages.push(assistant_message);
                        session.updated_at = js_sys::Date::now();
                        
                        sessions_clone.set(new_sessions.clone());
                        is_loading_clone.set(false);
                        
                        let _ = StorageManager::save_sessions(&new_sessions);
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