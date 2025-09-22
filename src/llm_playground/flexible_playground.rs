// Updated LLM Playground with flexible provider system
use yew::prelude::*;
use web_sys::HtmlTextAreaElement;
use gloo_storage::{LocalStorage, Storage};
use std::collections::HashMap;
use gloo_console::log;
use wasm_bindgen::JsCast;

use crate::llm_playground::{
    ChatSession, Message, MessageRole, FlexibleApiConfig,
    Sidebar, ChatHeader, ChatRoom, InputBar, FlexibleSettingsPanel, ModelSelector,
    flexible_client::FlexibleLLMClient
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
    let is_loading = use_state(|| false);
    let llm_client = use_state(|| FlexibleLLMClient::new());

    // Load data from localStorage on mount
    {
        let sessions = sessions.clone();
        let current_session_id = current_session_id.clone();
        let api_config = api_config.clone();
        let dark_mode = dark_mode.clone();
        
        use_effect_with((), move |_| {
            // Load API config
            if let Ok(config_str) = LocalStorage::get::<String>(STORAGE_KEY_FLEXIBLE_CONFIG) {
                if let Ok(config) = serde_json::from_str::<FlexibleApiConfig>(&config_str) {
                    api_config.set(config);
                }
            }
            
            // Load sessions
            if let Ok(sessions_str) = LocalStorage::get::<String>(STORAGE_KEY_SESSIONS) {
                if let Ok(loaded_sessions) = serde_json::from_str::<HashMap<String, ChatSession>>(&sessions_str) {
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
            let session_id = format!("session_{}", js_sys::Date::now() as u64);
            let session_title = format!("{} - {}", provider_name, model_name);
            
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
            new_config.set_session_provider(&provider_name, &model_name);
            api_config.set(new_config);
            
            // Add session and set as current
            let mut new_sessions = (*sessions).clone();
            new_sessions.insert(session_id.clone(), new_session);
            sessions.set(new_sessions);
            current_session_id.set(Some(session_id));
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
            show_settings.set(false);
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

    // Message handling
    let send_message = {
        let sessions = sessions.clone();
        let current_session_id = current_session_id.clone();
        let current_message = current_message.clone();
        let is_loading = is_loading.clone();
        let api_config = api_config.clone();
        let llm_client = llm_client.clone();
        
        Callback::from(move |_| {
            let sessions = sessions.clone();
            if let Some(session_id) = current_session_id.as_ref() {
                let message_content = (*current_message).clone();
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
                current_message.set(String::new());
                is_loading.set(true);
                
                // Send to LLM
                let session_id_clone = session_id.clone();
                let config = (*api_config).clone();
                let client = (*llm_client).clone();
                let is_loading_clone = is_loading.clone();
                
                wasm_bindgen_futures::spawn_local(async move {
                    if let Some(session) = new_sessions.get(&session_id_clone) {
                        let messages = &session.messages;
                        
                        match client.send_message(messages, &config).await {
                            Ok(response) => {
                                let assistant_message = Message {
                                    id: format!("assistant_{}", js_sys::Date::now() as u64),
                                    role: MessageRole::Assistant,
                                    content: response.content.unwrap_or_default(),
                                    timestamp: js_sys::Date::now(),
                                    function_call: None,
                                    function_response: None,
                                };
                                
                                if let Some(session) = new_sessions.get_mut(&session_id_clone) {
                                    session.messages.push(assistant_message);
                                    session.updated_at = js_sys::Date::now();
                                }
                                sessions.set(new_sessions);
                            }
                            Err(error) => {
                                log!("Error sending message:", &error);
                                let error_message = Message {
                                    id: format!("error_{}", js_sys::Date::now() as u64),
                                    role: MessageRole::Assistant,
                                    content: format!("Error: {}", error),
                                    timestamp: js_sys::Date::now(),
                                    function_call: None,
                                    function_response: None,
                                };
                                
                                if let Some(session) = new_sessions.get_mut(&session_id_clone) {
                                    session.messages.push(error_message);
                                    session.updated_at = js_sys::Date::now();
                                }
                                sessions.set(new_sessions);
                            }
                        }
                        is_loading_clone.set(false);
                    }
                });
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
                }
            }
        } else {
            crate::llm_playground::ApiConfig::default()
        }
    };

    // Get current session
    let current_session = current_session_id.as_ref()
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
                                    on_send_message={send_message}
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
            </div>
        </div>
    }
}