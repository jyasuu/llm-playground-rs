use yew::prelude::*;
use crate::llm_playground::{Message, MessageRole};

#[derive(Properties, PartialEq)]
pub struct MessageBubbleProps {
    pub message: Message,
}

#[function_component(MessageBubble)]
pub fn message_bubble(props: &MessageBubbleProps) -> Html {
    let (icon_class, bg_class, label, icon) = match props.message.role {
        MessageRole::System => (
            "bg-yellow-100 dark:bg-yellow-900/30",
            "bg-yellow-50 dark:bg-yellow-900/20",
            "System",
            "fas fa-cog text-yellow-600 dark:text-yellow-400"
        ),
        MessageRole::User => (
            "bg-blue-100 dark:bg-blue-900/30",
            "bg-blue-50 dark:bg-blue-900/20",
            "You",
            "fas fa-user text-blue-600 dark:text-blue-400"
        ),
        MessageRole::Assistant => (
            "bg-purple-100 dark:bg-purple-900/30",
            "bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700",
            "Assistant",
            "fas fa-robot text-purple-600 dark:text-purple-400"
        ),
        MessageRole::Function => (
            "bg-green-100 dark:bg-green-900/30",
            "bg-green-50 dark:bg-green-900/20",
            "Function",
            "fas fa-code text-green-600 dark:text-green-400"
        ),
    };

    html! {
        <div class="flex">
            <div class={classes!("w-10", "h-10", "rounded-full", "flex", "items-center", "justify-center", "mr-3", icon_class)}>
                <i class={icon}></i>
            </div>
            <div class={classes!("flex-1", "rounded-lg", "p-4", bg_class)}>
                <div class="font-medium mb-1">{label}</div>
                
                // Regular message content
                <div class="message-content text-sm">
                    {render_content(&props.message.content)}
                </div>
                
                // Function call display
                {if let Some(function_call) = &props.message.function_call {
                    html! {
                        <div class="function-call bg-gray-100 dark:bg-gray-700 rounded-lg p-4 mt-2 border-l-4 border-orange-500">
                            <div class="flex items-center mb-2">
                                <i class="fas fa-toolbox text-orange-500 mr-2"></i>
                                <span class="font-medium">{"Function Call"}</span>
                            </div>
                            <pre class="text-sm overflow-x-auto">
                                <code>{serde_json::to_string_pretty(function_call).unwrap_or_else(|_| "Invalid JSON".to_string())}</code>
                            </pre>
                        </div>
                    }
                } else {
                    html! {}
                }}
                
                // Function response display
                {if let Some(function_response) = &props.message.function_response {
                    html! {
                        <div class="structured-output bg-gray-100 dark:bg-gray-700 rounded-lg p-4 mt-2 border-l-4 border-green-500">
                            <div class="flex items-center mb-2">
                                <i class="fas fa-code text-green-500 mr-2"></i>
                                <span class="font-medium">{"Function Response"}</span>
                            </div>
                            <pre class="text-sm overflow-x-auto">
                                <code>{serde_json::to_string_pretty(function_response).unwrap_or_else(|_| "Invalid JSON".to_string())}</code>
                            </pre>
                        </div>
                    }
                } else {
                    html! {}
                }}
                
                // Timestamp
                <div class="text-xs text-gray-500 dark:text-gray-400 mt-2">
                    {format_timestamp(props.message.timestamp)}
                </div>
            </div>
        </div>
    }
}

fn render_content(content: &str) -> Html {
    // For now, just render as plain text with basic markdown-like formatting
    // TODO: Add proper markdown rendering with pulldown-cmark
    let lines: Vec<&str> = content.split('\n').collect();
    
    html! {
        <div class="space-y-1">
            {for lines.iter().map(|line| {
                html! {
                    {if line.starts_with("```") {
                        html! {
                            <pre class="bg-gray-800 text-gray-100 p-3 rounded my-2 overflow-x-auto">
                                <code>{line.trim_start_matches("```")}</code>
                            </pre>
                        }
                    } else if line.starts_with("# ") {
                        html! {
                            <h1 class="text-lg font-bold my-2">{line.trim_start_matches("# ")}</h1>
                        }
                    } else if line.starts_with("## ") {
                        html! {
                            <h2 class="text-md font-bold my-2">{line.trim_start_matches("## ")}</h2>
                        }
                    } else if line.starts_with("- ") {
                        html! {
                            <li class="ml-4">{line.trim_start_matches("- ")}</li>
                        }
                    } else if line.trim().is_empty() {
                        html! { <br/> }
                    } else {
                        html! {
                            <p>{line}</p>
                        }
                    }}
                }
            })}
        </div>
    }
}

fn format_timestamp(timestamp: f64) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(timestamp));
    let hours = date.get_hours();
    let minutes = date.get_minutes();
    format!("{:02}:{:02}", hours, minutes)
}