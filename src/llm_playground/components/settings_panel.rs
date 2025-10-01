use yew::prelude::*;
use web_sys::HtmlInputElement;
use crate::llm_playground::{ApiConfig, ApiProvider};
use crate::llm_playground::types::FunctionTool;
use crate::llm_playground::components::{FunctionToolEditor, VisualFunctionToolEditor};

#[derive(Properties, PartialEq)]
pub struct SettingsPanelProps {
    pub config: ApiConfig,
    pub on_save: Callback<ApiConfig>,
    pub on_close: Callback<()>,
}

#[function_component(SettingsPanel)]
pub fn settings_panel(props: &SettingsPanelProps) -> Html {
    let config = use_state(|| props.config.clone());
    let show_function_editor = use_state(|| false);
    let editing_function_index = use_state(|| None::<usize>);
    let use_visual_editor = use_state(|| true); // Default to visual editor

    // Update local state when props change
    {
        let config = config.clone();
        let props_config = props.config.clone();
        use_effect_with(
            props_config,
            move |props_config| {
                config.set(props_config.clone());
                || ()
            },
        );
    }

    let on_close = {
        let callback = props.on_close.clone();
        Callback::from(move |_| {
            callback.emit(());
        })
    };

    let on_save = {
        let callback = props.on_save.clone();
        let config = config.clone();
        Callback::from(move |_| {
            callback.emit((*config).clone());
        })
    };

    // API Provider change
    let on_provider_change = {
        let config = config.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_config = (*config).clone();
            new_config.current_provider = match input.value().as_str() {
                "gemini" => ApiProvider::Gemini,
                "openai" => ApiProvider::OpenAI,
                _ => ApiProvider::Gemini,
            };
            config.set(new_config);
        })
    };

    // Gemini config changes
    let on_gemini_key_change = {
        let config = config.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_config = (*config).clone();
            new_config.gemini.api_key = input.value();
            config.set(new_config);
        })
    };

    let on_gemini_model_change = {
        let config = config.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_config = (*config).clone();
            new_config.gemini.model = input.value();
            config.set(new_config);
        })
    };

    // OpenAI config changes
    let on_openai_url_change = {
        let config = config.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_config = (*config).clone();
            new_config.openai.base_url = input.value();
            config.set(new_config);
        })
    };

    let on_openai_key_change = {
        let config = config.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_config = (*config).clone();
            new_config.openai.api_key = input.value();
            config.set(new_config);
        })
    };

    let on_openai_model_change = {
        let config = config.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_config = (*config).clone();
            new_config.openai.model = input.value();
            config.set(new_config);
        })
    };

    // Shared settings changes
    let on_temperature_change = {
        let config = config.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Ok(temp) = input.value().parse::<f32>() {
                let mut new_config = (*config).clone();
                new_config.shared_settings.temperature = temp;
                config.set(new_config);
            }
        })
    };

    let on_max_tokens_change = {
        let config = config.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Ok(tokens) = input.value().parse::<u32>() {
                let mut new_config = (*config).clone();
                new_config.shared_settings.max_tokens = tokens;
                config.set(new_config);
            }
        })
    };

    let on_retry_delay_change = {
        let config = config.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Ok(delay) = input.value().parse::<u32>() {
                let mut new_config = (*config).clone();
                new_config.shared_settings.retry_delay = delay;
                config.set(new_config);
            }
        })
    };

    let on_system_prompt_change = {
        let config = config.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_config = (*config).clone();
            new_config.system_prompt = input.value();
            config.set(new_config);
        })
    };

    // Function tool management callbacks
    let add_function_tool = {
        let show_function_editor = show_function_editor.clone();
        let editing_function_index = editing_function_index.clone();
        Callback::from(move |_| {
            editing_function_index.set(None);
            show_function_editor.set(true);
        })
    };

    let edit_function_tool = {
        let show_function_editor = show_function_editor.clone();
        let editing_function_index = editing_function_index.clone();
        Callback::from(move |index: usize| {
            editing_function_index.set(Some(index));
            show_function_editor.set(true);
        })
    };

    let delete_function_tool = {
        let config = config.clone();
        Callback::from(move |index: usize| {
            let mut new_config = (*config).clone();
            if index < new_config.function_tools.len() {
                new_config.function_tools.remove(index);
                config.set(new_config);
            }
        })
    };

    let save_function_tool = {
        let config = config.clone();
        let show_function_editor = show_function_editor.clone();
        let editing_function_index = editing_function_index.clone();
        Callback::from(move |tool: FunctionTool| {
            let mut new_config = (*config).clone();
            
            if let Some(index) = *editing_function_index {
                // Edit existing tool
                if index < new_config.function_tools.len() {
                    new_config.function_tools[index] = tool;
                }
            } else {
                // Add new tool
                new_config.function_tools.push(tool);
            }
            
            config.set(new_config);
            show_function_editor.set(false);
            editing_function_index.set(None);
        })
    };

    let cancel_function_editor = {
        let show_function_editor = show_function_editor.clone();
        let editing_function_index = editing_function_index.clone();
        Callback::from(move |_| {
            show_function_editor.set(false);
            editing_function_index.set(None);
        })
    };

    html! {
        <div class="absolute inset-y-0 right-0 w-96 bg-white dark:bg-gray-800 border-l border-gray-200 dark:border-gray-700 overflow-y-auto custom-scrollbar z-50">
            <div class="p-4 border-b border-gray-200 dark:border-gray-700">
                <div class="flex justify-between items-center">
                    <h2 class="text-lg font-semibold text-gray-900 dark:text-gray-100">{"Settings"}</h2>
                    <button 
                        onclick={on_close}
                        class="p-2 rounded-md hover:bg-gray-100 dark:hover:bg-gray-700"
                    >
                        <i class="fas fa-times"></i>
                    </button>
                </div>
            </div>
            
            <div class="p-4 space-y-6">
                // API Selection
                <div>
                    <h3 class="font-medium mb-2 text-gray-900 dark:text-gray-100">{"API Configuration"}</h3>
                    <div class="space-y-4">
                        <div class="flex items-center">
                            <input 
                                type="radio" 
                                id="gemini-api" 
                                name="api-type" 
                                value="gemini"
                                checked={config.current_provider == ApiProvider::Gemini}
                                onchange={on_provider_change.clone()}
                                class="mr-2" 
                            />
                            <label for="gemini-api">{"Gemini API"}</label>
                        </div>
                        <div class="flex items-center">
                            <input 
                                type="radio" 
                                id="openai-api" 
                                name="api-type" 
                                value="openai"
                                checked={config.current_provider == ApiProvider::OpenAI}
                                onchange={on_provider_change}
                                class="mr-2"
                            />
                            <label for="openai-api">{"OpenAI-compatible API"}</label>
                        </div>
                    </div>
                </div>
                
                // Gemini Config
                {if config.current_provider == ApiProvider::Gemini {
                    html! {
                        <div>
                            <div class="mb-4">
                                <label class="block text-sm font-medium mb-1 text-gray-700 dark:text-gray-300" for="gemini-key">{"API Key"}</label>
                                <input 
                                    type="password" 
                                    id="gemini-key" 
                                    value={config.gemini.api_key.clone()}
                                    oninput={on_gemini_key_change}
                                    class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100" 
                                    placeholder="Enter your Gemini API key"
                                />
                            </div>
                            <div class="mb-4">
                                <label class="block text-sm font-medium mb-1 text-gray-700 dark:text-gray-300" for="gemini-model">{"Model"}</label>
                                <select 
                                    id="gemini-model" 
                                    value={config.gemini.model.clone()}
                                    onchange={on_gemini_model_change}
                                    class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                                >
                                    <option value="gemini-2.5-flash-lite-preview-06-17">{"Gemini 2.5 Flash (Experimental)"}</option>
                                    <option value="gemini-1.5-pro">{"Gemini 1.5 Pro"}</option>
                                    <option value="gemini-1.5-flash">{"Gemini 1.5 Flash"}</option>
                                    <option value="gemini-1.0-pro">{"Gemini 1.0 Pro"}</option>
                                </select>
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }}
                
                // OpenAI Config
                {if config.current_provider == ApiProvider::OpenAI {
                    html! {
                        <div>
                            <div class="mb-4">
                                <label class="block text-sm font-medium mb-1 text-gray-700 dark:text-gray-300" for="openai-url">{"API URL"}</label>
                                <input 
                                    type="text" 
                                    id="openai-url" 
                                    value={config.openai.base_url.clone()}
                                    oninput={on_openai_url_change}
                                    class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100" 
                                    placeholder="https://api.openai.com/v1"
                                />
                            </div>
                            <div class="mb-4">
                                <label class="block text-sm font-medium mb-1 text-gray-700 dark:text-gray-300" for="openai-key">{"API Key"}</label>
                                <input 
                                    type="password" 
                                    id="openai-key" 
                                    value={config.openai.api_key.clone()}
                                    oninput={on_openai_key_change}
                                    class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100" 
                                    placeholder="Enter your API key"
                                />
                            </div>
                            <div class="mb-4">
                                <label class="block text-sm font-medium mb-1 text-gray-700 dark:text-gray-300" for="openai-model">{"Model"}</label>
                                <select 
                                    id="openai-model" 
                                    value={config.openai.model.clone()}
                                    onchange={on_openai_model_change}
                                    class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                                >
                                    <option value="gpt-4o">{"GPT-4o"}</option>
                                    <option value="gpt-4-turbo">{"GPT-4 Turbo"}</option>
                                    <option value="gpt-3.5-turbo">{"GPT-3.5 Turbo"}</option>
                                </select>
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }}
                
                // General Settings
                <div>
                    <h3 class="font-medium mb-2 text-gray-900 dark:text-gray-100">{"General Settings"}</h3>
                    <div class="mb-4">
                        <label class="block text-sm font-medium mb-1 text-gray-700 dark:text-gray-300" for="temperature">
                            {format!("Temperature: {:.1}", config.shared_settings.temperature)}
                        </label>
                        <input 
                            type="range" 
                            id="temperature" 
                            min="0" 
                            max="1" 
                            step="0.1" 
                            value={config.shared_settings.temperature.to_string()}
                            oninput={on_temperature_change}
                            class="w-full"
                        />
                    </div>
                    <div class="mb-4">
                        <label class="block text-sm font-medium mb-1 text-gray-700 dark:text-gray-300" for="max-tokens">{"Max Tokens"}</label>
                        <input 
                            type="number" 
                            id="max-tokens" 
                            value={config.shared_settings.max_tokens.to_string()}
                            oninput={on_max_tokens_change}
                            class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                        />
                    </div>
                    <div class="mb-4">
                        <label class="block text-sm font-medium mb-1 text-gray-700 dark:text-gray-300" for="retry-delay">{"Retry Delay (ms)"}</label>
                        <input 
                            type="number" 
                            id="retry-delay" 
                            value={config.shared_settings.retry_delay.to_string()}
                            oninput={on_retry_delay_change}
                            class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                        />
                    </div>
                </div>
                
                // System Prompt
                <div>
                    <h3 class="font-medium mb-2 text-gray-900 dark:text-gray-100">{"System Prompt"}</h3>
                    <textarea 
                        id="system-prompt" 
                        value={config.system_prompt.clone()}
                        oninput={on_system_prompt_change}
                        class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 h-32" 
                        placeholder="Enter system prompt"
                    />
                </div>
                
                // Function Tools
                <div>
                    <div class="flex items-center justify-between mb-4">
                        <h3 class="font-medium text-gray-900 dark:text-gray-100">{"Function Tools"}</h3>
                        <div class="flex items-center space-x-2">
                            <span class="text-sm text-gray-600 dark:text-gray-300">{"Editor:"}</span>
                            <button 
                                onclick={
                                    let use_visual_editor = use_visual_editor.clone();
                                    Callback::from(move |_| {
                                        use_visual_editor.set(!*use_visual_editor);
                                    })
                                }
                                class={classes!(
                                    "px-3", "py-1", "text-xs", "rounded-md", "transition-colors",
                                    if *use_visual_editor {
                                        "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400"
                                    } else {
                                        "bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300"
                                    }
                                )}
                                title="Toggle between visual form editor and JSON editor"
                            >
                                {if *use_visual_editor { "📝 Visual" } else { "⚡ JSON" }}
                            </button>
                        </div>
                    </div>
                    {for config.function_tools.iter().enumerate().map(|(index, tool)| {
                        let edit_callback = edit_function_tool.clone();
                        let delete_callback = delete_function_tool.clone();
                        
                        let edit_click = {
                            let edit_callback = edit_callback.clone();
                            Callback::from(move |_| edit_callback.emit(index))
                        };
                        
                        let delete_click = {
                            let delete_callback = delete_callback.clone();
                            Callback::from(move |_| delete_callback.emit(index))
                        };
                        
                        html! {
                            <div key={index} class={format!("p-4 rounded-md mb-3 border {}",
                                if tool.is_builtin {
                                    "bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-700"
                                } else {
                                    "bg-gray-100 dark:bg-gray-700 border-gray-200 dark:border-gray-600"
                                }
                            )}>
                                <div class="flex items-start justify-between mb-2">
                                    <div class="flex-1">
                                        <div class="flex items-center mb-1">
                                            {if tool.is_builtin {
                                                html! { <i class="fas fa-cog text-blue-500 mr-2" title="Built-in tool"></i> }
                                            } else {
                                                html! { <i class="fas fa-function text-purple-500 mr-2"></i> }
                                            }}
                                            <span class="font-medium text-lg text-gray-900 dark:text-gray-100">{&tool.name}</span>
                                            {if tool.is_builtin {
                                                html! {
                                                    <span class="ml-2 px-2 py-1 text-xs bg-blue-100 dark:bg-blue-800 text-blue-600 dark:text-blue-200 rounded-full">
                                                        {"Built-in"}
                                                    </span>
                                                }
                                            } else {
                                                html! {}
                                            }}
                                        </div>
                                        <p class="text-sm text-gray-600 dark:text-gray-300 mb-2">{&tool.description}</p>
                                        
                                        // Show parameter count
                                        <div class="text-xs text-gray-500 dark:text-gray-400">
                                            {
                                                if let Some(properties) = tool.parameters.get("properties") {
                                                    if let Some(obj) = properties.as_object() {
                                                        format!("{} parameter(s)", obj.len())
                                                    } else {
                                                        "No parameters".to_string()
                                                    }
                                                } else {
                                                    "No parameters".to_string()
                                                }
                                            }
                                        </div>
                                    </div>
                                    <div class="flex space-x-2 ml-4">
                                        {if tool.is_builtin {
                                            html! {
                                                <div class="text-xs px-2 py-1 bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded">
                                                    <i class="fas fa-lock mr-1"></i>
                                                    {"Protected"}
                                                </div>
                                            }
                                        } else {
                                            html! {
                                                <>
                                                    <button 
                                                        onclick={edit_click}
                                                        class="text-xs px-2 py-1 bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded hover:bg-blue-200 dark:hover:bg-blue-900/50"
                                                        title="Edit function"
                                                    >
                                                        <i class="fas fa-edit"></i>
                                                    </button>
                                                    <button 
                                                        onclick={delete_click}
                                                        class="text-xs px-2 py-1 bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400 rounded hover:bg-red-200 dark:hover:bg-red-900/50"
                                                        title="Delete function"
                                                    >
                                                        <i class="fas fa-trash"></i>
                                                    </button>
                                                </>
                                            }
                                        }}
                                    </div>
                                </div>
                            </div>
                        }
                    })}
                    
                    <button 
                        onclick={add_function_tool}
                        class="flex items-center justify-center w-full p-3 border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-md text-gray-500 dark:text-gray-400 hover:border-primary-500 hover:text-primary-500 dark:hover:border-primary-400 dark:hover:text-primary-400 transition-colors"
                    >
                        <i class="fas fa-plus mr-2"></i> {"Add Function Tool"}
                    </button>
                </div>
                
                // Save Button
                <div class="pt-4">
                    <button 
                        onclick={on_save}
                        class="w-full bg-primary-600 hover:bg-primary-700 text-white py-2 px-4 rounded-md"
                    >
                        {"Save Configuration"}
                    </button>
                </div>
            </div>
            
            // Function Tool Editor Modal
            {if *show_function_editor {
                let editing_tool = if let Some(index) = *editing_function_index {
                    config.function_tools.get(index).cloned()
                } else {
                    None
                };
                
                if *use_visual_editor {
                    html! {
                        <VisualFunctionToolEditor
                            tool={editing_tool}
                            on_save={save_function_tool}
                            on_cancel={cancel_function_editor}
                        />
                    }
                } else {
                    html! {
                        <FunctionToolEditor
                            tool={editing_tool}
                            on_save={save_function_tool}
                            on_cancel={cancel_function_editor}
                        />
                    }
                }
            } else {
                html! {}
            }}
        </div>
    }
}