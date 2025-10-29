// MCP Settings Panel Component
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::llm_playground::{
    mcp_client::{McpClient, McpServerConfig},
    types::ApiConfig,
};

#[derive(Properties, PartialEq)]
pub struct McpSettingsPanelProps {
    pub config: ApiConfig,
    pub on_config_change: Callback<ApiConfig>,
    pub mcp_client: Option<McpClient>,
    pub on_mcp_client_change: Callback<Option<McpClient>>,
}

#[function_component(McpSettingsPanel)]
pub fn mcp_settings_panel(props: &McpSettingsPanelProps) -> Html {
    let config = props.config.clone();
    let mcp_config = config.get_mcp_config().clone();

    let new_server_name = use_state(|| String::new());
    let new_server_url = use_state(|| String::new());
    let new_server_auth_token = use_state(|| String::new());
    let new_server_headers_json = use_state(|| String::from("{}"));
    let show_add_server = use_state(|| false);
    let show_json_editor = use_state(|| false);
    let editing_server = use_state(|| None::<String>);
    let connection_status = use_state(|| HashMap::<String, String>::new());

    // Handle adding a new server
    let on_add_server = {
        let config = config.clone();
        let on_config_change = props.on_config_change.clone();
        let new_server_name = new_server_name.clone();
        let new_server_url = new_server_url.clone();
        let new_server_auth_token = new_server_auth_token.clone();
        let new_server_headers_json = new_server_headers_json.clone();
        let show_add_server = show_add_server.clone();
        let editing_server = editing_server.clone();

        Callback::from(move |_| {
            let name = (*new_server_name).clone();
            let url = (*new_server_url).clone();
            let auth_token = (*new_server_auth_token).clone();
            let headers_json = (*new_server_headers_json).clone();

            if !name.is_empty() && !url.is_empty() {
                let mut new_config = config.clone();
                let mut headers = HashMap::new();

                // Parse custom headers from JSON
                if let Ok(parsed_headers) = serde_json::from_str::<serde_json::Value>(&headers_json)
                {
                    if let Some(headers_obj) = parsed_headers.as_object() {
                        for (key, value) in headers_obj {
                            if let Some(value_str) = value.as_str() {
                                headers.insert(key.clone(), value_str.to_string());
                            }
                        }
                    }
                }

                // Add auth token if provided (will override JSON if same key exists)
                if !auth_token.is_empty() {
                    headers.insert(
                        "Authorization".to_string(),
                        format!("Bearer {}", auth_token),
                    );
                }

                let server_config = McpServerConfig {
                    name: name.clone(),
                    server_type: "http".to_string(),
                    url: Some(url),
                    headers: if headers.is_empty() {
                        None
                    } else {
                        Some(headers)
                    },
                    enabled: true,
                };

                let mut updated_mcp_config = new_config.get_mcp_config().clone();

                // Check if we're editing an existing server
                if let Some(editing_name) = editing_server.as_ref() {
                    updated_mcp_config.servers.remove(editing_name);
                }

                updated_mcp_config.servers.insert(name, server_config);
                new_config.update_mcp_config(updated_mcp_config);

                on_config_change.emit(new_config);

                // Reset form
                new_server_name.set(String::new());
                new_server_url.set(String::new());
                new_server_auth_token.set(String::new());
                new_server_headers_json.set(String::from("{}"));
                show_add_server.set(false);
                editing_server.set(None);
            }
        })
    };

    // Handle editing a server
    let on_edit_server = {
        let config = config.clone();
        let new_server_name = new_server_name.clone();
        let new_server_url = new_server_url.clone();
        let new_server_auth_token = new_server_auth_token.clone();
        let new_server_headers_json = new_server_headers_json.clone();
        let show_add_server = show_add_server.clone();
        let editing_server = editing_server.clone();

        Callback::from(move |server_name: String| {
            let mcp_config = config.get_mcp_config();
            if let Some(server_config) = mcp_config.servers.get(&server_name) {
                // Populate form with existing values
                new_server_name.set(server_config.name.clone());
                new_server_url.set(server_config.url.clone().unwrap_or_default());

                // Extract auth token from headers if present
                let mut auth_token = String::new();
                let mut other_headers = HashMap::new();

                if let Some(headers) = &server_config.headers {
                    for (key, value) in headers {
                        if key == "Authorization" && value.starts_with("Bearer ") {
                            auth_token = value.strip_prefix("Bearer ").unwrap_or("").to_string();
                        } else {
                            other_headers.insert(key.clone(), value.clone());
                        }
                    }
                }

                new_server_auth_token.set(auth_token);
                new_server_headers_json.set(
                    serde_json::to_string_pretty(&other_headers)
                        .unwrap_or_else(|_| "{}".to_string()),
                );

                editing_server.set(Some(server_name));
                show_add_server.set(true);
            }
        })
    };

    // Handle removing a server
    let on_remove_server = {
        let config = config.clone();
        let on_config_change = props.on_config_change.clone();

        Callback::from(move |server_name: String| {
            let mut new_config = config.clone();
            let mut updated_mcp_config = new_config.get_mcp_config().clone();
            updated_mcp_config.servers.remove(&server_name);
            new_config.update_mcp_config(updated_mcp_config);

            // Also remove MCP tools from that server
            new_config
                .function_tools
                .retain(|tool| !tool.name.starts_with(&format!("mcp_{}_", server_name)));

            on_config_change.emit(new_config);
        })
    };

    // Handle toggling server enabled state
    let on_toggle_server = {
        let config = config.clone();
        let on_config_change = props.on_config_change.clone();

        Callback::from(move |(server_name, enabled): (String, bool)| {
            let mut new_config = config.clone();
            let mut updated_mcp_config = new_config.get_mcp_config().clone();

            if let Some(server_config) = updated_mcp_config.servers.get_mut(&server_name) {
                server_config.enabled = enabled;
            }

            new_config.update_mcp_config(updated_mcp_config);
            on_config_change.emit(new_config);
        })
    };

    // Handle connecting to MCP servers
    let on_connect_servers = {
        let config = config.clone();
        let on_mcp_client_change = props.on_mcp_client_change.clone();
        let connection_status = connection_status.clone();

        Callback::from(move |_| {
            let mcp_config = config.get_mcp_config().clone();

            // Only proceed if there are enabled servers
            let has_enabled_servers = mcp_config.servers.values().any(|server| server.enabled);
            if !has_enabled_servers {
                gloo_console::log!("No enabled MCP servers to connect to");
                return;
            }

            let mut client = McpClient::new(mcp_config);
            let status = connection_status.clone();
            let callback = on_mcp_client_change.clone();

            // Set status to "Connecting..." for enabled servers
            let mut connecting_status = (*status).clone();
            for (server_name, server_config) in client.get_config().servers.iter() {
                if server_config.enabled {
                    connecting_status.insert(server_name.clone(), "Connecting...".to_string());
                }
            }
            status.set(connecting_status);

            wasm_bindgen_futures::spawn_local(async move {
                match client.initialize().await {
                    Ok(_) => {
                        let mut new_status = (*status).clone();
                        for (server_name, server_config) in client.get_config().servers.iter() {
                            if server_config.enabled {
                                new_status.insert(server_name.clone(), "Connected".to_string());
                            }
                        }
                        status.set(new_status);
                        callback.emit(Some(client));
                        gloo_console::log!(
                            "MCP client initialized successfully via manual connection"
                        );
                    }
                    Err(e) => {
                        let mut new_status = (*status).clone();
                        for (server_name, server_config) in client.get_config().servers.iter() {
                            if server_config.enabled {
                                new_status.insert(server_name.clone(), format!("Error: {}", e));
                            }
                        }
                        status.set(new_status);
                        callback.emit(None);
                        gloo_console::log!(
                            "Failed to initialize MCP client via manual connection:",
                            &e
                        );
                    }
                }
            });
        })
    };

    html! {
        <div class="mcp-settings-panel p-4 bg-gray-50 rounded-lg">
            <div class="flex items-center justify-between mb-4">
                <h3 class="text-lg font-medium text-gray-900">{"MCP Server Configuration"}</h3>
                <button
                    class="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600"
                    onclick={
                        let show_add_server = show_add_server.clone();
                        Callback::from(move |_| show_add_server.set(!*show_add_server))
                    }
                >
                    {"Add Server"}
                </button>
            </div>

            // Add/Edit Server Form
            if *show_add_server {
                <div class="mb-4 p-4 bg-white dark:bg-gray-800 rounded border">
                    <h4 class="font-medium mb-3 text-gray-900 dark:text-gray-100">
                        {if editing_server.is_some() { "Edit MCP Server" } else { "Add New MCP Server" }}
                    </h4>
                    <div class="space-y-3">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                {"Server Name"}
                            </label>
                            <input
                                type="text"
                                placeholder="e.g., github, database-tools"
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                                value={(*new_server_name).clone()}
                                oninput={
                                    let new_server_name = new_server_name.clone();
                                    Callback::from(move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
                                        new_server_name.set(input.value());
                                    })
                                }
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                {"Server URL"}
                            </label>
                            <input
                                type="url"
                                placeholder="https://api.example.com/mcp/"
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                                value={(*new_server_url).clone()}
                                oninput={
                                    let new_server_url = new_server_url.clone();
                                    Callback::from(move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
                                        new_server_url.set(input.value());
                                    })
                                }
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                {"Auth Token (Optional)"}
                            </label>
                            <input
                                type="password"
                                placeholder="Bearer token will be added automatically"
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white"
                                value={(*new_server_auth_token).clone()}
                                oninput={
                                    let new_server_auth_token = new_server_auth_token.clone();
                                    Callback::from(move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
                                        new_server_auth_token.set(input.value());
                                    })
                                }
                            />
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                {"Custom Headers (JSON)"}
                            </label>
                            <textarea
                                placeholder=r#"{"X-API-Key": "your-key", "Content-Type": "application/json"}"#
                                rows="4"
                                class="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:text-white font-mono text-sm"
                                value={(*new_server_headers_json).clone()}
                                oninput={
                                    let new_server_headers_json = new_server_headers_json.clone();
                                    Callback::from(move |e: InputEvent| {
                                        let textarea: web_sys::HtmlTextAreaElement = e.target().unwrap().dyn_into().unwrap();
                                        new_server_headers_json.set(textarea.value());
                                    })
                                }
                            />
                            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1">
                                {"Enter custom headers as JSON. Auth token above will override any Authorization header."}
                            </p>
                        </div>

                        <div class="flex justify-between pt-2">
                            <div class="flex space-x-2">
                                <button
                                    class="px-4 py-2 bg-blue-500 text-white rounded-md hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500"
                                    onclick={on_add_server}
                                >
                                    {if editing_server.is_some() { "Update Server" } else { "Add Server" }}
                                </button>
                                <button
                                    class="px-4 py-2 bg-gray-500 text-white rounded-md hover:bg-gray-600 focus:outline-none focus:ring-2 focus:ring-gray-500"
                                    onclick={
                                        let show_add_server = show_add_server.clone();
                                        let editing_server = editing_server.clone();
                                        let new_server_name = new_server_name.clone();
                                        let new_server_url = new_server_url.clone();
                                        let new_server_auth_token = new_server_auth_token.clone();
                                        let new_server_headers_json = new_server_headers_json.clone();
                                        Callback::from(move |_| {
                                            show_add_server.set(false);
                                            editing_server.set(None);
                                            new_server_name.set(String::new());
                                            new_server_url.set(String::new());
                                            new_server_auth_token.set(String::new());
                                            new_server_headers_json.set(String::from("{}"));
                                        })
                                    }
                                >
                                    {"Cancel"}
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            }

            // Server List
            <div class="space-y-2">
                {
                    mcp_config.servers.iter().map(|(name, server_config)| {
                        let server_name = name.clone();
                        let status = connection_status.get(name).cloned().unwrap_or_else(|| "Not connected".to_string());

                        html! {
                            <div key={name.clone()} class="p-4 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
                                <div class="flex items-start justify-between">
                                    <div class="flex-1 min-w-0">
                                        <div class="flex items-center space-x-3 mb-2">
                                            <h4 class="font-semibold text-gray-900 dark:text-gray-100">{name}</h4>
                                            <span class={format!("px-2 py-1 text-xs font-medium rounded-full {}",
                                                if status.starts_with("Connected") { "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200" }
                                                else if status.starts_with("Error") { "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200" }
                                                else { "bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-300" }
                                            )}>
                                                {status}
                                            </span>
                                            <label class="flex items-center">
                                                <input
                                                    type="checkbox"
                                                    checked={server_config.enabled}
                                                    class="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                                                    onchange={
                                                        let on_toggle = on_toggle_server.clone();
                                                        let server_name = server_name.clone();
                                                        Callback::from(move |e: Event| {
                                                            let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
                                                            on_toggle.emit((server_name.clone(), input.checked()));
                                                        })
                                                    }
                                                />
                                                <span class="ml-2 text-sm text-gray-700 dark:text-gray-300">{"Enabled"}</span>
                                            </label>
                                        </div>

                                        <div class="space-y-1 text-sm text-gray-600 dark:text-gray-400">
                                            <div class="flex items-center">
                                                <span class="w-12 font-medium">{"URL:"}</span>
                                                <span class="font-mono bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded text-xs break-all">
                                                    {server_config.url.as_deref().unwrap_or("Not configured")}
                                                </span>
                                            </div>

                                            {if let Some(headers) = &server_config.headers {
                                                if !headers.is_empty() {
                                                    html! {
                                                        <div class="flex items-start">
                                                            <span class="w-12 font-medium">{"Headers:"}</span>
                                                            <div class="space-y-1">
                                                                {headers.iter().map(|(key, value)| {
                                                                    let display_value = if key == "Authorization" && value.len() > 20 {
                                                                        format!("{}...{}", &value[..15], &value[value.len()-5..])
                                                                    } else if value.len() > 30 {
                                                                        format!("{}...", &value[..27])
                                                                    } else {
                                                                        value.clone()
                                                                    };

                                                                    html! {
                                                                        <div class="font-mono bg-gray-100 dark:bg-gray-700 px-2 py-1 rounded text-xs">
                                                                            <span class="text-blue-600 dark:text-blue-400">{key}{": "}</span>
                                                                            <span>{display_value}</span>
                                                                        </div>
                                                                    }
                                                                }).collect::<Html>()}
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
                                    </div>

                                    <div class="flex items-center space-x-2 ml-4">
                                        <button
                                            class="px-3 py-1 bg-blue-500 text-white rounded-md hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm"
                                            onclick={
                                                let on_edit = on_edit_server.clone();
                                                let server_name = server_name.clone();
                                                Callback::from(move |_| on_edit.emit(server_name.clone()))
                                            }
                                        >
                                            {"Edit"}
                                        </button>
                                        <button
                                            class="px-3 py-1 bg-red-500 text-white rounded-md hover:bg-red-600 focus:outline-none focus:ring-2 focus:ring-red-500 text-sm"
                                            onclick={
                                                let on_remove = on_remove_server.clone();
                                                let server_name = server_name.clone();
                                                Callback::from(move |_| on_remove.emit(server_name.clone()))
                                            }
                                        >
                                            {"Remove"}
                                        </button>
                                    </div>
                                </div>
                            </div>
                        }
                    }).collect::<Html>()
                }
            </div>

            // Connect Button
            <div class="mt-4">
                <button
                    class="w-full px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
                    onclick={on_connect_servers}
                >
                    {"Connect to MCP Servers"}
                </button>
            </div>

            // Instructions
            <div class="mt-4 p-3 bg-blue-50 rounded">
                <h4 class="font-medium text-blue-900 mb-2">{"How to use MCP servers:"}</h4>
                <ul class="text-sm text-blue-800 space-y-1">
                    <li>{"• Add your MCP server URL and authentication token"}</li>
                    <li>{"• Enable the servers you want to use"}</li>
                    <li>{"• Click 'Connect to MCP Servers' to discover available tools"}</li>
                    <li>{"• MCP tools will appear in your function tools list with 'mcp_' prefix"}</li>
                </ul>
            </div>
        </div>
    }
}
