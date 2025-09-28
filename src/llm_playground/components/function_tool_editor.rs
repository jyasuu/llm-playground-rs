use yew::prelude::*;
use web_sys::HtmlInputElement;
use crate::llm_playground::FunctionTool;

#[derive(Properties, PartialEq)]
pub struct FunctionToolEditorProps {
    pub tool: Option<FunctionTool>,
    pub on_save: Callback<FunctionTool>,
    pub on_cancel: Callback<()>,
}

#[function_component(FunctionToolEditor)]
pub fn function_tool_editor(props: &FunctionToolEditorProps) -> Html {
    let tool = use_state(|| {
        props.tool.clone().unwrap_or_else(|| FunctionTool {
            name: String::new(),
            description: String::new(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            mock_response: String::from(r#"{"result": "Success"}"#),
            enabled: true,
            category: "Custom".to_string(),
            is_builtin: false,
        })
    });

    let parameters_text = use_state(|| {
        serde_json::to_string_pretty(&tool.parameters)
            .unwrap_or_else(|_| String::from("{}"))
    });

    // Update local state when props change
    {
        let tool = tool.clone();
        let parameters_text = parameters_text.clone();
        let props_tool = props.tool.clone();
        use_effect_with(
            props_tool,
            move |props_tool| {
                if let Some(t) = props_tool {
                    tool.set(t.clone());
                    parameters_text.set(
                        serde_json::to_string_pretty(&t.parameters)
                            .unwrap_or_else(|_| String::from("{}"))
                    );
                }
                || ()
            },
        );
    }

    let on_name_change = {
        let tool = tool.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_tool = (*tool).clone();
            new_tool.name = input.value();
            tool.set(new_tool);
        })
    };

    let on_description_change = {
        let tool = tool.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_tool = (*tool).clone();
            new_tool.description = input.value();
            tool.set(new_tool);
        })
    };

    let on_parameters_change = {
        let tool = tool.clone();
        let parameters_text = parameters_text.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let text = input.value();
            parameters_text.set(text.clone());
            
            // Try to parse JSON and update tool if valid
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                let mut new_tool = (*tool).clone();
                new_tool.parameters = parsed;
                tool.set(new_tool);
            }
        })
    };

    let on_mock_response_change = {
        let tool = tool.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_tool = (*tool).clone();
            new_tool.mock_response = input.value();
            tool.set(new_tool);
        })
    };

    let on_save_click = {
        let tool = tool.clone();
        let on_save = props.on_save.clone();
        Callback::from(move |_| {
            on_save.emit((*tool).clone());
        })
    };

    let on_cancel_click = {
        let on_cancel = props.on_cancel.clone();
        Callback::from(move |_| {
            on_cancel.emit(());
        })
    };

    // Validate if current tool is valid
    let is_valid = !tool.name.trim().is_empty() 
        && !tool.description.trim().is_empty()
        && serde_json::from_str::<serde_json::Value>(&parameters_text).is_ok()
        && serde_json::from_str::<serde_json::Value>(&tool.mock_response).is_ok();

    // Check if this is a built-in tool (read-only)
    let is_builtin = tool.is_builtin;

    html! {
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div class="bg-white dark:bg-gray-800 rounded-lg p-6 w-full max-w-2xl max-h-[80vh] overflow-y-auto">
                <div class="flex justify-between items-center mb-4">
                    <h3 class="text-lg font-semibold">
                        {if props.tool.is_some() { "Edit Function Tool" } else { "Add Function Tool" }}
                    </h3>
                    <button 
                        onclick={on_cancel_click.clone()}
                        class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                    >
                        <i class="fas fa-times"></i>
                    </button>
                </div>

                {if is_builtin {
                    html! {
                        <div class="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-700 rounded-md p-3 mb-4">
                            <div class="flex items-center space-x-2">
                                <i class="fas fa-info-circle text-blue-600 dark:text-blue-400"></i>
                                <span class="text-sm text-blue-800 dark:text-blue-200 font-medium">
                                    {"This is a built-in function tool. It cannot be edited or deleted, but can be enabled/disabled."}
                                </span>
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }}

                <div class="space-y-4">
                    <div>
                        <label class="block text-sm font-medium mb-1">{"Function Name"}</label>
                        <input 
                            type="text"
                            value={tool.name.clone()}
                            oninput={on_name_change}
                            placeholder="e.g., get_weather"
                            disabled={is_builtin}
                            class={format!("w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 {}", 
                                if is_builtin { "opacity-50 cursor-not-allowed" } else { "" })}
                        />
                    </div>

                    <div>
                        <label class="block text-sm font-medium mb-1">{"Description"}</label>
                        <textarea 
                            value={tool.description.clone()}
                            oninput={on_description_change}
                            placeholder="What does this function do?"
                            rows="2"
                            disabled={is_builtin}
                            class={format!("w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 {}", 
                                if is_builtin { "opacity-50 cursor-not-allowed" } else { "" })}
                        />
                    </div>

                    <div>
                        <label class="block text-sm font-medium mb-1">{"Parameters (JSON Schema)"}</label>
                        <textarea 
                            value={(*parameters_text).clone()}
                            oninput={on_parameters_change}
                            placeholder={r#"{
  "type": "object",
  "properties": {
    "param1": {
      "type": "string",
      "description": "Parameter description"
    }
  },
  "required": ["param1"]
}"#}
                            rows="8"
                            class={classes!(
                                "w-full", "p-2", "border", "rounded-md", "font-mono", "text-sm",
                                "bg-white", "dark:bg-gray-700",
                                if serde_json::from_str::<serde_json::Value>(&parameters_text).is_ok() {
                                    "border-gray-300 dark:border-gray-600"
                                } else {
                                    "border-red-300 dark:border-red-600"
                                }
                            )}
                        />
                        {if serde_json::from_str::<serde_json::Value>(&parameters_text).is_err() {
                            html! {
                                <p class="text-red-500 text-xs mt-1">{"Invalid JSON syntax"}</p>
                            }
                        } else {
                            html! {}
                        }}
                    </div>

                    <div>
                        <label class="block text-sm font-medium mb-1">{"Mock Response (JSON)"}</label>
                        <textarea 
                            value={tool.mock_response.clone()}
                            oninput={on_mock_response_change}
                            placeholder={r#"{"result": "Mock response data"}"#}
                            rows="4"
                            class={classes!(
                                "w-full", "p-2", "border", "rounded-md", "font-mono", "text-sm",
                                "bg-white", "dark:bg-gray-700",
                                if serde_json::from_str::<serde_json::Value>(&tool.mock_response).is_ok() {
                                    "border-gray-300 dark:border-gray-600"
                                } else {
                                    "border-red-300 dark:border-red-600"
                                }
                            )}
                        />
                        {if serde_json::from_str::<serde_json::Value>(&tool.mock_response).is_err() {
                            html! {
                                <p class="text-red-500 text-xs mt-1">{"Invalid JSON syntax"}</p>
                            }
                        } else {
                            html! {}
                        }}
                    </div>
                </div>

                <div class="flex justify-end space-x-2 mt-6">
                    <button 
                        onclick={on_cancel_click}
                        class="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-700"
                    >
                        {"Cancel"}
                    </button>
                    <button 
                        onclick={on_save_click}
                        disabled={!is_valid}
                        class={classes!(
                            "px-4", "py-2", "rounded-md", "text-white",
                            if is_valid {
                                "bg-primary-600 hover:bg-primary-700"
                            } else {
                                "bg-gray-400 cursor-not-allowed"
                            }
                        )}
                    >
                        {"Save"}
                    </button>
                </div>
            </div>
        </div>
    }
}