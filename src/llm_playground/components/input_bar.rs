use web_sys::{HtmlTextAreaElement, KeyboardEvent};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct InputBarProps {
    pub current_message: String,
    pub on_message_change: Callback<InputEvent>,
    pub on_send_message: Callback<()>,
    pub is_loading: bool,
}

#[function_component(InputBar)]
pub fn input_bar(props: &InputBarProps) -> Html {
    let textarea_ref = use_node_ref();

    let on_input = props.on_message_change.clone();

    let on_send = {
        let callback = props.on_send_message.clone();
        Callback::from(move |_| {
            callback.emit(());
        })
    };

    let on_keydown = {
        let on_send = props.on_send_message.clone();
        let is_loading = props.is_loading;

        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" && !e.shift_key() && !is_loading {
                e.prevent_default();
                on_send.emit(());
            }
        })
    };

    // Auto-resize textarea
    let on_input_resize = {
        let textarea_ref = textarea_ref.clone();
        Callback::from(move |_: InputEvent| {
            if let Some(textarea) = textarea_ref.cast::<HtmlTextAreaElement>() {
                // For now, let's skip the auto-resize functionality to get the basic app working
                // TODO: Implement proper auto-resize later
                let _ = textarea.scroll_height(); // Just to use the variable
            }
        })
    };

    // Combine input callbacks
    let combined_input = {
        let on_input = on_input.clone();
        let on_input_resize = on_input_resize.clone();

        Callback::from(move |e: InputEvent| {
            on_input.emit(e.clone());
            on_input_resize.emit(e);
        })
    };

    html! {
        <div class="p-4 border-t border-gray-200 dark:border-gray-700">
            <div class="flex items-end border border-gray-300 dark:border-gray-500 rounded-lg bg-white dark:bg-gray-800 p-2">
                <div class="flex-1">
                    <textarea
                        ref={textarea_ref}
                        class="w-full resize-none border-0 focus:ring-0 bg-transparent dark:bg-transparent p-2 text-sm text-gray-900 dark:text-gray-100"
                        rows="1"
                        placeholder="Type your message here..."
                        style="outline: none; min-height: 20px;"
                        value={props.current_message.clone()}
                        oninput={combined_input}
                        onkeydown={on_keydown}
                        disabled={props.is_loading}
                    />
                </div>
                <div class="flex items-center space-x-1">
                    <button
                        class="p-2 text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200"
                        title="Attach file (not implemented)"
                    >
                        <i class="fas fa-paperclip"></i>
                    </button>
                    <button
                        onclick={on_send}
                        disabled={props.current_message.trim().is_empty() || props.is_loading}
                        class={classes!(
                            "p-2", "rounded-md",
                            if props.current_message.trim().is_empty() || props.is_loading {
                                "text-gray-400 dark:text-gray-600 cursor-not-allowed"
                            } else {
                                "text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300 hover:bg-primary-50 dark:hover:bg-primary-900/20"
                            }
                        )}
                        title="Send message (Shift+Enter for new line)"
                    >
                        {if props.is_loading {
                            html! { <i class="fas fa-spinner fa-spin"></i> }
                        } else {
                            html! { <i class="fas fa-paper-plane"></i> }
                        }}
                    </button>
                </div>
            </div>
            <div class="text-xs text-gray-600 dark:text-gray-300 mt-2 flex justify-between">
                <span>
                    <i class="fas fa-keyboard mr-1"></i>
                    {"Enter to send • Shift+Enter for new line"}
                </span>
                {if !props.current_message.is_empty() {
                    html! {
                        <span>{format!("{} characters", props.current_message.len())}</span>
                    }
                } else {
                    html! {}
                }}
            </div>
        </div>
    }
}
