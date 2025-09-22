use yew::prelude::*;
use web_sys::HtmlInputElement;
use crate::llm_playground::provider_config::FlexibleApiConfig;

#[derive(Properties, PartialEq)]
pub struct ModelSelectorProps {
    pub config: FlexibleApiConfig,
    pub on_select: Callback<(String, String)>, // (provider_name, model_name)
    pub on_cancel: Callback<()>,
    pub show: bool,
}

#[function_component(ModelSelector)]
pub fn model_selector(props: &ModelSelectorProps) -> Html {
    let selected_provider = use_state(|| {
        if let Some(provider) = props.config.providers.first() {
            provider.name.clone()
        } else {
            String::new()
        }
    });
    
    let selected_model = use_state(|| {
        if let Some(provider) = props.config.providers.first() {
            if let Some(model) = provider.models.first() {
                model.clone()
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    });

    // Update model list when provider changes
    {
        let selected_model = selected_model.clone();
        let config = props.config.clone();
        let selected_provider = selected_provider.clone();
        use_effect_with(selected_provider.clone(), move |provider_name| {
            if let Some(provider) = config.get_provider(provider_name) {
                if let Some(first_model) = provider.models.first() {
                    selected_model.set(first_model.clone());
                }
            }
            || ()
        });
    }

    let on_provider_change = {
        let selected_provider = selected_provider.clone();
        Callback::from(move |e: Event| {
            let select: HtmlInputElement = e.target_unchecked_into();
            selected_provider.set(select.value());
        })
    };

    let on_model_change = {
        let selected_model = selected_model.clone();
        Callback::from(move |e: Event| {
            let select: HtmlInputElement = e.target_unchecked_into();
            selected_model.set(select.value());
        })
    };

    let on_confirm = {
        let on_select = props.on_select.clone();
        let selected_provider = selected_provider.clone();
        let selected_model = selected_model.clone();
        Callback::from(move |_| {
            on_select.emit(((*selected_provider).clone(), (*selected_model).clone()));
        })
    };

    let on_cancel = {
        let callback = props.on_cancel.clone();
        Callback::from(move |_| {
            callback.emit(());
        })
    };

    if !props.show {
        return html! {};
    }

    // Get current provider for model list
    let current_provider = props.config.get_provider(&selected_provider);
    let available_models = current_provider
        .map(|p| p.models.clone())
        .unwrap_or_default();

    html! {
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-md w-full mx-4">
                <div class="p-6">
                    <div class="flex items-center justify-between mb-6">
                        <h2 class="text-xl font-semibold text-gray-900 dark:text-white">
                            {"Select Model for New Session"}
                        </h2>
                        <button 
                            onclick={on_cancel.clone()}
                            class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                        >
                            <i class="fas fa-times text-lg"></i>
                        </button>
                    </div>

                    <div class="space-y-4">
                        // Provider Selection
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                {"Provider"}
                            </label>
                            <select 
                                value={(*selected_provider).clone()}
                                onchange={on_provider_change}
                                class="w-full p-3 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                            >
                                {for props.config.providers.iter().map(|provider| {
                                    // Determine provider display info
                                    let transformer_type = provider.transformer.r#use.first()
                                        .map(|t| format!(" ({})", t))
                                        .unwrap_or_default();
                                    
                                    let display_name = format!("{}{}", provider.name, transformer_type);
                                    
                                    html! {
                                        <option key={provider.name.clone()} value={provider.name.clone()}>
                                            {display_name}
                                        </option>
                                    }
                                })}
                            </select>
                            
                            // Show provider info
                            {if let Some(provider) = current_provider {
                                html! {
                                    <div class="mt-2 text-xs text-gray-500 dark:text-gray-400">
                                        <div>{"API: "}{&provider.api_base_url}</div>
                                        <div>{"Type: "}{provider.transformer.r#use.join(", ")}</div>
                                        <div>
                                            {"Status: "}
                                            {if provider.api_key.is_empty() && provider.name != "ollama" {
                                                html! { <span class="text-red-500">{"⚠️ No API key configured"}</span> }
                                            } else {
                                                html! { <span class="text-green-500">{"✅ Configured"}</span> }
                                            }}
                                        </div>
                                    </div>
                                }
                            } else {
                                html! {}
                            }}
                        </div>

                        // Model Selection
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                {"Model"}
                            </label>
                            <select 
                                value={(*selected_model).clone()}
                                onchange={on_model_change}
                                class="w-full p-3 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                                disabled={available_models.is_empty()}
                            >
                                {if available_models.is_empty() {
                                    html! {
                                        <option>{"No models available"}</option>
                                    }
                                } else {
                                    html! {
                                        <>
                                            {for available_models.iter().map(|model| {
                                                html! {
                                                    <option key={model.clone()} value={model.clone()}>
                                                        {model}
                                                    </option>
                                                }
                                            })}
                                        </>
                                    }
                                }}
                            </select>
                        </div>

                        // Current Selection Summary
                        <div class="p-3 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-md">
                            <div class="text-sm text-blue-800 dark:text-blue-200">
                                <div class="font-medium mb-1">{"Selected Configuration:"}</div>
                                <div>{"Provider: "}{&*selected_provider}</div>
                                <div>{"Model: "}{&*selected_model}</div>
                                {if let Some(provider) = current_provider {
                                    html! {
                                        <div class="mt-1 text-xs text-blue-600 dark:text-blue-300">
                                            {"API Type: "}{provider.transformer.r#use.join(", ")}
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }}
                            </div>
                        </div>

                        // Warning if API key missing
                        {if let Some(provider) = current_provider {
                            if provider.api_key.is_empty() && provider.name != "ollama" {
                                html! {
                                    <div class="p-3 bg-yellow-50 dark:bg-yellow-900/20 border border-yellow-200 dark:border-yellow-800 rounded-md">
                                        <div class="flex items-center text-yellow-800 dark:text-yellow-200">
                                            <i class="fas fa-exclamation-triangle mr-2"></i>
                                            <span class="text-sm">
                                                {"Warning: API key not configured for this provider. Configure it in Settings before using."}
                                            </span>
                                        </div>
                                    </div>
                                }
                            } else {
                                html! {}
                            }
                        } else {
                            html! {}
                        }}
                    </div>

                    // Action buttons
                    <div class="flex justify-end space-x-3 mt-6 pt-4 border-t border-gray-200 dark:border-gray-600">
                        <button 
                            onclick={on_cancel}
                            class="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-md transition-colors"
                        >
                            {"Cancel"}
                        </button>
                        <button 
                            onclick={on_confirm}
                            disabled={selected_provider.is_empty() || selected_model.is_empty()}
                            class="px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed rounded-md transition-colors"
                        >
                            {"Start Session"}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}