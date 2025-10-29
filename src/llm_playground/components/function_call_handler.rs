use crate::llm_playground::api_clients::{FunctionCallRequest, FunctionResponse};
use crate::llm_playground::builtin_tools;
use crate::llm_playground::{ApiConfig, FunctionTool};
use gloo_console::log;
use wasm_bindgen_futures;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct FunctionCallHandlerProps {
    pub function_call: FunctionCallRequest,
    pub config: ApiConfig,
    pub on_response: Callback<FunctionResponse>,
    pub auto_approve: bool, // Whether to auto-approve function calls or require user approval
}

#[function_component(FunctionCallHandler)]
pub fn function_call_handler(props: &FunctionCallHandlerProps) -> Html {
    let pending_approval = use_state(|| !props.auto_approve);
    let executed = use_state(|| false);
    let response = use_state(|| None::<FunctionResponse>);

    // Auto-execute if auto_approve is true
    {
        let function_call = props.function_call.clone();
        let config = props.config.clone();
        let on_response = props.on_response.clone();
        let executed = executed.clone();
        let response = response.clone();
        let auto_approve = props.auto_approve;

        use_effect_with((function_call.clone(), auto_approve), move |_| {
            if auto_approve && !*executed {
                executed.set(true);

                // Check if this is a built-in tool
                let tool = config
                    .function_tools
                    .iter()
                    .find(|tool| tool.name == function_call.name);

                if let Some(tool) = tool {
                    if tool.is_builtin {
                        // Execute built-in tool with real functionality
                        log!("Executing built-in tool: {}", &function_call.name);
                        let function_call_clone = function_call.clone();
                        let on_response_clone = on_response.clone();
                        let response_clone = response.clone();

                        wasm_bindgen_futures::spawn_local(async move {
                            match builtin_tools::execute_builtin_tool(
                                &function_call_clone.name,
                                &function_call_clone.arguments,
                                None,
                            )
                            .await
                            {
                                Ok(result) => {
                                    let func_response = FunctionResponse {
                                        id: function_call_clone.id.clone(),
                                        name: function_call_clone.name.clone(),
                                        response: result,
                                    };
                                    response_clone.set(Some(func_response.clone()));
                                    on_response_clone.emit(func_response);
                                }
                                Err(error) => {
                                    let func_response = FunctionResponse {
                                        id: function_call_clone.id.clone(),
                                        name: function_call_clone.name.clone(),
                                        response: serde_json::json!({"error": error}),
                                    };
                                    response_clone.set(Some(func_response.clone()));
                                    on_response_clone.emit(func_response);
                                }
                            }
                        });
                    } else {
                        // Use mock response for regular tools
                        let mock_response = tool.mock_response.clone();
                        let response_value = serde_json::from_str(&mock_response)
                            .unwrap_or_else(|_| serde_json::json!({"result": mock_response}));

                        let func_response = FunctionResponse {
                            id: function_call.id.clone(),
                            name: function_call.name.clone(),
                            response: response_value,
                        };

                        response.set(Some(func_response.clone()));
                        on_response.emit(func_response);
                    }
                } else {
                    // Unknown tool
                    let func_response = FunctionResponse {
                        id: function_call.id.clone(),
                        name: function_call.name.clone(),
                        response: serde_json::json!({"error": "Unknown function tool"}),
                    };
                    response.set(Some(func_response.clone()));
                    on_response.emit(func_response);
                }
            }
            || ()
        });
    }

    let handle_approve = {
        let function_call = props.function_call.clone();
        let config = props.config.clone();
        let on_response = props.on_response.clone();
        let pending_approval = pending_approval.clone();
        let executed = executed.clone();
        let response = response.clone();

        Callback::from(move |_| {
            pending_approval.set(false);
            executed.set(true);

            // Check if this is a built-in tool
            let tool = config
                .function_tools
                .iter()
                .find(|tool| tool.name == function_call.name);

            if let Some(tool) = tool {
                if tool.is_builtin {
                    // Execute built-in tool with real functionality
                    log!(
                        "Executing built-in tool (manual approval): {}",
                        &function_call.name
                    );
                    let function_call_clone = function_call.clone();
                    let on_response_clone = on_response.clone();
                    let response_clone = response.clone();

                    wasm_bindgen_futures::spawn_local(async move {
                        match builtin_tools::execute_builtin_tool(
                            &function_call_clone.name,
                            &function_call_clone.arguments,
                            None,
                        )
                        .await
                        {
                            Ok(result) => {
                                let func_response = FunctionResponse {
                                    id: function_call_clone.id.clone(),
                                    name: function_call_clone.name.clone(),
                                    response: result,
                                };
                                response_clone.set(Some(func_response.clone()));
                                on_response_clone.emit(func_response);
                            }
                            Err(error) => {
                                let func_response = FunctionResponse {
                                    id: function_call_clone.id.clone(),
                                    name: function_call_clone.name.clone(),
                                    response: serde_json::json!({"error": error}),
                                };
                                response_clone.set(Some(func_response.clone()));
                                on_response_clone.emit(func_response);
                            }
                        }
                    });
                } else {
                    // Use mock response for regular tools
                    let mock_response = tool.mock_response.clone();
                    let response_value = serde_json::from_str(&mock_response)
                        .unwrap_or_else(|_| serde_json::json!({"result": mock_response}));

                    let func_response = FunctionResponse {
                        id: function_call.id.clone(),
                        name: function_call.name.clone(),
                        response: response_value,
                    };

                    response.set(Some(func_response.clone()));
                    on_response.emit(func_response);
                }
            } else {
                // Unknown tool
                let func_response = FunctionResponse {
                    id: function_call.id.clone(),
                    name: function_call.name.clone(),
                    response: serde_json::json!({"error": "Unknown function tool"}),
                };
                response.set(Some(func_response.clone()));
                on_response.emit(func_response);
            }
        })
    };

    let handle_reject = {
        let on_response = props.on_response.clone();
        let function_call = props.function_call.clone();
        let pending_approval = pending_approval.clone();
        let executed = executed.clone();
        let response = response.clone();

        Callback::from(move |_| {
            pending_approval.set(false);
            executed.set(true);

            let func_response = FunctionResponse {
                id: function_call.id.clone(),
                name: function_call.name.clone(),
                response: serde_json::json!({"error": "Function call rejected by user"}),
            };

            response.set(Some(func_response.clone()));
            on_response.emit(func_response);
        })
    };

    // Find the function tool details
    let function_tool: Option<&FunctionTool> = props
        .config
        .function_tools
        .iter()
        .find(|tool| tool.name == props.function_call.name);

    html! {
        <div class="function-call-container bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-700 rounded-lg p-4 my-2">
            <div class="flex items-start space-x-3">
                <div class="flex-shrink-0">
                    <i class="fas fa-cog text-blue-600 dark:text-blue-400 text-lg"></i>
                </div>
                <div class="flex-1 min-w-0">
                    <div class="flex items-center justify-between">
                        <h4 class="text-lg font-medium text-blue-900 dark:text-blue-100">
                            {"🔧 Function Call: "}
                            <code class="bg-blue-100 dark:bg-blue-800 px-2 py-1 rounded text-sm">
                                {&props.function_call.name}
                            </code>
                        </h4>
                        {if *executed {
                            html! {
                                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200">
                                    {"Executed"}
                                </span>
                            }
                        } else if *pending_approval {
                            html! {
                                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200">
                                    {"Pending Approval"}
                                </span>
                            }
                        } else {
                            html! {}
                        }}
                    </div>

                    {if let Some(tool) = function_tool {
                        html! {
                            <p class="text-sm text-blue-700 dark:text-blue-300 mt-1">
                                {&tool.description}
                            </p>
                        }
                    } else {
                        html! {}
                    }}

                    <div class="mt-3">
                        <h5 class="text-sm font-medium text-blue-900 dark:text-blue-100 mb-2">{"Function Parameters:"}</h5>
                        <div class="bg-white dark:bg-gray-800 rounded-md p-3 border border-blue-200 dark:border-blue-600">
                            {if let Some(args_obj) = props.function_call.arguments.as_object() {
                                if args_obj.is_empty() {
                                    html! {
                                        <div class="text-sm text-gray-500 dark:text-gray-400 italic">{"No parameters"}</div>
                                    }
                                } else {
                                    html! {
                                        <div class="space-y-2">
                                            {for args_obj.iter().map(|(key, value)| {
                                                html! {
                                                    <div class="flex items-start">
                                                        <span class="text-xs font-mono bg-blue-100 dark:bg-blue-700 px-2 py-1 rounded mr-3 text-blue-600 dark:text-blue-400 font-semibold min-w-0 flex-shrink-0">
                                                            {key}
                                                        </span>
                                                        <span class="text-xs font-mono text-gray-800 dark:text-gray-200 flex-1 break-all">
                                                            {format!("{}", value)}
                                                        </span>
                                                    </div>
                                                }
                                            })}
                                        </div>
                                    }
                                }
                            } else {
                                html! {
                                    <pre class="text-xs font-mono text-gray-800 dark:text-gray-200 overflow-x-auto">
                                        <code>{serde_json::to_string_pretty(&props.function_call.arguments).unwrap_or_else(|_| "Invalid parameters".to_string())}</code>
                                    </pre>
                                }
                            }}
                        </div>
                    </div>

                    {if let Some(resp) = response.as_ref() {
                        html! {
                            <div class="mt-4">
                                <div class="flex items-center mb-2">
                                    <i class="fas fa-check-circle text-green-600 dark:text-green-400 mr-2"></i>
                                    <h5 class="text-sm font-medium text-green-900 dark:text-green-100">{"Function Response:"}</h5>
                                </div>
                                <div class="bg-gradient-to-r from-green-50 to-emerald-50 dark:from-green-900/20 dark:to-emerald-900/20 rounded-md p-3 border border-green-200 dark:border-green-600">
                                    <div class="mb-2">
                                        <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 dark:bg-green-900/50 text-green-800 dark:text-green-300">
                                            <i class="fas fa-reply mr-1"></i>
                                            {&resp.name}
                                        </span>
                                    </div>
                                    <div class="bg-white dark:bg-gray-800 rounded p-3 border border-gray-200 dark:border-gray-600">
                                        <pre class="text-xs font-mono text-gray-800 dark:text-gray-200 overflow-x-auto">
                                            <code>{serde_json::to_string_pretty(&resp.response).unwrap_or_else(|_| "Invalid response".to_string())}</code>
                                        </pre>
                                    </div>
                                </div>
                            </div>
                        }
                    } else {
                        html! {}
                    }}

                    {if *pending_approval {
                        html! {
                            <div class="mt-4 flex space-x-3">
                                <button
                                    onclick={handle_approve}
                                    class="inline-flex items-center px-3 py-2 border border-transparent text-sm leading-4 font-medium rounded-md text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
                                >
                                    <i class="fas fa-check mr-2"></i>
                                    {"Approve & Execute"}
                                </button>
                                <button
                                    onclick={handle_reject}
                                    class="inline-flex items-center px-3 py-2 border border-gray-300 text-sm leading-4 font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-gray-200 dark:border-gray-600 dark:hover:bg-gray-600"
                                >
                                    <i class="fas fa-times mr-2"></i>
                                    {"Reject"}
                                </button>
                            </div>
                        }
                    } else {
                        html! {}
                    }}
                </div>
            </div>
        </div>
    }
}
