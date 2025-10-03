# MCP Client Integration Guide

This guide explains how to integrate MCP (Model Context Protocol) servers as built-in function tools in the LLM Playground.

## Overview

The MCP client integration allows you to:
- Connect to external MCP servers via HTTP
- Automatically discover available tools from MCP servers
- Use MCP tools as function tools in the LLM playground
- Manage multiple MCP server connections
- Handle authentication and configuration

## Quick Start

### 1. Configure MCP Servers

Add MCP server configurations to your API config:

```rust
use crate::llm_playground::mcp_client::{McpConfig, McpServerConfig};
use std::collections::HashMap;

let mut mcp_config = McpConfig::default();

// Add GitHub Copilot MCP server
let mut headers = HashMap::new();
headers.insert("Authorization".to_string(), "Bearer YOUR_TOKEN".to_string());

mcp_config.servers.insert("github".to_string(), McpServerConfig {
    name: "GitHub Copilot MCP".to_string(),
    server_type: "http".to_string(),
    url: Some("https://api.githubcopilot.com/mcp/".to_string()),
    headers: Some(headers),
    enabled: true,
});
```

### 2. Initialize MCP Client

```rust
use crate::llm_playground::mcp_client::McpClient;

let mut mcp_client = McpClient::new(mcp_config);
mcp_client.initialize().await?;
```

### 3. Add MCP Tools to Function Tools

```rust
let mcp_tools = mcp_client.get_function_tools();
api_config.add_mcp_tools(mcp_tools);
```

### 4. Use MCP Tools

MCP tools are automatically available as function tools with the `mcp_` prefix:

```rust
use crate::llm_playground::builtin_tools::execute_builtin_tool;

let result = execute_builtin_tool(
    "mcp_github_search_repositories",
    &serde_json::json!({
        "query": "rust mcp",
        "language": "rust"
    }),
    Some(&mcp_client)
).await?;
```

## Configuration

### Server Configuration

Each MCP server requires:

- **name**: Display name for the server
- **server_type**: Currently only "http" is supported in WASM
- **url**: The MCP server endpoint URL
- **headers**: Optional HTTP headers (e.g., for authentication)
- **enabled**: Whether the server should be used

### Example Server Configurations

#### GitHub Copilot MCP
```json
{
  "name": "GitHub Copilot MCP",
  "server_type": "http",
  "url": "https://api.githubcopilot.com/mcp/",
  "headers": {
    "Authorization": "Bearer YOUR_GITHUB_TOKEN"
  },
  "enabled": true
}
```

#### Custom API Server
```json
{
  "name": "Custom API",
  "server_type": "http", 
  "url": "https://api.example.com/mcp/",
  "headers": {
    "Authorization": "Bearer api_key_here",
    "X-API-Version": "2024-01"
  },
  "enabled": true
}
```

#### Local Development Server
```json
{
  "name": "Local Dev Server",
  "server_type": "http",
  "url": "http://localhost:3000/mcp/",
  "headers": null,
  "enabled": false
}
```

## UI Integration

### MCP Settings Panel

The `McpSettingsPanel` component provides a user interface for:
- Adding new MCP servers
- Configuring server URLs and authentication
- Enabling/disabling servers
- Viewing connection status
- Managing server configurations

```rust
use crate::llm_playground::components::McpSettingsPanel;

html! {
    <McpSettingsPanel
        config={api_config.clone()}
        on_config_change={on_config_change}
        mcp_client={mcp_client.clone()}
        on_mcp_client_change={on_mcp_client_change}
    />
}
```

## Tool Discovery and Usage

### Automatic Tool Discovery

When the MCP client initializes:
1. Connects to each enabled server
2. Calls the `tools/list` method to discover available tools
3. Converts MCP tool definitions to `FunctionTool` format
4. Adds tools to the function tools list with `mcp_` prefix

### Tool Naming Convention

MCP tools are prefixed with `mcp_{server_name}_{tool_name}`:
- `mcp_github_search_repositories`
- `mcp_custom_api_fetch_data`
- `mcp_local_dev_process_file`

### Tool Execution

MCP tools are executed by:
1. Detecting the `mcp_` prefix
2. Extracting server name and tool name
3. Sending a `tools/call` request to the appropriate server
4. Returning the response to the LLM

## Error Handling

The MCP client handles various error scenarios:

### Connection Errors
- Server unreachable
- Invalid URL
- Network timeouts

### Authentication Errors
- Invalid tokens
- Missing credentials
- Expired authentication

### Protocol Errors
- Invalid MCP responses
- Unsupported protocol versions
- Malformed tool definitions

## Security Considerations

### Authentication Tokens
- Store tokens securely
- Use environment variables for sensitive data
- Rotate tokens regularly

### Server Validation
- Verify server certificates
- Use HTTPS for production servers
- Validate server responses

### CORS Configuration
When running MCP servers, ensure proper CORS headers:
```
Access-Control-Allow-Origin: *
Access-Control-Allow-Methods: POST, GET, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization
```

## Debugging

### Enable Logging
The MCP client logs connection attempts and tool calls:

```rust
// Check browser console for MCP-related logs
console.log("MCP client initialized");
console.log("Available tools:", tool_names);
```

### Common Issues

1. **"MCP tool not available"**
   - Check if server is enabled
   - Verify connection status
   - Ensure tool discovery completed

2. **"Network request failed"**
   - Check server URL
   - Verify CORS configuration
   - Test server availability

3. **"Authorization failed"**
   - Verify authentication token
   - Check token format and headers
   - Ensure token has required permissions

## Examples

See `src/llm_playground/examples/mcp_integration_example.rs` for complete working examples.

## Supported MCP Features

### Currently Supported
- ✅ HTTP transport
- ✅ Tool discovery (`tools/list`)
- ✅ Tool execution (`tools/call`)
- ✅ Authentication headers
- ✅ Error handling

### Planned Features
- 🔄 WebSocket transport
- 🔄 Resource discovery
- 🔄 Prompt templates
- 🔄 Session management
- 🔄 Tool streaming responses

## Contributing

To add new MCP features:
1. Update `McpClient` with new methods
2. Add UI components if needed
3. Update function tool integration
4. Add tests and documentation

## References

- [MCP Specification](https://spec.modelcontextprotocol.io/)
- [Rust MCP SDK](./rust-sdk/README.md)
- [MCP Transport Documentation](./rust-sdk/crates/rmcp/README.md)



╭─ Response ───────────────────────────────────────────────────────────────────────────────────────────────────────────╮
│                                                                                                                      │
│                                      ✅ Enhanced MCP Configuration UI Complete!                                      │
│                                                                                                                      │
│ I have successfully implemented a comprehensive MCP server configuration interface with both user-friendly forms and │
│ raw JSON support. Here's what was accomplished:                                                                      │
│                                                                                                                      │
│                                             🚀 New Features Implemented:                                             │
│                                                                                                                      │
│                                            1. Enhanced Configuration Form                                            │
│                                                                                                                      │
│  • Server Name: Clear naming for easy identification                                                                 │
│  • Server URL: Full URL input with validation                                                                        │
│  • Auth Token: Dedicated field that automatically adds Bearer  prefix                                                │
│  • Custom Headers (JSON): Raw JSON textarea for advanced header configuration                                        │
│  • Edit Functionality: Click "Edit" to modify existing servers                                                       │
│  • Enable/Disable Toggle: Inline checkbox for quick server management                                                │
│                                                                                                                      │
│                                              2. Improved Server Display                                              │
│                                                                                                                      │
│  • Detailed Server Cards: Each server shows comprehensive information                                                │
│  • Connection Status: Real-time status indicators with color coding                                                  │
│  • Header Display: Shows configured headers with truncation for security                                             │
│  • URL Display: Monospace font for easy reading                                                                      │
│  • Action Buttons: Edit and Remove buttons for each server                                                           │
│                                                                                                                      │
│                                             3. Enhanced User Experience                                              │
│                                                                                                                      │
│  • Form Validation: Ensures required fields are filled                                                               │
│  • JSON Parsing: Validates and parses custom headers automatically                                                   │
│  • Edit Mode: Pre-populates form when editing existing servers                                                       │
│  • Auth Token Separation: Separates auth tokens from other headers for clarity                                       │
│  • Dark Mode Support: Full dark mode compatibility                                                                   │
│                                                                                                                      │
│                                                 🎨 UI Improvements:                                                  │
│                                                                                                                      │
│                                                Add/Edit Server Form:                                                 │
│                                                                                                                      │
│                                                                                                                      │
│  ┌─────────────────────────────────────────┐                                                                         │
│  │ Add New MCP Server                      │                                                                         │
│  ├─────────────────────────────────────────┤                                                                         │
│  │ Server Name: [github                  ] │                                                                         │
│  │ Server URL:  [https://api.github.com/] │                                                                          │
│  │ Auth Token:  [••••••••••••••••••••••••] │                                                                         │
│  │ Custom Headers (JSON):                  │                                                                         │
│  │ ┌─────────────────────────────────────┐ │                                                                         │
│  │ │ {                                   │ │                                                                         │
│  │ │   "X-API-Key": "your-key",          │ │                                                                         │
│  │ │   "Content-Type": "application/json"│ │                                                                         │
│  │ │ }                                   │ │                                                                         │
│  │ └─────────────────────────────────────┘ │                                                                         │
│  │ [Update Server] [Cancel]                │                                                                         │
│  └─────────────────────────────────────────┘                                                                         │
│                                                                                                                      │
│                                                                                                                      │
│                                                 Server List Display:                                                 │
│                                                                                                                      │
│                                                                                                                      │
│  ┌─────────────────────────────────────────┐                                                                         │
│  │ github    🟢 Connected    ☑ Enabled     │                                                                         │
│  │ URL: https://api.github.com/mcp/        │                                                                         │
│  │ Headers:                                │                                                                         │
│  │   Authorization: Bearer abc12...xyz89   │                                                                         │
│  │   X-API-Key: key123...789               │                                                                         │
│  │                           [Edit][Remove]│                                                                         │
│  └─────────────────────────────────────────┘                                                                         │
│                                                                                                                      │
│                                                                                                                      │
│                                                   📝 Key Features:                                                   │
│                                                                                                                      │
│  1 User-Friendly Forms: Clean, labeled inputs with placeholders                                                      │
│  2 JSON Editor: Raw JSON textarea for complex header configurations                                                  │
│  3 Edit Mode: Full editing support with form pre-population                                                          │
│  4 Security: Auth tokens are truncated in display but fully editable                                                 │
│  5 Validation: JSON parsing with error handling                                                                      │
│  6 Real-time Updates: Changes reflected immediately                                                                  │
│  7 Responsive Design: Works on different screen sizes                                                                │
│                                                                                                                      │
│                                                    🔧 How to Use:                                                    │
│                                                                                                                      │
│  1 Add Server: Click "Add Server" → Fill form → Click "Add Server"                                                   │
│  2 Edit Server: Click "Edit" on any server → Modify → Click "Update Server"                                          │
│  3 Configure Headers: Use JSON textarea for custom headers like API keys                                             │
│  4 Enable/Disable: Use checkbox to toggle server without deleting                                                    │
│  5 Remove Server: Click "Remove" to delete server completely                                                         │
│                                                                                                                      │
│ The enhanced MCP configuration UI now provides both beginner-friendly forms and advanced JSON configuration options, │
│ making it easy for users to configure MCP servers with any authentication scheme or custom headers!                  │
│                                                                                                                      │
│ What would you like to work on next? I can help with:                                                                │
│                                                                                                                      │
│  • Testing the new UI with real MCP servers                                                                          │
│  • Adding more advanced features like server templates                                                               │
│  • Implementing import/export for server configurations                                                              │
│  • Adding server testing/validation features                                                                         │
│  • Creating documentation for different MCP server types                                                             │
╰──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────╯