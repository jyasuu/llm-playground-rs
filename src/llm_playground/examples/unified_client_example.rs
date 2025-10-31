// Example demonstrating the unified LLM client system
// This shows how the refactored system addresses all TODO requirements

use crate::llm_playground::{
    unified_client::{UnifiedLLMClient, UnifiedFunctionCall, UnifiedFunctionResponse},
    ApiConfig, ApiProvider,
};
use gloo_console::log;
use serde_json::json;

/// Example demonstrating the unified client for both Gemini and OpenAI
pub async fn unified_client_example() {
    log!("=== Unified LLM Client Example ===");
    
    // 1. Create a unified client - works with both providers
    let mut client = UnifiedLLMClient::new();
    
    // 2. System prompt is handled internally by each provider
    // No need for caller to worry about provider-specific differences
    client.set_system_prompt("You are a helpful assistant that responds in markdown format. Always be concise and to the point.");
    
    // 3. Add user message
    let user_msg_id = client.add_user_message("help me use fetch tool to access https://httpbin.org/get and https://httpbin.org/post");
    log!(format!("Added user message with ID: {}", user_msg_id));
    
    // 4. Example configuration - we can switch providers easily
    let mut config = ApiConfig::default();
    
    // Example 1: Using Gemini
    config.current_provider = ApiProvider::Gemini;
    config.gemini.api_key = "your-gemini-api-key".to_string();
    config.gemini.model = "gemini-2.5-flash-lite-preview-06-17".to_string();
    
    log!("--- Sending to Gemini ---");
    match client.send_message(&config).await {
        Ok(response) => {
            log!("Gemini response received");
            
            // Handle text response
            if let Some(content) = &response.content {
                log!(format!("Response content: {}", content));
            }
            
            // Handle function calls - unified format regardless of provider
            if !response.function_calls.is_empty() {
                log!(format!("Function calls: {}", response.function_calls.len()));
                
                // Convert to unified format
                let unified_calls = client.convert_function_calls_to_unified(response.function_calls);
                
                // Add assistant message with function calls
                let assistant_msg_id = client.add_assistant_message(response.content, unified_calls.clone());
                log!(format!("Added assistant message with ID: {}", assistant_msg_id));
                
                // Simulate function execution
                let mut function_results = Vec::new();
                for call in &unified_calls {
                    log!(format!("Executing function: {} with args: {}", call.name, call.arguments));
                    
                    // Mock function execution
                    let result = match call.name.as_str() {
                        "fetch" => {
                            let url = call.arguments.get("url").and_then(|v| v.as_str()).unwrap_or("unknown");
                            json!({
                                "status": 200,
                                "headers": {"content-type": "application/json"},
                                "body": format!("{{\"url\": \"{}\", \"method\": \"GET\"}}", url)
                            })
                        }
                        _ => json!({"error": "Unknown function"})
                    };
                    
                    function_results.push(result);
                }
                
                // Create unified function responses
                let unified_responses = client.create_function_responses(&unified_calls, function_results);
                
                // Add function responses
                let tool_msg_id = client.add_function_responses(unified_responses);
                log!(format!("Added tool responses with ID: {}", tool_msg_id));
            }
        }
        Err(e) => {
            log!(format!("Gemini error: {}", e));
        }
    }
    
    // Example 2: Switch to OpenAI - same conversation continues seamlessly
    config.current_provider = ApiProvider::OpenAI;
    config.openai.api_key = "your-openai-api-key".to_string();
    config.openai.model = "gpt-4".to_string();
    config.openai.base_url = "https://api.openai.com/v1".to_string();
    
    log!("--- Switching to OpenAI ---");
    
    // Add another user message
    client.add_user_message("Now summarize what we just did");
    
    match client.send_message(&config).await {
        Ok(response) => {
            log!("OpenAI response received");
            if let Some(content) = &response.content {
                log!(format!("Response content: {}", content));
            }
            
            // Add the response to conversation
            let assistant_msg_id = client.add_assistant_message(
                response.content, 
                client.convert_function_calls_to_unified(response.function_calls)
            );
            log!(format!("Added OpenAI response with ID: {}", assistant_msg_id));
        }
        Err(e) => {
            log!(format!("OpenAI error: {}", e));
        }
    }
    
    // 5. Show conversation state
    client.log_conversation_state();
    
    // 6. Demonstrate backward compatibility
    log!("--- Backward Compatibility ---");
    let legacy_messages = client.to_legacy_messages();
    log!(format!("Converted to {} legacy messages", legacy_messages.len()));
    
    // 7. Show how the unified system handles provider differences automatically:
    log!("--- Provider Difference Handling ---");
    log!("✓ System prompt: Handled internally by each provider");
    log!("✓ Function call IDs: Generated appropriately for each provider");
    log!("✓ Message structure: Converted automatically");
    log!("✓ UI and storage: Uses unified data model");
}

/// Example showing function call ID generation differences
pub fn function_call_id_example() {
    let client = UnifiedLLMClient::new();
    
    // Gemini style IDs
    let gemini_id = client.generate_function_call_id("fetch", &ApiProvider::Gemini);
    log!(format!("Gemini ID: {}", gemini_id)); // gemini-fetch-1234567890
    
    // OpenAI style IDs  
    let openai_id = client.generate_function_call_id("fetch", &ApiProvider::OpenAI);
    log!(format!("OpenAI ID: {}", openai_id)); // call_1234567890
}

/// Example showing data structure conversion
pub fn data_structure_example() {
    log!("=== Data Structure Conversion Example ===");
    
    // Create unified messages
    let mut client = UnifiedLLMClient::new();
    client.set_system_prompt("Test system prompt");
    client.add_user_message("Hello");
    
    // Add assistant message with function call
    let function_calls = vec![UnifiedFunctionCall {
        id: "call_123".to_string(),
        name: "fetch".to_string(),
        arguments: json!({"url": "https://example.com"}),
    }];
    client.add_assistant_message(Some("I'll fetch that for you".to_string()), function_calls);
    
    // Add function response
    let function_responses = vec![UnifiedFunctionResponse {
        id: "call_123".to_string(),
        name: "fetch".to_string(),
        content: json!({"status": 200, "body": "success"}),
    }];
    client.add_function_responses(function_responses);
    
    log!("Unified conversation created with all message types");
    client.log_conversation_state();
    
    // Convert to legacy format for backward compatibility
    let legacy_messages = client.to_legacy_messages();
    log!(format!("Converted to {} legacy messages", legacy_messages.len()));
    
    // Show how to recreate from legacy
    let restored_client = UnifiedLLMClient::from_legacy_messages(&legacy_messages);
    log!("Restored client from legacy messages");
    restored_client.log_conversation_state();
}

/// Main example function to run all demonstrations
pub async fn run_all_examples() {
    log!("🚀 Starting Unified LLM Client Examples");
    
    // Run the main example
    unified_client_example().await;
    
    // Run supporting examples
    function_call_id_example();
    data_structure_example();
    
    log!("✅ All examples completed!");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_unified_client_creation() {
        let client = UnifiedLLMClient::new();
        assert!(client.get_conversation().messages.is_empty());
        assert!(client.get_conversation().system_prompt.is_none());
    }
    
    #[test]
    fn test_message_flow() {
        let mut client = UnifiedLLMClient::new();
        
        // Add system prompt
        client.set_system_prompt("Test prompt");
        assert_eq!(client.get_conversation().system_prompt, Some("Test prompt".to_string()));
        
        // Add user message
        let user_id = client.add_user_message("Hello");
        assert_eq!(client.get_conversation().messages.len(), 1);
        
        // Add assistant message with function call
        let function_calls = vec![UnifiedFunctionCall {
            id: "test_id".to_string(),
            name: "test_func".to_string(),
            arguments: json!({"arg": "value"}),
        }];
        let assistant_id = client.add_assistant_message(Some("Response".to_string()), function_calls);
        assert_eq!(client.get_conversation().messages.len(), 2);
        
        // Add function response
        let responses = vec![UnifiedFunctionResponse {
            id: "test_id".to_string(),
            name: "test_func".to_string(),
            content: json!({"result": "success"}),
        }];
        let tool_id = client.add_function_responses(responses);
        assert_eq!(client.get_conversation().messages.len(), 3);
        
        // Verify message IDs are different
        assert_ne!(user_id, assistant_id);
        assert_ne!(assistant_id, tool_id);
    }
    
    #[test]
    fn test_legacy_compatibility() {
        // Create some legacy messages
        let legacy_messages = vec![
            crate::llm_playground::Message {
                id: "msg1".to_string(),
                role: crate::llm_playground::MessageRole::User,
                content: "Hello".to_string(),
                timestamp: 1234567890.0,
                function_call: None,
                function_response: None,
            }
        ];
        
        // Convert to unified
        let client = UnifiedLLMClient::from_legacy_messages(&legacy_messages);
        assert_eq!(client.get_conversation().messages.len(), 1);
        
        // Convert back to legacy
        let restored_legacy = client.to_legacy_messages();
        assert_eq!(restored_legacy.len(), 1);
        assert_eq!(restored_legacy[0].content, "Hello");
    }
}