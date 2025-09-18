use yew::prelude::*;
use std::collections::HashMap;
use crate::llm_playground::{ChatSession};

#[derive(Properties, PartialEq)]
pub struct SidebarProps {
    pub sessions: HashMap<String, ChatSession>,
    pub current_session_id: Option<String>,
    pub on_new_session: Callback<()>,
    pub on_select_session: Callback<String>,
    pub on_toggle_settings: Callback<()>,
}

#[function_component(Sidebar)]
pub fn sidebar(props: &SidebarProps) -> Html {
    // Sort sessions by updated_at (most recent first)
    let mut sessions_vec: Vec<_> = props.sessions.iter().collect();
    sessions_vec.sort_by(|a, b| b.1.updated_at.partial_cmp(&a.1.updated_at).unwrap_or(std::cmp::Ordering::Equal));
    
    let on_new_session = props.on_new_session.clone();
    let new_session_click = Callback::from(move |_| {
        on_new_session.emit(());
    });

    let on_settings_click = {
        let on_toggle_settings = props.on_toggle_settings.clone();
        Callback::from(move |_| {
            on_toggle_settings.emit(());
        })
    };

    html! {
        <div class="w-64 bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700 flex flex-col">
            // Header
            <div class="p-4 border-b border-gray-200 dark:border-gray-700">
                <h1 class="text-xl font-bold">{"LLM Playground"}</h1>
                <p class="text-sm text-gray-500 dark:text-gray-400">{"Local Storage Demo"}</p>
            </div>
            
            // Session List
            <div class="flex-1 overflow-y-auto custom-scrollbar">
                <div class="p-4">
                    <div class="flex justify-between items-center mb-2">
                        <h2 class="font-semibold">{"Sessions"}</h2>
                        <button 
                            onclick={new_session_click}
                            class="text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300"
                        >
                            <i class="fas fa-plus"></i>
                        </button>
                    </div>
                    <ul class="space-y-2">
                        {for sessions_vec.iter().map(|(session_id, session)| {
                            let is_current = props.current_session_id.as_ref() == Some(session_id);
                            let session_id_clone = (*session_id).clone();
                            let on_select = props.on_select_session.clone();
                            
                            let click_handler = Callback::from(move |_| {
                                on_select.emit(session_id_clone.clone());
                            });

                            let time_ago = format_time_ago(session.updated_at);
                            
                            html! {
                                <li 
                                    key={session.id.clone()}
                                    onclick={click_handler}
                                    class={classes!(
                                        "p-2", "rounded-md", "cursor-pointer",
                                        if is_current {
                                            "bg-primary-100 dark:bg-primary-900/30"
                                        } else {
                                            "hover:bg-gray-100 dark:hover:bg-gray-700"
                                        }
                                    )}
                                >
                                    <div class="font-medium">{&session.title}</div>
                                    <div class="text-xs text-gray-500 dark:text-gray-400">{time_ago}</div>
                                    {if session.pinned {
                                        html! { <i class="fas fa-thumbtack text-xs text-yellow-500"></i> }
                                    } else {
                                        html! {}
                                    }}
                                </li>
                            }
                        })}
                        {if sessions_vec.is_empty() {
                            html! {
                                <li class="p-4 text-center text-gray-500 dark:text-gray-400">
                                    <p>{"No sessions yet"}</p>
                                    <p class="text-sm">{"Click + to create one"}</p>
                                </li>
                            }
                        } else {
                            html! {}
                        }}
                    </ul>
                </div>
            </div>
            
            // Settings Button
            <div class="p-4 border-t border-gray-200 dark:border-gray-700">
                <button 
                    onclick={on_settings_click}
                    class="w-full py-2 px-4 bg-gray-100 dark:bg-gray-700 rounded-md hover:bg-gray-200 dark:hover:bg-gray-600 flex items-center justify-center"
                >
                    <i class="fas fa-cog mr-2"></i> {"Settings"}
                </button>
            </div>
        </div>
    }
}

fn format_time_ago(timestamp: f64) -> String {
    let now = js_sys::Date::now();
    let diff = now - timestamp;
    let seconds = diff / 1000.0;
    
    if seconds < 60.0 {
        "Just now".to_string()
    } else if seconds < 3600.0 {
        let minutes = (seconds / 60.0).floor() as i32;
        format!("{} min{} ago", minutes, if minutes == 1 { "" } else { "s" })
    } else if seconds < 86400.0 {
        let hours = (seconds / 3600.0).floor() as i32;
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else {
        let days = (seconds / 86400.0).floor() as i32;
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    }
}