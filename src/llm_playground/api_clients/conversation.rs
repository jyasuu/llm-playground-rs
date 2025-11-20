use super::traits::{ConversationManager, ConversationMessage, FunctionResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct Conversation {
    history: Vec<ConversationMessage>,
    system_prompt: Option<String>,
}

impl ConversationManager for Conversation {
    fn add_user_message(&mut self, message: &str) {
        self.history.push(ConversationMessage {
            role: "user".to_string(),
            content: message.to_string(),
            function_call: None,
            function_response: None,
        });
    }

    fn add_assistant_message(&mut self, message: &str, function_call: Option<serde_json::Value>) {
        self.history.push(ConversationMessage {
            role: "assistant".to_string(),
            content: message.to_string(),
            function_call,
            function_response: None,
        });
    }

    fn add_function_response(&mut self, function_response: &FunctionResponse) {
        self.history.push(ConversationMessage {
            role: "tool".to_string(),
            content: serde_json::to_string(&function_response.response).unwrap_or_default(),
            function_call: None,
            function_response: Some(serde_json::json!({
                "id": function_response.id,
                "name": function_response.name,
                "response": function_response.response
            })),
        });
    }

    fn clear_conversation(&mut self) {
        self.history.clear();
    }

    fn set_system_prompt(&mut self, prompt: &str) {
        self.system_prompt = Some(prompt.to_string());
    }

    fn get_conversation_history(&self) -> &[ConversationMessage] {
        &self.history
    }
}
