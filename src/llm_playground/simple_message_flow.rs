// Simplified working message flow implementation
use std::collections::HashMap;
use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::llm_playground::{
    types::{Message, MessageRole, ChatSession},
    flexible_client::FlexibleLLMClient,
    provider_config::FlexibleApiConfig,
    mcp_client::McpClient,
};
use gloo_console::log;

/// Simple message flow handler that works with the existing UI
pub struct SimpleMessageFlow;

impl SimpleMessageFlow {
    pub fn create_send_message_callback(
        sessions: UseStateHandle<HashMap<String, ChatSession>>,
        current_session_id: UseStateHandle<Option<String>>,
        current_message: UseStateHandle<String>,
        is_loading: UseStateHandle<bool>,
        api_config: UseStateHandle<FlexibleApiConfig>,
        llm_client: UseStateHandle<FlexibleLLMClient>,
        mcp_client: UseStateHandle<Option<McpClient>>,
    ) -> Callback<()> {
        Callback::from(move |_| {
            if let Some(session_id) = current_session_id.as_ref() {
                let message_content = (*current_message).clone();
                if message_content.trim().is_empty() {
                    return;
                }

                // Create user message
                let user_message = Message {
                    id: format!("user_{}", js_sys::Date::now() as u64),
                    role: MessageRole::User,
                    content: message_content.clone(),
                    timestamp: js_sys::Date::now(),
                    function_call: None,
                    function_response: None,
                };

                // Add user message to session
                let mut sessions_map = (*sessions).clone();
                if let Some(session) = sessions_map.get_mut(session_id) {
                    session.messages.push(user_message);
                    session.updated_at = js_sys::Date::now();
                    sessions.set(sessions_map.clone());
                }

                current_message.set(String::new());
                is_loading.set(true);

                // Make LLM call
                let session_id_clone = session_id.clone();
                let config = (*api_config).clone();
                let client = (*llm_client).clone();
                let mcp_client_ref = (*mcp_client).clone();
                let sessions_handle = sessions.clone();
                let is_loading_handle = is_loading.clone();

                spawn_local(async move {
                    // Get current session messages
                    let sessions_map_current = (*sessions_handle).clone();
                    if let Some(session) = sessions_map_current.get(&session_id_clone) {
                        let mut current_messages = session.messages.clone();

                        // Add system message if exists
                        if !config.system_prompt.trim().is_empty() {
                            current_messages.insert(
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

                        // Simple LLM call without complex function handling for now
                        match client.send_message(&current_messages, &config).await {
                            Ok(response) => {
                                let mut final_response = response.content.unwrap_or_default();

                                // Handle function calls if any (simplified)
                                if !response.function_calls.is_empty() {
                                    final_response.push_str("\n\n## 🔧 Function Calls Executed:\n");
                                    
                                    for function_call in &response.function_calls {
                                        // Execute function call
                                        let function_result = if let Some(tool) = config
                                            .function_tools
                                            .iter()
                                            .find(|tool| tool.name == function_call.name)
                                        {
                                            if tool.is_builtin {
                                                match crate::llm_playground::builtin_tools::execute_builtin_tool(
                                                    &function_call.name,
                                                    &function_call.arguments,
                                                    mcp_client_ref.as_ref(),
                                                ).await {
                                                    Ok(result) => serde_json::to_string_pretty(&result).unwrap_or_default(),
                                                    Err(error) => format!("Error: {}", error),
                                                }
                                            } else {
                                                tool.mock_response.clone()
                                            }
                                        } else {
                                            "Unknown function".to_string()
                                        };

                                        final_response.push_str(&format!(
                                            "\n**{}**: {}\n",
                                            function_call.name,
                                            function_result
                                        ));
                                    }
                                }

                                // Add assistant response
                                let mut sessions_map_final = (*sessions_handle).clone();
                                if let Some(session) = sessions_map_final.get_mut(&session_id_clone) {
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
                                    sessions_handle.set(sessions_map_final);
                                }
                            }
                            Err(error) => {
                                log!("API Error:", &error);
                                
                                // Add error message
                                let mut sessions_map_error = (*sessions_handle).clone();
                                if let Some(session) = sessions_map_error.get_mut(&session_id_clone) {
                                    let error_message = Message {
                                        id: format!("error_{}", js_sys::Date::now() as u64),
                                        role: MessageRole::Assistant,
                                        content: format!("❌ **API Error**: {}", error),
                                        timestamp: js_sys::Date::now(),
                                        function_call: None,
                                        function_response: None,
                                    };

                                    session.messages.push(error_message);
                                    session.updated_at = js_sys::Date::now();
                                    sessions_handle.set(sessions_map_error);
                                }
                            }
                        }
                    }

                    is_loading_handle.set(false);
                });
            }
        })
    }
}