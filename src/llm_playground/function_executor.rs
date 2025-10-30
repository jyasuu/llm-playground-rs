// Function executor for handling function call execution
use gloo_console::log;
use std::collections::HashMap;
use yew::prelude::*;

use crate::llm_playground::{
    FlexibleApiConfig, Message, MessageRole,
    mcp_client::McpClient,
};
use crate::llm_playground::api_clients::traits::FunctionCallRequest;
use super::event_system::{PlaygroundEvent, EventBus, MessageProcessingState};

// Function executor for decoupled function handling
#[derive(Debug, Clone)]
pub struct FunctionExecutor {
    event_bus: EventBus,
    processing_state: UseStateHandle<HashMap<String, MessageProcessingState>>,
    mcp_client: Option<McpClient>,
}

impl FunctionExecutor {
    pub fn new(
        event_bus: EventBus,
        processing_state: UseStateHandle<HashMap<String, MessageProcessingState>>,
        mcp_client: Option<McpClient>,
    ) -> Self {
        Self {
            event_bus,
            processing_state,
            mcp_client,
        }
    }

    // Execute function calls
    pub fn execute_function_calls(
        &self,
        session_id: String,
        function_calls: Vec<FunctionCallRequest>,
        config: FlexibleApiConfig,
    ) {
        if function_calls.is_empty() {
            return;
        }

        log!(
            "FunctionExecutor: Executing {} function calls for session {}",
            function_calls.len(),
            &session_id
        );

        // Update processing state
        self.update_processing_state(&session_id, |state| {
            state.current_step = format!("Executing {} function calls", function_calls.len());
            state.function_calls_pending = function_calls.len();
            state.function_calls_completed = 0;
        });

        let event_bus = self.event_bus.clone();
        let processing_state = self.processing_state.clone();
        let mcp_client = self.mcp_client.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let mut function_responses = Vec::new();
            let mut completed_count = 0;

            for function_call in &function_calls {
                log!("Executing function: {}", &function_call.name);

                // Update processing state for current function
                {
                    let mut state_map = (*processing_state).clone();
                    if let Some(state) = state_map.get_mut(&session_id) {
                        state.current_step = format!("Executing function: {}", function_call.name);
                    }
                    processing_state.set(state_map);
                }

                // Execute the function
                let result = Self::execute_single_function(
                    function_call,
                    &config,
                    mcp_client.as_ref(),
                ).await;

                completed_count += 1;

                // Update processing state
                {
                    let mut state_map = (*processing_state).clone();
                    if let Some(state) = state_map.get_mut(&session_id) {
                        state.function_calls_completed = completed_count;
                    }
                    processing_state.set(state_map);
                }

                match result {
                    Ok(response_value) => {
                        // Create function response message
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

                        function_responses.push(function_response_message);

                        // Emit individual function completion event
                        event_bus.emit(PlaygroundEvent::FunctionCallCompleted {
                            session_id: session_id.clone(),
                            function_call: function_call.clone(),
                            result: response_value,
                        });
                    }
                    Err(error) => {
                        log!("Function execution error: {}", &error);
                        event_bus.emit(PlaygroundEvent::FunctionError {
                            session_id: session_id.clone(),
                            function_name: function_call.name.clone(),
                            error,
                        });
                        
                        // Create error response message
                        let error_response_message = Message {
                            id: format!("msg_fr_err_{}", js_sys::Date::now() as u64),
                            role: MessageRole::Function,
                            content: format!("Function {} failed", function_call.name),
                            timestamp: js_sys::Date::now(),
                            function_call: None,
                            function_response: Some(serde_json::json!({
                                "id": function_call.id,
                                "name": function_call.name,
                                "error": "Function execution failed"
                            })),
                        };

                        function_responses.push(error_response_message);
                    }
                }
            }

            // Reset processing state
            {
                let mut state_map = (*processing_state).clone();
                if let Some(state) = state_map.get_mut(&session_id) {
                    *state = MessageProcessingState::default();
                }
                processing_state.set(state_map);
            }

            // Emit completion event with all function response messages
            event_bus.emit(PlaygroundEvent::AllFunctionCallsCompleted {
                session_id,
                messages: function_responses,
            });
        });
    }

    // Execute a single function call
    async fn execute_single_function(
        function_call: &FunctionCallRequest,
        config: &FlexibleApiConfig,
        mcp_client: Option<&McpClient>,
    ) -> Result<serde_json::Value, String> {
        // Check if this is a built-in tool and execute it properly
        if let Some(tool) = config
            .function_tools
            .iter()
            .find(|tool| tool.name == function_call.name)
        {
            if tool.is_builtin {
                // Execute built-in tool with real functionality (including MCP tools)
                log!("Executing built-in tool: {}", &function_call.name);
                match crate::llm_playground::builtin_tools::execute_builtin_tool(
                    &function_call.name,
                    &function_call.arguments,
                    mcp_client,
                ).await {
                    Ok(result) => Ok(result),
                    Err(error) => Ok(serde_json::json!({"error": error})),
                }
            } else {
                // Use mock response for regular tools
                Ok(serde_json::from_str(&tool.mock_response)
                    .unwrap_or_else(|_| serde_json::json!({"result": tool.mock_response.clone()})))
            }
        } else {
            // Unknown tool
            Ok(serde_json::json!({"error": "Unknown function tool"}))
        }
    }

    // Update processing state helper
    fn update_processing_state<F>(&self, session_id: &str, updater: F)
    where
        F: FnOnce(&mut MessageProcessingState),
    {
        let mut state_map = (*self.processing_state).clone();
        let state = state_map.entry(session_id.to_string()).or_default();
        updater(state);
        self.processing_state.set(state_map);
    }
}

// Hook for using the function executor
#[hook]
pub fn use_function_executor(
    event_bus: EventBus,
    processing_state: UseStateHandle<HashMap<String, MessageProcessingState>>,
    mcp_client: Option<McpClient>,
) -> FunctionExecutor {
    FunctionExecutor::new(event_bus, processing_state, mcp_client)
}