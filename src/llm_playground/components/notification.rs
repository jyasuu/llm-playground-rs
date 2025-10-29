// UI notification component for showing temporary messages
use yew::prelude::*;
use gloo_timers::future::TimeoutFuture;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum NotificationType {
    Info,
    Warning,
    Error,
    Success,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NotificationMessage {
    pub id: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub auto_dismiss: bool,
    pub duration_ms: u32,
}

#[derive(Properties, PartialEq)]
pub struct NotificationProps {
    pub notifications: HashMap<String, NotificationMessage>,
    pub on_dismiss: Callback<String>,
}

#[function_component(NotificationContainer)]
pub fn notification_container(props: &NotificationProps) -> Html {
    let notifications = &props.notifications;
    let on_dismiss = &props.on_dismiss;

    if notifications.is_empty() {
        return html! {};
    }

    html! {
        <div class="fixed top-4 right-4 z-50 space-y-2">
            {for notifications.values().map(|notification| {
                let notification_id = notification.id.clone();
                let on_dismiss_clone = on_dismiss.clone();
                
                let dismiss_callback = Callback::from(move |_: MouseEvent| {
                    on_dismiss_clone.emit(notification_id.clone());
                });

                let (bg_class, icon_class, icon) = match notification.notification_type {
                    NotificationType::Info => ("bg-blue-500", "text-blue-200", "fas fa-info-circle"),
                    NotificationType::Warning => ("bg-yellow-500", "text-yellow-200", "fas fa-exclamation-triangle"),
                    NotificationType::Error => ("bg-red-500", "text-red-200", "fas fa-times-circle"),
                    NotificationType::Success => ("bg-green-500", "text-green-200", "fas fa-check-circle"),
                };

                html! {
                    <div key={notification.id.clone()} class={classes!(
                        "flex", "items-center", "p-4", "rounded-lg", "shadow-lg", "text-white",
                        "min-w-80", "max-w-96", "transition-all", "duration-300", bg_class
                    )}>
                        <i class={classes!("mr-3", "text-lg", icon_class, icon)}></i>
                        <div class="flex-1 text-sm">
                            {&notification.message}
                        </div>
                        <button
                            onclick={dismiss_callback}
                            class="ml-3 text-white hover:text-gray-200 transition-colors"
                        >
                            <i class="fas fa-times"></i>
                        </button>
                    </div>
                }
            })}
        </div>
    }
}

impl NotificationMessage {
    pub fn new(message: String, notification_type: NotificationType) -> Self {
        Self {
            id: format!("notif_{}", js_sys::Date::now() as u64),
            message,
            notification_type,
            auto_dismiss: true,
            duration_ms: 5000,
        }
    }

    pub fn with_duration(mut self, duration_ms: u32) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    pub fn persistent(mut self) -> Self {
        self.auto_dismiss = false;
        self
    }
}

// Hook for managing notifications
#[hook]
pub fn use_notifications() -> (HashMap<String, NotificationMessage>, Callback<NotificationMessage>, Callback<String>) {
    let notifications = use_state(|| HashMap::<String, NotificationMessage>::new());
    
    let add_notification = {
        let notifications = notifications.clone();
        Callback::from(move |notification: NotificationMessage| {
            let mut new_notifications = (*notifications).clone();
            let notification_id = notification.id.clone();
            let auto_dismiss = notification.auto_dismiss;
            let duration_ms = notification.duration_ms;
            
            new_notifications.insert(notification.id.clone(), notification);
            notifications.set(new_notifications);
            
            // Auto-dismiss if configured
            if auto_dismiss {
                let notifications_clone = notifications.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    TimeoutFuture::new(duration_ms).await;
                    let mut current_notifications = (*notifications_clone).clone();
                    current_notifications.remove(&notification_id);
                    notifications_clone.set(current_notifications);
                });
            }
        })
    };
    
    let dismiss_notification = {
        let notifications = notifications.clone();
        Callback::from(move |notification_id: String| {
            let mut new_notifications = (*notifications).clone();
            new_notifications.remove(&notification_id);
            notifications.set(new_notifications);
        })
    };
    
    ((*notifications).clone(), add_notification, dismiss_notification)
}