// Updated LLM Playground with flexible provider system
use yew::prelude::*;
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
                        let mut current_messages = session.messages.clone();
                        
                        // Add system message if exists
                        if !config.system_prompt.trim().is_empty() {
                            current_messages.insert(0, Message {
                                id: "system".to_string(),
                                role: MessageRole::System,
                                content: config.system_prompt.clone(),
                                timestamp: js_sys::Date::now(),
                                function_call: None,
                                function_response: None,
                            });
                        }
                        
                        // Handle function calls automatically with feedback loop
                        let mut final_response = String::new();
                        let mut max_iterations = 5; // Prevent infinite loops
                        
                        loop {
                            match client.send_message(&current_messages, &config).await {
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
                                    
                                    // Add function calls section header
                                    let num_function_calls = response.function_calls.len();
                                    final_response.push_str(&format!(
                                        "## 🔧 Function Execution Sequence ({} {})\n\n",
                                        num_function_calls,
                                        if num_function_calls == 1 { "call" } else { "calls" }
                                    ));
                                    
                                    // Add additional context for multiple function calls
                                    if num_function_calls > 1 {
                                        final_response.push_str("The AI has requested multiple function calls to be executed in sequence. Each step is detailed below:\n\n");
                                    }
                                    
                                    // Add assistant message with function calls to conversation
                                    let assistant_message = Message {
                                        id: format!("msg_fc_{}", js_sys::Date::now() as u64),
                                        role: MessageRole::Assistant,
                                        content: response.content.unwrap_or_default(),
                                        timestamp: js_sys::Date::now(),
                                        function_call: if !response.function_calls.is_empty() {
                                            Some(serde_json::json!(
                                                response.function_calls.iter().map(|fc| {
                                                    serde_json::json!({
                                                        "id": format!("function-call-{}", js_sys::Date::now() as u64),
                                                        "name": fc.name,
                                                        "arguments": fc.arguments
                                                    })
                                                }).collect::<Vec<_>>()
                                            ))
                                        } else { None },
                                        function_response: None,
                                    };
                                    current_messages.push(assistant_message.clone());
                                    
                                    // Save assistant function call message to session immediately for display
                                    {
                                        if let Some(session) = new_sessions.get_mut(&session_id_clone) {
                                            session.messages.push(assistant_message);
                                            session.updated_at = js_sys::Date::now();
                                        }
                                    }
                                    
                                    // Execute each function call and add responses
                                    for function_call in &response.function_calls {
                                        // Find mock response from config
                                        let mock_response = config
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
                                            id: format!("msg_fr_{}", js_sys::Date::now() as u64),
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
                                        current_messages.push(function_response_message.clone());
                                        
                                        // Save function response message to session immediately for display
                                        {
                                            if let Some(session) = new_sessions.get_mut(&session_id_clone) {
                                                session.messages.push(function_response_message);
                                                session.updated_at = js_sys::Date::now();
                                            }
                                        }
                                        
                                        // Get the call number for this function
                                        let call_number = response.function_calls.iter().position(|fc| fc.id == function_call.id).map(|i| i + 1).unwrap_or(0);
                                        
                                        // Add to display (keeping for final response text)
                                        final_response.push_str(&format!(
                                            "### Step {}: Calling `{}`\n\n**Function**: `{}()`\n**Purpose**: {}\n\n**📤 Request Parameters**:\n```json\n{}\n```\n\n**📥 Response Received**:\n```json\n{}\n```\n\n**✅ Function call completed**",
                                            call_number,
                                            function_call.name,
                                            function_call.name,
                                            config
                                                .function_tools
                                                .iter()
                                                .find(|tool| tool.name == function_call.name)
                                                .map(|tool| tool.description.clone())
                                                .unwrap_or_else(|| "Execute function".to_string()),
                                            serde_json::to_string_pretty(&function_call.arguments).unwrap_or_else(|_| "{}".to_string()),
                                            serde_json::to_string_pretty(&response_value).unwrap_or_else(|_| "Invalid response".to_string())
                                        ));
                                        if function_call != response.function_calls.last().unwrap() {
                                            final_response.push_str("\n\n");
                                        }
                                    }
                                    
                                    // Add a summary at the end of all function calls
                                    final_response.push_str("\n\n---\n\n");
                                    final_response.push_str(&format!(
                                        "**🔄 Function Execution Summary**: Completed {} function {}.\n\n",
                                        response.function_calls.len(),
                                        if response.function_calls.len() == 1 { "call" } else { "calls" }
                                    ));
                                    
                                    // Check iteration limit
                                    max_iterations -= 1;
                                    if max_iterations <= 0 {
                                        final_response.push_str("\n\n⚠️ Maximum function call iterations reached");
                                        break;
                                    }
                                },
                                Err(error) => {
                                    log!("API error:", &error);
                                    if final_response.is_empty() {
                                        final_response = format!("❌ **API Error**: {}", error);
                                    } else {
                                        final_response.push_str(&format!("\n\n❌ **API Error**: {}", error));
                                    }
                                    break;
                                }
                            }
                        }
                        
                        // Add final assistant response to session only if it has content
                        if !final_response.trim().is_empty() {
                            if let Some(session) = new_sessions.get_mut(&session_id_clone) {
                                let assistant_message = Message {
                                    id: format!("assistant_{}", js_sys::Date::now() as u64),
                                    role: MessageRole::Assistant,
                                    content: final_response,
                                    timestamp: js_sys::Date::now(),
                                    function_call: None,
                                    function_response: None,
                                };
                                
                                session.messages.push(assistant_message);
                                session.updated_at = js_sys::Date::now();
                            }
                        }
                        
                        is_loading_clone.set(false);
                        
                        // Set state after mutations
                        sessions.set(new_sessions.clone());
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