// Example of MCP Client Integration
// This example shows how to integrate MCP servers as built-in function tools

use crate::llm_playground::{
    mcp_client::{McpClient, McpConfig, McpServerConfig},
    builtin_tools::execute_builtin_tool,
    types::{ApiConfig, FunctionTool},
};
use std::collections::HashMap;
use serde_json::json;

/// Example function showing how to initialize and use MCP client
pub async fn mcp_integration_example() -> Result<(), String> {
    // 1. Create MCP configuration
    let mut mcp_config = McpConfig::default();
    
    // Add a GitHub Copilot MCP server (example)
    let mut github_headers = HashMap::new();
    github_headers.insert("Authorization".to_string(), "Bearer YOUR_GITHUB_TOKEN".to_string());
    
    mcp_config.servers.insert("github".to_string(), McpServerConfig {
        name: "GitHub Copilot MCP".to_string(),
        server_type: "http".to_string(),
        url: Some("https://api.githubcopilot.com/mcp/".to_string()),
        headers: Some(github_headers),
        enabled: true,
    });
    
    // Add a custom MCP server
    mcp_config.servers.insert("custom".to_string(), McpServerConfig {
        name: "Custom MCP Server".to_string(),
        server_type: "http".to_string(),
        url: Some("http://localhost:8080/mcp/".to_string()),
        headers: None,
        enabled: true,
    });

    // 2. Initialize MCP client
    let mut mcp_client = McpClient::new(mcp_config);
    
    // Connect to servers and discover tools
    match mcp_client.initialize().await {
        Ok(_) => {
            println!("Successfully connected to MCP servers!");
            
            // 3. Get discovered MCP tools as FunctionTool objects
            let mcp_function_tools = mcp_client.get_function_tools();
            println!("Discovered {} MCP tools", mcp_function_tools.len());
            
            for tool in &mcp_function_tools {
                println!("- {}: {}", tool.name, tool.description);
            }
            
            // 4. Update ApiConfig with MCP tools
            let mut api_config = ApiConfig::default();
            api_config.add_mcp_tools(mcp_function_tools);
            
            // 5. Example: Call an MCP tool
            let tool_name = "mcp_github_search_repositories";
            let arguments = json!({
                "query": "rust mcp client",
                "language": "rust",
                "sort": "stars"
            });
            
            match execute_builtin_tool(tool_name, &arguments, Some(&mcp_client)).await {
                Ok(result) => {
                    println!("MCP tool result: {}", result);
                }
                Err(e) => {
                    println!("Error calling MCP tool: {}", e);
                }
            }
            
            Ok(())
        }
        Err(e) => {
            println!("Failed to initialize MCP client: {}", e);
            Err(e)
        }
    }
}

/// Example of manually configuring MCP servers
pub fn configure_mcp_servers_example() -> McpConfig {
    let mut config = McpConfig::default();
    
    // Example 1: GitHub Copilot MCP server
    let mut github_headers = HashMap::new();
    github_headers.insert("Authorization".to_string(), "Bearer ghp_xxxxxxxxxxxxxxxxxxxx".to_string());
    
    config.servers.insert("github".to_string(), McpServerConfig {
        name: "GitHub Copilot".to_string(),
        server_type: "http".to_string(),
        url: Some("https://api.githubcopilot.com/mcp/".to_string()),
        headers: Some(github_headers),
        enabled: true,
    });
    
    // Example 2: Local development MCP server
    config.servers.insert("local_dev".to_string(), McpServerConfig {
        name: "Local Development Server".to_string(),
        server_type: "http".to_string(),
        url: Some("http://localhost:3000/mcp/".to_string()),
        headers: None,
        enabled: false, // Disabled by default
    });
    
    // Example 3: Custom API with authentication
    let mut custom_headers = HashMap::new();
    custom_headers.insert("Authorization".to_string(), "Bearer api_key_here".to_string());
    custom_headers.insert("X-API-Version".to_string(), "2024-01".to_string());
    
    config.servers.insert("custom_api".to_string(), McpServerConfig {
        name: "Custom API Server".to_string(),
        server_type: "http".to_string(),
        url: Some("https://api.example.com/mcp/".to_string()),
        headers: Some(custom_headers),
        enabled: true,
    });
    
    config
}

/// Example of handling MCP tool calls in the UI
pub async fn handle_mcp_function_call(
    tool_name: &str,
    arguments: &serde_json::Value,
    mcp_client: Option<&McpClient>
) -> Result<serde_json::Value, String> {
    // Check if this is an MCP tool
    if tool_name.starts_with("mcp_") {
        if let Some(client) = mcp_client {
            // Call the MCP tool
            return client.call_tool(tool_name, arguments).await;
        } else {
            return Err("MCP client not available".to_string());
        }
    }
    
    // Handle other built-in tools
    execute_builtin_tool(tool_name, arguments, mcp_client).await
}

/// Example of updating function tools list with MCP tools
pub fn update_function_tools_with_mcp(
    api_config: &mut ApiConfig,
    mcp_client: &McpClient
) {
    // Get MCP tools
    let mcp_tools = mcp_client.get_function_tools();
    
    // Add them to the configuration
    api_config.add_mcp_tools(mcp_tools);
    
    // The function tools are now available for the LLM to use
    println!("Updated function tools list with {} MCP tools", 
        api_config.get_enabled_function_tools().len());
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mcp_config_creation() {
        let config = configure_mcp_servers_example();
        assert_eq!(config.servers.len(), 3);
        assert!(config.servers.contains_key("github"));
        assert!(config.servers.contains_key("local_dev"));
        assert!(config.servers.contains_key("custom_api"));
    }
    
    #[test]
    fn test_mcp_tool_name_detection() {
        assert!(is_mcp_tool_name("mcp_github_search"));
        assert!(is_mcp_tool_name("mcp_custom_api_fetch"));
        assert!(!is_mcp_tool_name("fetch"));
        assert!(!is_mcp_tool_name("regular_tool"));
    }
    
    fn is_mcp_tool_name(name: &str) -> bool {
        name.starts_with("mcp_")
    }
}