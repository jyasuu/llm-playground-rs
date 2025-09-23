use yew::prelude::*;
use crate::llm_playground::ChatSession;
use super::message_bubble::MessageBubble;

#[derive(Properties, PartialEq)]
pub struct ChatRoomProps {
    pub session: Option<ChatSession>,
    pub is_loading: bool,
}

#[function_component(ChatRoom)]
pub fn chat_room(props: &ChatRoomProps) -> Html {
    let messages_container_ref = use_node_ref();

    // Auto-scroll to bottom when new messages arrive
    {
        let messages_container_ref = messages_container_ref.clone();
        let messages_len = props.session.as_ref().map(|s| s.messages.len()).unwrap_or(0);
        
        use_effect_with(
            messages_len,
            move |_| {
                if let Some(container) = messages_container_ref.cast::<web_sys::Element>() {
                    container.set_scroll_top(container.scroll_height());
                }
                || ()
            },
        );
    }

    html! {
        <div class="flex-1 overflow-hidden flex flex-col">
            <div 
                ref={messages_container_ref}
                class="chat-container overflow-y-auto p-4 space-y-6 custom-scrollbar"
                style="height: calc(100vh - 140px);"
            >
                {if let Some(session) = &props.session {
                    html! {
                        <>
                            {for session.messages.iter().map(|message| {
                                html! {
                                    <MessageBubble 
                                        key={message.id.clone()}
                                        message={message.clone()}
                                    />
                                }
                            })}
                            {if props.is_loading {
                                html! {
                                    <div class="flex">
                                        <div class="w-10 h-10 rounded-full bg-purple-100 dark:bg-purple-900/30 flex items-center justify-center mr-3">
                                            <i class="fas fa-robot text-purple-600 dark:text-purple-400"></i>
                                        </div>
                                        <div class="flex-1 bg-white dark:bg-gray-800 rounded-lg p-4 border border-gray-200 dark:border-gray-700">
                                            <div class="font-medium mb-1">{"Assistant"}</div>
                                            <div class="flex items-center space-x-2">
                                                <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-purple-600"></div>
                                                <span class="text-sm text-gray-500 dark:text-gray-400">{"Thinking..."}</span>
                                            </div>
                                        </div>
                                    </div>
                                }
                            } else {
                                html! {}
                            }}
                        </>
                    }
                } else {
                    html! {
                        <div class="flex items-center justify-center h-full">
                            <div class="text-center text-gray-500 dark:text-gray-400">
                                <i class="fas fa-comments text-4xl mb-4"></i>
                                <h3 class="text-lg font-medium mb-2">{"Welcome to LLM Playground"}</h3>
                                <p>{"Select a session from the sidebar or create a new one to start chatting."}</p>
                            </div>
                        </div>
                    }
                }}
            </div>
        </div>
    }
}