// Built-in function tools with real implementations
use serde_json::Value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};
use std::collections::HashMap;

use crate::llm_playground::mcp_client::McpClient;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Execute a built-in function tool
pub async fn execute_builtin_tool(
    tool_name: &str,
    arguments: &Value,
    mcp_client: Option<&McpClient>,
) -> Result<Value, String> {
    log(&format!("execute_builtin_tool called with: {}", tool_name));
    
    // Check if this is an MCP tool
    if let Some(client) = mcp_client {
        if client.is_mcp_tool(tool_name) {
            return client.call_tool(tool_name, arguments).await;
        }
    }
    
    // Handle built-in tools
    match tool_name {
        "fetch" => execute_fetch(arguments).await,
        _ => Err(format!("Unknown built-in tool: {}", tool_name)),
    }
}

/// Execute the fetch tool with real HTTP requests
async fn execute_fetch(arguments: &Value) -> Result<Value, String> {
    // Extract parameters
    let url = arguments
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or("Missing required parameter: url")?;

    let method = arguments
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("GET");

    let headers = arguments
        .get("headers")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let payload = arguments
        .get("payload")
        .and_then(|v| v.as_str());

    // Log the request for debugging
    log(&format!("Making {} request to: {}", method, url));

    // Create request options
    let opts = RequestInit::new();
    opts.set_method(method);
    opts.set_mode(RequestMode::Cors);

    // Add payload if provided
    if let Some(body) = payload {
        if !body.is_empty() {
            opts.set_body(&JsValue::from_str(body));
        }
    }

    // Create the request
    let request = Request::new_with_str_and_init(url, &opts)
        .map_err(|e| format!("Failed to create request: {:?}", e))?;

    // Add headers
    for (key, value) in headers {
        if let Some(header_value) = value.as_str() {
            request
                .headers()
                .set(&key, header_value)
                .map_err(|e| format!("Failed to set header {}: {:?}", key, e))?;
        }
    }

    // Make the request
    let window = web_sys::window().ok_or("No global window object")?;
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|e| format!("Network request failed: {:?}", e))?;

    let resp: Response = resp_value
        .dyn_into()
        .map_err(|_| "Response is not a Response object")?;

    // Extract response status and headers
    let status = resp.status();
    let status_text = resp.status_text();
    
    // Get response headers
    let mut response_headers = HashMap::new();
    let headers_iterator = resp.headers().entries();
    let iterator = js_sys::try_iter(&headers_iterator)
        .map_err(|_| "Failed to iterate headers")?
        .ok_or("Headers iterator is null")?;

    for entry in iterator {
        if let Ok(entry_array) = entry {
            if let Ok(array) = entry_array.dyn_into::<js_sys::Array>() {
                if array.length() >= 2 {
                    let key = array.get(0).as_string().unwrap_or_default();
                    let value = array.get(1).as_string().unwrap_or_default();
                    response_headers.insert(key, value);
                }
            }
        }
    }

    // Get response body
    let body_text = if resp.body().is_some() {
        JsFuture::from(resp.text().map_err(|e| format!("Failed to get response text: {:?}", e))?)
            .await
            .map_err(|e| format!("Failed to read response body: {:?}", e))?
            .as_string()
            .unwrap_or_default()
    } else {
        String::new()
    };

    // Build response
    let response = serde_json::json!({
        "status": status,
        "status_text": status_text,
        "headers": response_headers,
        "body": body_text
    });

    log(&format!("Response received: status {}", status));

    Ok(response)
}