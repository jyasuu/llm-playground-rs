use yew::prelude::*;
use crate::llm_playground::{ApiConfig, FunctionTool};
use crate::llm_playground::api_clients::{FunctionCallRequest, FunctionResponse};
use gloo_console::log;
use web_sys::js_sys;

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

        use_effect_with(
            (function_call.clone(), auto_approve),
            move |_| {
                if auto_approve && !*executed {
                    executed.set(true);
                    
                    // Find the mock response for this function
                    let mock_response = config
                        .function_tools
                        .iter()
                        .find(|tool| tool.name == function_call.name)
                        .map(|tool| tool.mock_response.clone())
                        .unwrap_or_else(|| r#"{"result": "Function executed successfully"}"#.to_string());
                    
                    // Parse the mock response as JSON
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
                || ()
            },
        );
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
            
            // Find the mock response for this function
            let mock_response = config
                .function_tools
                .iter()
                .find(|tool| tool.name == function_call.name)
                .map(|tool| tool.mock_response.clone())
                .unwrap_or_else(|| r#"{"result": "Function executed successfully"}"#.to_string());
            
            // Parse the mock response as JSON
            let response_value = serde_json::from_str(&mock_response)
                .unwrap_or_else(|_| serde_json::json!({"result": mock_response}));
            
            let func_response = FunctionResponse {
                id: function_call.id.clone(),
                name: function_call.name.clone(),
                response: response_value,
            };
            
            response.set(Some(func_response.clone()));
            on_response.emit(func_response);
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
    let function_tool: Option<&FunctionTool> = props.config
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
                        <h5 class="text-sm font-medium text-blue-900 dark:text-blue-100 mb-2">{"Arguments:"}</h5>
                        <pre class="bg-blue-100 dark:bg-blue-800 p-3 rounded text-xs overflow-x-auto">
                            <code>{serde_json::to_string_pretty(&props.function_call.arguments).unwrap_or_else(|_| "Invalid JSON".to_string())}</code>
                        </pre>
                    </div>

                    {if let Some(resp) = response.as_ref() {
                        html! {
                            <div class="mt-3">
                                <h5 class="text-sm font-medium text-blue-900 dark:text-blue-100 mb-2">{"Response:"}</h5>
                                <pre class="bg-green-100 dark:bg-green-800 p-3 rounded text-xs overflow-x-auto">
                                    <code>{serde_json::to_string_pretty(&resp.response).unwrap_or_else(|_| "Invalid JSON".to_string())}</code>
                                </pre>
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