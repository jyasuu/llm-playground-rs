use yew::prelude::*;
use web_sys::HtmlInputElement;
use wasm_bindgen::JsCast;
use crate::llm_playground::provider_config::{FlexibleApiConfig, ProviderConfig};
use crate::llm_playground::types::FunctionTool;
use crate::llm_playground::components::{FunctionToolEditor, VisualFunctionToolEditor};

#[derive(Properties, PartialEq)]
pub struct FlexibleSettingsPanelProps {
    pub config: FlexibleApiConfig,
    pub on_save: Callback<FlexibleApiConfig>,
    pub on_close: Callback<()>,
}

#[function_component(FlexibleSettingsPanel)]
pub fn flexible_settings_panel(props: &FlexibleSettingsPanelProps) -> Html {
    let config = use_state(|| props.config.clone());
    let show_function_editor = use_state(|| false);
    let editing_function_index = use_state(|| None::<usize>);
    let use_visual_editor = use_state(|| true);
    let selected_provider_index = use_state(|| 0);
    let show_add_provider = use_state(|| false);

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

    // Provider management
    let on_provider_select = {
        let selected_provider_index = selected_provider_index.clone();
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Ok(index) = input.value().parse::<usize>() {
                selected_provider_index.set(index);
            }
        })
    };

    let on_provider_field_change = {
        let config = config.clone();
        let selected_provider_index = selected_provider_index.clone();
        Callback::from(move |(field, value): (String, String)| {
            let mut new_config = (*config).clone();
            let index = *selected_provider_index;
            if index < new_config.providers.len() {
                match field.as_str() {
                    "name" => new_config.providers[index].name = value,
                    "api_base_url" => new_config.providers[index].api_base_url = value,
                    "api_key" => new_config.providers[index].api_key = value,
                    _ => {}
                }
                config.set(new_config);
            }
        })
    };

    let on_add_model = {
        let config = config.clone();
        let selected_provider_index = selected_provider_index.clone();
        Callback::from(move |model: String| {
            if !model.trim().is_empty() {
                let mut new_config = (*config).clone();
                let index = *selected_provider_index;
                if index < new_config.providers.len() {
                    new_config.providers[index].models.push(model.trim().to_string());
                    config.set(new_config);
                }
            }
        })
    };

    let on_remove_model = {
        let config = config.clone();
        let selected_provider_index = selected_provider_index.clone();
        Callback::from(move |model_index: usize| {
            let mut new_config = (*config).clone();
            let provider_index = *selected_provider_index;
            if provider_index < new_config.providers.len() && model_index < new_config.providers[provider_index].models.len() {
                new_config.providers[provider_index].models.remove(model_index);
                config.set(new_config);
            }
        })
    };

    let on_add_provider = {
        let config = config.clone();
        let show_add_provider = show_add_provider.clone();
        Callback::from(move |_: MouseEvent| {
            let mut new_config = (*config).clone();
            new_config.providers.push(ProviderConfig {
                name: "new-provider".to_string(),
                api_base_url: "https://api.example.com/v1/chat/completions".to_string(),
                api_key: String::new(),
                models: vec!["model-1".to_string()],
                transformer: crate::llm_playground::provider_config::TransformerConfig {
                    r#use: vec!["openai".to_string()],
                },
            });
            config.set(new_config);
            show_add_provider.set(false);
        })
    };

    let on_remove_provider = {
        let config = config.clone();
        let selected_provider_index = selected_provider_index.clone();
        Callback::from(move |_| {
            let mut new_config = (*config).clone();
            let index = *selected_provider_index;
            if index < new_config.providers.len() && new_config.providers.len() > 1 {
                new_config.providers.remove(index);
                selected_provider_index.set(0);
                config.set(new_config);
            }
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

    // Function tool management callbacks (same as before)
    let add_function_tool = {
        let show_function_editor = show_function_editor.clone();
        let editing_function_index = editing_function_index.clone();
        Callback::from(move |_: MouseEvent| {
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
                if index < new_config.function_tools.len() {
                    new_config.function_tools[index] = tool;
                }
            } else {
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
        Callback::from(move |_: ()| {
            show_function_editor.set(false);
            editing_function_index.set(None);
        })
    };

    let current_provider = config.providers.get(*selected_provider_index);

    html! {
        <div class="absolute inset-y-0 right-0 w-96 bg-white dark:bg-gray-800 border-l border-gray-200 dark:border-gray-700 overflow-y-auto custom-scrollbar z-50">
            <div class="p-4 border-b border-gray-200 dark:border-gray-700">
                <div class="flex justify-between items-center">
                    <h2 class="text-lg font-semibold">{"Settings"}</h2>
                    <button 
                        onclick={on_close}
                        class="p-2 rounded-md hover:bg-gray-100 dark:hover:bg-gray-700"
                    >
                        <i class="fas fa-times"></i>
                    </button>
                </div>
            </div>
            
            <div class="p-4 space-y-6">
                // Provider Management
                <div>
                    <div class="flex justify-between items-center mb-4">
                        <h3 class="font-medium">{"LLM Providers"}</h3>
                        <div class="flex space-x-2">
                            <button 
                                onclick={
                                    let show_add_provider = show_add_provider.clone();
                                    Callback::from(move |_| show_add_provider.set(true))
                                }
                                class="text-xs px-2 py-1 bg-green-100 dark:bg-green-900/30 text-green-600 dark:text-green-400 rounded hover:bg-green-200 dark:hover:bg-green-900/50"
                            >
                                <i class="fas fa-plus mr-1"></i>{"Add"}
                            </button>
                            {if config.providers.len() > 1 {
                                html! {
                                    <button 
                                        onclick={on_remove_provider}
                                        class="text-xs px-2 py-1 bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400 rounded hover:bg-red-200 dark:hover:bg-red-900/50"
                                    >
                                        <i class="fas fa-trash mr-1"></i>{"Remove"}
                                    </button>
                                }
                            } else {
                                html! {}
                            }}
                        </div>
                    </div>
                    
                    // Provider selector
                    <div class="mb-4">
                        <label class="block text-sm font-medium mb-1">{"Select Provider"}</label>
                        <select 
                            value={selected_provider_index.to_string()}
                            onchange={on_provider_select}
                            class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700"
                        >
                            {for config.providers.iter().enumerate().map(|(index, provider)| {
                                html! {
                                    <option key={index} value={index.to_string()}>
                                        {&provider.name}
                                    </option>
                                }
                            })}
                        </select>
                    </div>

                    // Provider configuration
                    {if let Some(provider) = current_provider {
                        html! {
                            <div class="space-y-4 p-4 border border-gray-200 dark:border-gray-600 rounded-md">
                                <div>
                                    <label class="block text-sm font-medium mb-1">{"Provider Name"}</label>
                                    <input 
                                        type="text" 
                                        value={provider.name.clone()}
                                        oninput={
                                            let callback = on_provider_field_change.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                callback.emit(("name".to_string(), input.value()));
                                            })
                                        }
                                        class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700"
                                    />
                                </div>
                                
                                <div>
                                    <label class="block text-sm font-medium mb-1">{"API Base URL"}</label>
                                    <input 
                                        type="text" 
                                        value={provider.api_base_url.clone()}
                                        oninput={
                                            let callback = on_provider_field_change.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                callback.emit(("api_base_url".to_string(), input.value()));
                                            })
                                        }
                                        class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700"
                                        placeholder="https://api.example.com/v1/chat/completions"
                                    />
                                </div>
                                
                                <div>
                                    <label class="block text-sm font-medium mb-1">{"API Key"}</label>
                                    <input 
                                        type="password" 
                                        value={provider.api_key.clone()}
                                        oninput={
                                            let callback = on_provider_field_change.clone();
                                            Callback::from(move |e: InputEvent| {
                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                callback.emit(("api_key".to_string(), input.value()));
                                            })
                                        }
                                        class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700"
                                        placeholder="Enter API key"
                                    />
                                </div>
                                
                                // Transformer type
                                <div>
                                    <label class="block text-sm font-medium mb-1">{"API Type"}</label>
                                    <div class="text-sm text-gray-600 dark:text-gray-300">
                                        {format!("Uses: {}", provider.transformer.r#use.join(", "))}
                                    </div>
                                </div>
                                
                                // Models management
                                <div>
                                    <label class="block text-sm font-medium mb-2">{"Available Models"}</label>
                                    <div class="space-y-2">
                                        {for provider.models.iter().enumerate().map(|(model_index, model)| {
                                            let remove_callback = on_remove_model.clone();
                                            html! {
                                                <div key={model_index} class="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-600 rounded">
                                                    <span class="text-sm">{model}</span>
                                                    <button 
                                                        onclick={
                                                            Callback::from(move |_| remove_callback.emit(model_index))
                                                        }
                                                        class="text-xs px-1 py-0.5 text-red-600 hover:text-red-800"
                                                    >
                                                        <i class="fas fa-times"></i>
                                                    </button>
                                                </div>
                                            }
                                        })}
                                        
                                        // Add model input
                                        <div class="flex space-x-2">
                                            <input 
                                                type="text" 
                                                id={format!("new-model-{}", *selected_provider_index)}
                                                class="flex-1 p-2 text-sm border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700"
                                                placeholder="Add new model..."
                                                onkeypress={
                                                    let add_callback = on_add_model.clone();
                                                    Callback::from(move |e: KeyboardEvent| {
                                                        if e.key() == "Enter" {
                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                            let value = input.value();
                                                            if !value.trim().is_empty() {
                                                                add_callback.emit(value);
                                                                input.set_value("");
                                                            }
                                                        }
                                                    })
                                                }
                                            />
                                            <button 
                                                onclick={
                                                    let add_callback = on_add_model.clone();
                                                    let selected_index = *selected_provider_index;
                                                    Callback::from(move |_| {
                                                        if let Some(input) = web_sys::window()
                                                            .and_then(|w| w.document())
                                                            .and_then(|d| d.get_element_by_id(&format!("new-model-{}", selected_index)))
                                                            .and_then(|e| e.dyn_into::<HtmlInputElement>().ok()) {
                                                            let value = input.value();
                                                            if !value.trim().is_empty() {
                                                                add_callback.emit(value);
                                                                input.set_value("");
                                                            }
                                                        }
                                                    })
                                                }
                                                class="px-3 py-2 text-sm bg-blue-500 text-white rounded hover:bg-blue-600"
                                            >
                                                <i class="fas fa-plus"></i>
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }
                    } else {
                        html! {}
                    }}
                </div>
                
                // General Settings (same as before)
                <div>
                    <h3 class="font-medium mb-2">{"General Settings"}</h3>
                    <div class="mb-4">
                        <label class="block text-sm font-medium mb-1" for="temperature">
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
                        <label class="block text-sm font-medium mb-1" for="max-tokens">{"Max Tokens"}</label>
                        <input 
                            type="number" 
                            id="max-tokens" 
                            value={config.shared_settings.max_tokens.to_string()}
                            oninput={on_max_tokens_change}
                            class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700"
                        />
                    </div>
                    <div class="mb-4">
                        <label class="block text-sm font-medium mb-1" for="retry-delay">{"Retry Delay (ms)"}</label>
                        <input 
                            type="number" 
                            id="retry-delay" 
                            value={config.shared_settings.retry_delay.to_string()}
                            oninput={on_retry_delay_change}
                            class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700"
                        />
                    </div>
                </div>
                
                // System Prompt
                <div>
                    <h3 class="font-medium mb-2">{"System Prompt"}</h3>
                    <textarea 
                        id="system-prompt" 
                        value={config.system_prompt.clone()}
                        oninput={on_system_prompt_change}
                        class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 h-32" 
                        placeholder="Enter system prompt"
                    />
                </div>
                
                // Function Tools
                <div>
                    <div class="flex items-center justify-between mb-4">
                        <h3 class="font-medium">{"Function Tools"}</h3>
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
                            <div key={index} class="bg-gray-100 dark:bg-gray-700 p-4 rounded-md mb-3 border border-gray-200 dark:border-gray-600">
                                <div class="flex items-start justify-between mb-2">
                                    <div class="flex-1">
                                        <div class="flex items-center mb-1">
                                            <i class="fas fa-function text-purple-500 mr-2"></i>
                                            <span class="font-medium text-lg">{&tool.name}</span>
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