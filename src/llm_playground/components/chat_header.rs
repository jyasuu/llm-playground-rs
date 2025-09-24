use yew::prelude::*;
use crate::llm_playground::{ChatSession, ApiConfig, ApiProvider};

#[derive(Properties, PartialEq)]
pub struct ChatHeaderProps {
    pub current_session: Option<ChatSession>,
    pub api_config: ApiConfig,
    pub on_toggle_dark_mode: Callback<()>,
    pub dark_mode: bool,
}

#[function_component(ChatHeader)]
pub fn chat_header(props: &ChatHeaderProps) -> Html {
    let on_dark_mode_toggle = {
        let callback = props.on_toggle_dark_mode.clone();
        Callback::from(move |_| {
            callback.emit(());
        })
    };

    let (session_title, model_info) = if let Some(session) = &props.current_session {
        let model = match props.api_config.current_provider {
            ApiProvider::Gemini => &props.api_config.gemini.model,
            ApiProvider::OpenAI => &props.api_config.openai.model,
        };
        let provider = match props.api_config.current_provider {
            ApiProvider::Gemini => "Gemini",
            ApiProvider::OpenAI => "OpenAI",
        };
        (session.title.clone(), format!("Using {} {}", provider, model))
    } else {
        ("No Session Selected".to_string(), "Select or create a session to start chatting".to_string())
    };

    html! {
        <div class="p-4 border-b border-gray-200 dark:border-gray-600 flex justify-between items-center">
            <div>
                <h2 class="font-semibold text-gray-900 dark:text-gray-100">{session_title}</h2>
                <div class="text-sm text-gray-600 dark:text-gray-300">{model_info}</div>
            </div>
            <div class="flex space-x-2">
                <button 
                    onclick={on_dark_mode_toggle}
                    class="p-2 rounded-md hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-300"
                    title="Toggle dark mode"
                >
                    {if props.dark_mode {
                        html! { <i class="fas fa-sun"></i> }
                    } else {
                        html! { <i class="fas fa-moon"></i> }
                    }}
                </button>
                <button class="p-2 rounded-md hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-300">
                    <i class="fas fa-ellipsis-v"></i>
                </button>
            </div>
        </div>
    }
}