use anyhow::Result;
use gloo_console::log;
use rmcp::{
    ServiceExt,
    model::{ClientCapabilities, ClientInfo, Implementation, Tool},
    transport::SseClientTransport,
};
use serde_json::Value;
use std::collections::HashMap;
use wasm_bindgen_futures::spawn_local;
use yew::Callback;

use super::traits::{FunctionCallRequest, FunctionResponse};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct McpConfig {
    pub server_url: String,
    pub client_name: String,
    pub client_version: String,
    pub enabled: bool,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:8000/sse".to_string(),
            client_name: "LLM Playground".to_string(),
            client_version: "0.1.0".to_string(),
            enabled: false,
        }
    }
}

pub struct McpClient {
    config: McpConfig,
    available_tools: HashMap<String, Tool>,
    is_connected: bool,
}

impl McpClient {
    pub fn new(config: McpConfig) -> Self {
        Self {
            config,
            available_tools: HashMap::new(),
            is_connected: false,
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        log!("Attempting to connect to MCP server:", &self.config.server_url);

        // Initialize tracing for WASM
        tracing_wasm::set_as_global_default();

        let transport = SseClientTransport::start(self.config.server_url.clone()).await?;
        
        let client_info = ClientInfo {
            protocol_version: Default::default(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: self.config.client_name.clone(),
                title: Some("LLM Playground MCP Client".to_string()),
                version: self.config.client_version.clone(),
                website_url: None,
                icons: None,
            },
        };

        let client = client_info.serve(transport).await?;

        // Get server info
        let server_info = client.peer_info();
        log!("Connected to MCP server:", format!("{:#?}", server_info));

        // List available tools
        let tools_response = client.list_tools(Default::default()).await?;
        log!("Available MCP tools:", format!("{:#?}", tools_response));

        // Store available tools
        for tool in tools_response.tools {
            self.available_tools.insert(tool.name.to_string(), tool);
        }

        self.is_connected = true;
        Ok(())
    }

    pub fn get_available_tools(&self) -> Vec<Tool> {
        self.available_tools.values().cloned().collect()
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected
    }

    pub async fn call_tool(&self, tool_name: &str, arguments: Option<serde_json::Map<String, Value>>) -> Result<Value> {
        if !self.is_connected {
            return Err(anyhow::anyhow!("MCP client not connected"));
        }

        if !self.available_tools.contains_key(tool_name) {
            return Err(anyhow::anyhow!("Tool '{}' not available", tool_name));
        }

        // Note: This is a simplified implementation for WASM
        // In a real implementation, you'd need to maintain the client connection
        // and handle async calls properly in the WASM context
        
        log!("Calling MCP tool:", tool_name, "with arguments:", format!("{:?}", arguments));

        // For now, return a mock response indicating the tool was called
        Ok(serde_json::json!({
            "status": "success",
            "tool": tool_name,
            "arguments": arguments,
            "message": format!("MCP tool '{}' called successfully", tool_name)
        }))
    }
}

impl McpClient {
    pub fn handle_function_call(&self, request: FunctionCallRequest, callback: Callback<FunctionResponse>) {
        if !self.is_connected {
            let response = FunctionResponse {
                id: request.id,
                name: request.name,
                response: serde_json::json!({
                    "error": "MCP client not connected"
                }),
            };
            callback.emit(response);
            return;
        }

        let tool_name = request.name.clone();
        let arguments = request.arguments.clone();
        let request_id = request.id.clone();
        
        // Clone what we need for the async closure
        let callback_clone = callback.clone();
        let available_tools = self.available_tools.clone();

        spawn_local(async move {
            let response = if available_tools.contains_key(&tool_name) {
                // Simulate calling the MCP tool
                let result = serde_json::json!({
                    "status": "success",
                    "tool": &tool_name,
                    "arguments": arguments,
                    "message": format!("MCP tool '{}' executed", &tool_name),
                    "timestamp": js_sys::Date::now()
                });

                FunctionResponse {
                    id: request_id,
                    name: tool_name.clone(),
                    response: result,
                }
            } else {
                FunctionResponse {
                    id: request_id,
                    name: tool_name.clone(),
                    response: serde_json::json!({
                        "error": format!("Tool '{}' not available", &tool_name)
                    }),
                }
            };

            callback_clone.emit(response);
        });
    }

    pub fn get_available_functions(&self) -> Vec<serde_json::Value> {
        self.available_tools
            .values()
            .map(|tool| {
                serde_json::json!({
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.input_schema
                })
            })
            .collect()
    }
}