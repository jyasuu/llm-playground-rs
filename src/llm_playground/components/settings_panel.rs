use yew::prelude::*;
use web_sys::HtmlInputElement;
use crate::llm_playground::{ApiConfig, ApiProvider, GeminiConfig, OpenAIConfig, SharedSettings, FunctionTool};

#[derive(Properties, PartialEq)]
pub struct SettingsPanelProps {
    pub config: ApiConfig,
    pub on_save: Callback<ApiConfig>,
    pub on_close: Callback<()>,
}

#[function_component(SettingsPanel)]
pub fn settings_panel(props: &SettingsPanelProps) -> Html {
    let config = use_state(|| props.config.clone());

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
                // API Selection
                <div>
                    <h3 class="font-medium mb-2">{"API Configuration"}</h3>
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
                                <label class="block text-sm font-medium mb-1" for="gemini-key">{"API Key"}</label>
                                <input 
                                    type="password" 
                                    id="gemini-key" 
                                    value={config.gemini.api_key.clone()}
                                    oninput={on_gemini_key_change}
                                    class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700" 
                                    placeholder="Enter your Gemini API key"
                                />
                            </div>
                            <div class="mb-4">
                                <label class="block text-sm font-medium mb-1" for="gemini-model">{"Model"}</label>
                                <select 
                                    id="gemini-model" 
                                    value={config.gemini.model.clone()}
                                    onchange={on_gemini_model_change}
                                    class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700"
                                >
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
                                <label class="block text-sm font-medium mb-1" for="openai-url">{"API URL"}</label>
                                <input 
                                    type="text" 
                                    id="openai-url" 
                                    value={config.openai.base_url.clone()}
                                    oninput={on_openai_url_change}
                                    class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700" 
                                    placeholder="https://api.openai.com/v1"
                                />
                            </div>
                            <div class="mb-4">
                                <label class="block text-sm font-medium mb-1" for="openai-key">{"API Key"}</label>
                                <input 
                                    type="password" 
                                    id="openai-key" 
                                    value={config.openai.api_key.clone()}
                                    oninput={on_openai_key_change}
                                    class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700" 
                                    placeholder="Enter your API key"
                                />
                            </div>
                            <div class="mb-4">
                                <label class="block text-sm font-medium mb-1" for="openai-model">{"Model"}</label>
                                <select 
                                    id="openai-model" 
                                    value={config.openai.model.clone()}
                                    onchange={on_openai_model_change}
                                    class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-700"
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
                
                // Mock Functions
                <div>
                    <h3 class="font-medium mb-2">{"Function Tools"}</h3>
                    {for config.function_tools.iter().map(|tool| {
                        html! {
                            <div class="bg-gray-100 dark:bg-gray-700 p-4 rounded-md mb-4">
                                <div class="flex items-center justify-between mb-2">
                                    <span class="font-medium">{&tool.name}</span>
                                    <button class="text-xs text-primary-600 dark:text-primary-400">{"Edit"}</button>
                                </div>
                                <p class="text-sm">{&tool.description}</p>
                            </div>
                        }
                    })}
                    <button class="flex items-center text-primary-600 dark:text-primary-400 text-sm">
                        <i class="fas fa-plus mr-1"></i> {"Add Function"}
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
        </div>
    }
}