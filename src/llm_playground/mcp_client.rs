// MCP Client for LLM Playground
// Integrates MCP servers as built-in function tools

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

use crate::llm_playground::types::FunctionTool;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub server_type: String, // "http", "stdio", etc.
    pub url: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpConfig {
    pub servers: HashMap<String, McpServerConfig>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Value,
    pub server_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpResponse {
    pub jsonrpc: String,
    pub id: String,
    pub result: Option<Value>,
    pub error: Option<McpError>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Clone, PartialEq)]
pub struct McpClient {
    config: McpConfig,
    available_tools: HashMap<String, McpTool>,
    session_ids: HashMap<String, String>, // server_name -> session_id
}

impl Default for McpConfig {
    fn default() -> Self {
        let mut servers = HashMap::new();
        
        // Add example GitHub Copilot MCP server
        servers.insert("github".to_string(), McpServerConfig {
            name: "GitHub Copilot MCP".to_string(),
            server_type: "http".to_string(),
            url: Some("https://api.githubcopilot.com/mcp/".to_string()),
            headers: Some({
                let mut headers = HashMap::new();
                headers.insert("Authorization".to_string(), "Bearer YOUR_TOKEN_HERE".to_string());
                headers
            }),
            enabled: false, // Disabled by default until user configures token
        });

        Self { servers }
    }
}

impl McpClient {
    pub fn new(config: McpConfig) -> Self {
        Self {
            config,
            available_tools: HashMap::new(),
            session_ids: HashMap::new(),
        }
    }

    /// Initialize connections to all enabled MCP servers
    pub async fn initialize(&mut self) -> Result<(), String> {
        log("Initializing MCP client connections...");
        
        // Clone the servers to avoid borrowing issues
        let servers = self.config.servers.clone();
        for (server_name, server_config) in servers {
            if server_config.enabled {
                match self.connect_to_server(&server_name, &server_config).await {
                    Ok(_) => {
                        log(&format!("Successfully connected to MCP server: {}", server_name));
                    }
                    Err(e) => {
                        log(&format!("Failed to connect to MCP server {}: {}", server_name, e));
                    }
                }
            }
        }

        Ok(())
    }

    /// Connect to a specific MCP server and discover its tools
    async fn connect_to_server(&mut self, server_name: &str, config: &McpServerConfig) -> Result<(), String> {
        if config.server_type != "http" {
            return Err("Only HTTP MCP servers are currently supported in WASM".to_string());
        }

        let url = config.url.as_ref().ok_or("HTTP server must have URL")?;

        // Initialize connection
        match self.initialize_connection(server_name, url, &config.headers).await {
            Ok(session_id) => {
                self.session_ids.insert(server_name.to_string(), session_id);
            }
            Err(e) => {
                log(&format!("Failed to initialize connection to {}: {}", server_name, e));
                return Err(e);
            }
        }

        // List available tools
        match self.list_tools(server_name).await {
            Ok(tools) => {
                for tool in tools {
                    self.available_tools.insert(
                        format!("mcp_{}_{}", server_name, tool.name),
                        tool
                    );
                }
            }
            Err(e) => {
                log(&format!("Failed to list tools from {}: {}", server_name, e));
                return Err(e);
            }
        }

        Ok(())
    }

    /// Initialize connection with MCP server
    async fn initialize_connection(
        &self,
        _server_name: &str,
        url: &str,
        headers: &Option<HashMap<String, String>>
    ) -> Result<String, String> {
        let init_request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: uuid::Uuid::new_v4().to_string(),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "clientInfo": {
                    "name": "LLM Playground",
                    "version": "1.0.0"
                }
            })),
        };

        let response = self.send_request(url, &init_request, headers).await?;
        
        if let Some(error) = response.error {
            return Err(format!("MCP initialization error: {}", error.message));
        }

        // Extract session ID from response headers or generate one
        let session_id = uuid::Uuid::new_v4().to_string();
        Ok(session_id)
    }

    /// List tools available on an MCP server
    async fn list_tools(&self, server_name: &str) -> Result<Vec<McpTool>, String> {
        let server_config = self.config.servers.get(server_name)
            .ok_or("Server not found in configuration")?;
        
        let url = server_config.url.as_ref().ok_or("Server URL not configured")?;

        let list_request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: uuid::Uuid::new_v4().to_string(),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = self.send_request(url, &list_request, &server_config.headers).await?;
        
        if let Some(error) = response.error {
            return Err(format!("MCP tools/list error: {}", error.message));
        }

        let result = response.result.ok_or("No result in tools/list response")?;
        let tools_array = result.get("tools").ok_or("No 'tools' field in response")?;
        
        let mut tools = Vec::new();
        if let Some(tools_list) = tools_array.as_array() {
            for tool_value in tools_list {
                if let Ok(tool_data) = serde_json::from_value::<Value>(tool_value.clone()) {
                    let name = tool_data.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    
                    let description = tool_data.get("description")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    
                    let input_schema = tool_data.get("inputSchema")
                        .cloned()
                        .unwrap_or(serde_json::json!({}));

                    tools.push(McpTool {
                        name,
                        description,
                        input_schema,
                        server_name: server_name.to_string(),
                    });
                }
            }
        }

        Ok(tools)
    }

    /// Call a tool on an MCP server
    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: &Value
    ) -> Result<Value, String> {
        // Extract server name and tool name from prefixed tool name
        let (server_name, actual_tool_name) = if tool_name.starts_with("mcp_") {
            let parts: Vec<&str> = tool_name.splitn(3, '_').collect();
            if parts.len() >= 3 {
                (parts[1], parts[2])
            } else {
                return Err("Invalid MCP tool name format".to_string());
            }
        } else {
            return Err("Tool name must start with 'mcp_' prefix".to_string());
        };

        let server_config = self.config.servers.get(server_name)
            .ok_or("Server not found in configuration")?;
        
        let url = server_config.url.as_ref().ok_or("Server URL not configured")?;

        let call_request = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: uuid::Uuid::new_v4().to_string(),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": actual_tool_name,
                "arguments": arguments
            })),
        };

        let response = self.send_request(url, &call_request, &server_config.headers).await?;
        
        if let Some(error) = response.error {
            return Err(format!("MCP tools/call error: {}", error.message));
        }

        let result = response.result.ok_or("No result in tools/call response")?;
        Ok(result)
    }

    /// Send an MCP request to a server
    async fn send_request(
        &self,
        url: &str,
        request: &McpRequest,
        headers: &Option<HashMap<String, String>>
    ) -> Result<McpResponse, String> {
        // Create request options
        let opts = RequestInit::new();
        opts.set_method("POST");
        opts.set_mode(RequestMode::Cors);

        // Set body
        let body = serde_json::to_string(request)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;
        opts.set_body(&JsValue::from_str(&body));

        // Create the request
        let web_request = Request::new_with_str_and_init(url, &opts)
            .map_err(|e| format!("Failed to create request: {:?}", e))?;

        // Set headers
        let request_headers = web_request.headers();
        request_headers.set("Content-Type", "application/json")
            .map_err(|e| format!("Failed to set content-type header: {:?}", e))?;

        if let Some(custom_headers) = headers {
            for (key, value) in custom_headers {
                request_headers.set(key, value)
                    .map_err(|e| format!("Failed to set header {}: {:?}", key, e))?;
            }
        }

        // Add session ID if available
        // Note: This would typically be handled by the MCP transport layer

        // Make the request
        let window = web_sys::window().ok_or("No global window object")?;
        let resp_value = JsFuture::from(window.fetch_with_request(&web_request))
            .await
            .map_err(|e| format!("Network request failed: {:?}", e))?;

        let resp: Response = resp_value
            .dyn_into()
            .map_err(|_| "Response is not a Response object")?;

        // Check response status
        if !resp.ok() {
            return Err(format!("HTTP error: {} {}", resp.status(), resp.status_text()));
        }

        // Get response body
        let response_text = JsFuture::from(resp.text().map_err(|e| format!("Failed to get response text: {:?}", e))?)
            .await
            .map_err(|e| format!("Failed to read response body: {:?}", e))?
            .as_string()
            .unwrap_or_default();

        // Parse MCP response
        let mcp_response: McpResponse = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse MCP response: {}", e))?;

        Ok(mcp_response)
    }

    /// Convert MCP tools to FunctionTool format for the LLM playground
    pub fn get_function_tools(&self) -> Vec<FunctionTool> {
        let mut function_tools = Vec::new();

        for (prefixed_name, mcp_tool) in &self.available_tools {
            let function_tool = FunctionTool {
                name: prefixed_name.clone(),
                description: mcp_tool.description.clone()
                    .unwrap_or_else(|| format!("MCP tool: {}", mcp_tool.name)),
                parameters: mcp_tool.input_schema.clone(),
                mock_response: r#"{"status": "success", "source": "mcp_server"}"#.to_string(),
                enabled: true,
                category: format!("MCP ({})", mcp_tool.server_name),
                is_builtin: true,
            };
            function_tools.push(function_tool);
        }

        function_tools
    }

    /// Check if a tool name is an MCP tool
    pub fn is_mcp_tool(&self, tool_name: &str) -> bool {
        tool_name.starts_with("mcp_") && self.available_tools.contains_key(tool_name)
    }

    /// Get MCP configuration
    pub fn get_config(&self) -> &McpConfig {
        &self.config
    }

    /// Update MCP configuration
    pub fn update_config(&mut self, config: McpConfig) {
        self.config = config;
    }

    /// Add or update a server configuration
    pub fn add_server(&mut self, name: String, config: McpServerConfig) {
        self.config.servers.insert(name, config);
    }

    /// Remove a server configuration
    pub fn remove_server(&mut self, name: &str) {
        self.config.servers.remove(name);
        // Also remove any tools from that server
        self.available_tools.retain(|tool_name, _| {
            !tool_name.starts_with(&format!("mcp_{}_", name))
        });
        self.session_ids.remove(name);
    }

    /// Get available MCP tools
    pub fn get_available_tools(&self) -> &HashMap<String, McpTool> {
        &self.available_tools
    }
}

// UUID generation for WASM
mod uuid {
    pub struct Uuid;
    
    impl Uuid {
        pub fn new_v4() -> Self {
            Self
        }
        
        pub fn to_string(&self) -> String {
            // Simple UUID v4 generation for WASM
            let mut id = String::new();
            for i in 0..32 {
                if i == 8 || i == 12 || i == 16 || i == 20 {
                    id.push('-');
                }
                id.push_str(&format!("{:x}", (js_sys::Math::random() * 16.0) as u8 & 0xf));
            }
            id
        }
    }
}