// Simplified LLM Playground with decoupled message handling
use gloo_console::log;
use gloo_storage::{LocalStorage, Storage};
use gloo_timers::future::TimeoutFuture;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::llm_playground::{
    components::notification::{
        use_notifications, NotificationContainer,
    },
    flexible_client::FlexibleLLMClient,
    mcp_client::McpClient,
    simple_orchestrator::use_simple_orchestrator,
    ChatHeader, ChatRoom, ChatSession, FlexibleApiConfig, FlexibleSettingsPanel, InputBar,
    ModelSelector, Sidebar,
};

const STORAGE_KEY_FLEXIBLE_CONFIG: &str = "llm_playground_flexible_config";
const STORAGE_KEY_SESSIONS: &str = "llm_playground_sessions";
const STORAGE_KEY_CURRENT_SESSION: &str = "llm_playground_current_session";
const STORAGE_KEY_DARK_MODE: &str = "llm_playground_dark_mode";

#[function_component(FlexibleLLMPlaygroundSimple)]
pub fn flexible_llm_playground_simple() -> Html {
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
    let mcp_client = use_state(|| Option::<McpClient>::None);

    // Notification system
    let (notifications, add_notification, dismiss_notification) = use_notifications();

    // Loading state change callback
    let on_loading_change = {
        let is_loading = is_loading.clone();
        Callback::from(move |(session_id, loading): (String, bool)| {
            log!("Loading state changed for session {}: {}", session_id, loading);
            is_loading.set(loading);
        })
    };

    // Initialize simple orchestrator
    let orchestrator = use_simple_orchestrator(
        sessions.clone(),
        api_config.clone(),
        (*llm_client).clone(),
        (*mcp_client).clone(),
        add_notification.clone(),
        on_loading_change,
    );

    // Auto-initialize MCP connections when MCP config changes
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
                    let needs_initialization = if let Some(current_client) = mcp_client.as_ref() {
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
                    log!("Added MCP tools to function tools list");

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
            // Load API config
            if let Ok(config_str) = LocalStorage::get::<String>(STORAGE_KEY_FLEXIBLE_CONFIG) {
                if let Ok(config) = serde_json::from_str::<FlexibleApiConfig>(&config_str) {
                    api_config.set(config);
                }
            }

            // Load sessions
            if let Ok(sessions_str) = LocalStorage::get::<String>(STORAGE_KEY_SESSIONS) {
                if let Ok(loaded_sessions) =
                    serde_json::from_str::<HashMap<String, ChatSession>>(&sessions_str)
                {
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

    // SIMPLIFIED MESSAGE HANDLING - Direct orchestrator call!
    let send_message = {
        let orchestrator = orchestrator.clone();
        let current_session_id = current_session_id.clone();
        let current_message = current_message.clone();
        
        Callback::from(move |_| {
            if let Some(session_id) = current_session_id.as_ref() {
                let message_content = (*current_message).clone();
                if !message_content.trim().is_empty() {
                    log!("Sending message: {}", &message_content);
                    // Direct call to orchestrator - much simpler!
                    orchestrator.send_message(session_id.clone(), message_content);
                    current_message.set(String::new());
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