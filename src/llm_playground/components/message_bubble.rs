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
                    // Handle function calls as an array
                    if let Some(function_calls_array) = function_call.as_array() {
                        html! {
                            <div class="function-calls">
                                {for function_calls_array.iter().enumerate().map(|(index, fc)| {
                                    html! {
                                        <div class="function-call bg-gradient-to-r from-orange-50 to-yellow-50 dark:from-orange-900/20 dark:to-yellow-900/20 rounded-lg p-4 mt-3 border border-orange-200 dark:border-orange-700">
                                            <div class="flex items-center mb-3">
                                                <i class="fas fa-play-circle text-orange-600 dark:text-orange-400 mr-2"></i>
                                                <span class="font-semibold text-orange-800 dark:text-orange-300">
                                                    {if function_calls_array.len() > 1 {
                                                        format!("Function Call {} of {}", index + 1, function_calls_array.len())
                                                    } else {
                                                        "Function Call Invoked".to_string()
                                                    }}
                                                </span>
                                            </div>
                                            
                                            {if let Some(name) = fc.get("name") {
                                                html! {
                                                    <div class="mb-2">
                                                        <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-orange-100 dark:bg-orange-900/50 text-orange-800 dark:text-orange-300">
                                                            <i class="fas fa-function mr-1"></i>
                                                            {name.as_str().unwrap_or("Unknown")}
                                                        </span>
                                                    </div>
                                                }
                                            } else {
                                                html! {}
                                            }}
                                            
                                            {if let Some(args) = fc.get("arguments") {
                                                html! {
                                                    <div>
                                                        <div class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{"Parameters:"}</div>
                                                        <div class="bg-white dark:bg-gray-800 rounded-md p-3 border border-gray-200 dark:border-gray-600">
                                                            {if let Some(args_obj) = args.as_object() {
                                                                html! {
                                                                    <div class="space-y-2">
                                                                        {for args_obj.iter().map(|(key, value)| {
                                                                            html! {
                                                                                <div class="flex items-start">
                                                                                    <span class="text-xs font-mono bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded mr-2 text-blue-600 dark:text-blue-400">{key}</span>
                                                                                    <span class="text-xs font-mono text-gray-800 dark:text-gray-200 flex-1">{format!("{}", value)}</span>
                                                                                </div>
                                                                            }
                                                                        })}
                                                                    </div>
                                                                }
                                                            } else {
                                                                html! {
                                                                    <pre class="text-xs font-mono text-gray-800 dark:text-gray-200 overflow-x-auto">
                                                                        <code>{serde_json::to_string_pretty(args).unwrap_or_else(|_| "Invalid parameters".to_string())}</code>
                                                                    </pre>
                                                                }
                                                            }}
                                                        </div>
                                                    </div>
                                                }
                                            } else {
                                                html! {
                                                    <div class="text-sm text-gray-500 dark:text-gray-400 italic">{"No parameters"}</div>
                                                }
                                            }}
                                        </div>
                                    }
                                })}
                            </div>
                        }
                    } else {
                        // Fallback for single function call object (backward compatibility)
                        html! {
                            <div class="function-call bg-gradient-to-r from-orange-50 to-yellow-50 dark:from-orange-900/20 dark:to-yellow-900/20 rounded-lg p-4 mt-3 border border-orange-200 dark:border-orange-700">
                                <div class="flex items-center mb-3">
                                    <i class="fas fa-play-circle text-orange-600 dark:text-orange-400 mr-2"></i>
                                    <span class="font-semibold text-orange-800 dark:text-orange-300">{"Function Call Invoked"}</span>
                                </div>
                                
                                {if let Some(name) = function_call.get("name") {
                                    html! {
                                        <div class="mb-2">
                                            <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-orange-100 dark:bg-orange-900/50 text-orange-800 dark:text-orange-300">
                                                <i class="fas fa-function mr-1"></i>
                                                {name.as_str().unwrap_or("Unknown")}
                                            </span>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }}
                                
                                {if let Some(args) = function_call.get("arguments") {
                                    html! {
                                        <div>
                                            <div class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{"Parameters:"}</div>
                                            <div class="bg-white dark:bg-gray-800 rounded-md p-3 border border-gray-200 dark:border-gray-600">
                                                {if let Some(args_obj) = args.as_object() {
                                                    html! {
                                                        <div class="space-y-2">
                                                            {for args_obj.iter().map(|(key, value)| {
                                                                html! {
                                                                    <div class="flex items-start">
                                                                        <span class="text-xs font-mono bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded mr-2 text-blue-600 dark:text-blue-400">{key}</span>
                                                                        <span class="text-xs font-mono text-gray-800 dark:text-gray-200 flex-1">{format!("{}", value)}</span>
                                                                    </div>
                                                                }
                                                            })}
                                                        </div>
                                                    }
                                                } else {
                                                    html! {
                                                        <pre class="text-xs font-mono text-gray-800 dark:text-gray-200 overflow-x-auto">
                                                            <code>{serde_json::to_string_pretty(args).unwrap_or_else(|_| "Invalid parameters".to_string())}</code>
                                                        </pre>
                                                    }
                                                }}
                                            </div>
                                        </div>
                                    }
                                } else {
                                    html! {
                                        <div class="text-sm text-gray-500 dark:text-gray-400 italic">{"No parameters"}</div>
                                    }
                                }}
                            </div>
                        }
                    }
                } else {
                    html! {}
                }}
                
                // Function response display
                {if let Some(function_response) = &props.message.function_response {
                    html! {
                        <div class="function-response bg-gradient-to-r from-green-50 to-emerald-50 dark:from-green-900/20 dark:to-emerald-900/20 rounded-lg p-4 mt-3 border border-green-200 dark:border-green-700">
                            <div class="flex items-center mb-3">
                                <i class="fas fa-check-circle text-green-600 dark:text-green-400 mr-2"></i>
                                <span class="font-semibold text-green-800 dark:text-green-300">{"Function Response"}</span>
                            </div>
                            
                            {if let Some(name) = function_response.get("name") {
                                html! {
                                    <div class="mb-2">
                                        <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 dark:bg-green-900/50 text-green-800 dark:text-green-300">
                                            <i class="fas fa-reply mr-1"></i>
                                            {name.as_str().unwrap_or("Unknown")}
                                        </span>
                                    </div>
                                }
                            } else {
                                html! {}
                            }}
                            
                            {if let Some(response) = function_response.get("response") {
                                html! {
                                    <div>
                                        <div class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">{"Response Data:"}</div>
                                        <div class="bg-white dark:bg-gray-800 rounded-md p-3 border border-gray-200 dark:border-gray-600">
                                            <pre class="text-xs font-mono text-gray-800 dark:text-gray-200 overflow-x-auto">
                                                <code>{serde_json::to_string_pretty(response).unwrap_or_else(|_| "Invalid response".to_string())}</code>
                                            </pre>
                                        </div>
                                    </div>
                                }
                            } else {
                                html! {
                                    <div class="bg-white dark:bg-gray-800 rounded-md p-3 border border-gray-200 dark:border-gray-600">
                                        <pre class="text-xs font-mono text-gray-800 dark:text-gray-200 overflow-x-auto">
                                            <code>{serde_json::to_string_pretty(function_response).unwrap_or_else(|_| "Invalid JSON".to_string())}</code>
                                        </pre>
                                    </div>
                                }
                            }}
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
    // Enhanced markdown rendering for function calls and formatting
    parse_markdown(content)
}

fn parse_markdown(content: &str) -> Html {
    let mut lines = Vec::new();
    let mut in_code_block = false;
    let mut code_block_content = Vec::new();
    let mut code_block_language = String::new();
    
    for line in content.split('\n') {
        if line.trim().starts_with("```") {
            if in_code_block {
                // End of code block
                let code_content = code_block_content.join("\n");
                lines.push(html! {
                    <pre class="bg-gray-900 text-gray-100 p-4 rounded-lg my-3 overflow-x-auto border-l-4 border-blue-500">
                        <div class="text-xs text-gray-400 mb-2">{if code_block_language.is_empty() { "Code" } else { &code_block_language }}</div>
                        <code class="text-sm">{code_content}</code>
                    </pre>
                });
                code_block_content.clear();
                code_block_language.clear();
                in_code_block = false;
            } else {
                // Start of code block
                code_block_language = line.trim().trim_start_matches("```").to_string();
                in_code_block = true;
            }
        } else if in_code_block {
            code_block_content.push(line.to_string());
        } else {
            lines.push(render_line(line));
        }
    }
    
    html! {
        <div class="space-y-2">
            {for lines}
        </div>
    }
}

fn render_line(line: &str) -> Html {
    if line.trim().is_empty() {
        return html! { <div class="h-2"></div> };
    }
    
    if line.starts_with("🔧 ") {
        // Function call header
        return html! {
            <div class="flex items-center space-x-2 p-3 bg-orange-50 dark:bg-orange-900/20 rounded-lg border-l-4 border-orange-500">
                <span class="text-orange-600 dark:text-orange-400">{"🔧"}</span>
                <span class="font-semibold text-orange-800 dark:text-orange-300">{render_inline_formatting(&line[4..])}</span>
            </div>
        };
    }
    
    if line.starts_with("**") && line.ends_with("**:") {
        // Section headers like **Arguments**: or **Response**:
        let content = line.trim_start_matches("**").trim_end_matches("**:");
        return html! {
            <div class="font-semibold text-gray-800 dark:text-gray-200 mt-4 mb-2 pb-1 border-b border-gray-200 dark:border-gray-600">
                {content}
            </div>
        };
    }
    
    if line.starts_with("# ") {
        return html! {
            <h1 class="text-xl font-bold my-3 text-gray-900 dark:text-gray-100">{line.trim_start_matches("# ")}</h1>
        };
    }
    
    if line.starts_with("## ") {
        return html! {
            <h2 class="text-lg font-semibold my-2 text-gray-800 dark:text-gray-200">{line.trim_start_matches("## ")}</h2>
        };
    }
    
    if line.starts_with("- ") {
        return html! {
            <div class="flex items-start space-x-2 ml-4">
                <span class="text-gray-500 mt-1">{"•"}</span>
                <span>{render_inline_formatting(&line[2..])}</span>
            </div>
        };
    }
    
    html! {
        <p class="text-gray-800 dark:text-gray-200 leading-relaxed">{render_inline_formatting(line)}</p>
    }
}

fn render_inline_formatting(text: &str) -> Html {
    let mut result = Vec::new();
    let mut chars = text.chars().peekable();
    let mut current_text = String::new();
    
    while let Some(ch) = chars.next() {
        if ch == '`' {
            // Handle inline code
            if !current_text.is_empty() {
                result.push(html! { <span>{current_text.clone()}</span> });
                current_text.clear();
            }
            
            let mut code_content = String::new();
            while let Some(next_ch) = chars.next() {
                if next_ch == '`' {
                    break;
                }
                code_content.push(next_ch);
            }
            
            result.push(html! {
                <code class="bg-gray-200 dark:bg-gray-700 text-gray-800 dark:text-gray-200 px-2 py-1 rounded text-sm font-mono">
                    {code_content}
                </code>
            });
        } else if ch == '*' && chars.peek() == Some(&'*') {
            // Handle bold text
            chars.next(); // consume second *
            
            if !current_text.is_empty() {
                result.push(html! { <span>{current_text.clone()}</span> });
                current_text.clear();
            }
            
            let mut bold_content = String::new();
            let mut found_end = false;
            
            while let Some(next_ch) = chars.next() {
                if next_ch == '*' && chars.peek() == Some(&'*') {
                    chars.next(); // consume second *
                    found_end = true;
                    break;
                }
                bold_content.push(next_ch);
            }
            
            if found_end {
                result.push(html! {
                    <strong class="font-semibold text-gray-900 dark:text-gray-100">{bold_content}</strong>
                });
            } else {
                current_text.push_str("**");
                current_text.push_str(&bold_content);
            }
        } else {
            current_text.push(ch);
        }
    }
    
    if !current_text.is_empty() {
        result.push(html! { <span>{current_text}</span> });
    }
    
    if result.is_empty() {
        html! { <span>{text}</span> }
    } else {
        html! { <>{for result}</> }
    }
}

fn format_timestamp(timestamp: f64) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(timestamp));
    let hours = date.get_hours();
    let minutes = date.get_minutes();
    format!("{:02}:{:02}", hours, minutes)
}