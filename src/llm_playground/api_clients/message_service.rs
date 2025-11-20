// Message conversion service (SRP compliance)
use super::traits::{UnifiedMessage, UnifiedMessageRole};
use crate::llm_playground::{Message, MessageRole};

/// Service responsible for message format conversion
/// Follows SRP by having a single responsibility: message conversion
pub struct MessageConversionService;

impl MessageConversionService {
    pub fn new() -> Self {
        Self
    }

    /// Convert legacy messages to unified format
    pub fn convert_legacy_to_unified(&self, messages: &[Message]) -> Vec<UnifiedMessage> {
        messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    MessageRole::User => UnifiedMessageRole::User,
                    MessageRole::Assistant => UnifiedMessageRole::Assistant,
                    MessageRole::System => UnifiedMessageRole::System,
                    MessageRole::Function => UnifiedMessageRole::Assistant, // Map function to assistant
                };

                UnifiedMessage {
                    id: msg.id.clone(),
                    role,
                    content: Some(msg.content.clone()),
                    timestamp: msg.timestamp,
                    function_calls: vec![], // Legacy messages don't have function calls
                    function_responses: vec![],
                }
            })
            .collect()
    }

    /// Convert unified messages back to legacy format (if needed)
    pub fn convert_unified_to_legacy(&self, messages: &[UnifiedMessage]) -> Vec<Message> {
        messages
            .iter()
            .filter_map(|msg| {
                let role = match msg.role {
                    UnifiedMessageRole::User => MessageRole::User,
                    UnifiedMessageRole::Assistant => MessageRole::Assistant,
                    UnifiedMessageRole::System => MessageRole::System,
                };

                msg.content.as_ref().map(|content| Message {
                    id: msg.id.clone(),
                    role,
                    content: content.clone(),
                    timestamp: msg.timestamp,
                    function_call: None,
                    function_response: None,
                })
            })
            .collect()
    }
}

impl Default for MessageConversionService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_to_unified_conversion() {
        let service = MessageConversionService::new();
        let legacy_messages = vec![
            Message {
                id: "1".to_string(),
                role: MessageRole::User,
                content: "Hello".to_string(),
                timestamp: 123.0,
            }
        ];

        let unified = service.convert_legacy_to_unified(&legacy_messages);
        assert_eq!(unified.len(), 1);
        assert_eq!(unified[0].content, Some("Hello".to_string()));
        assert!(matches!(unified[0].role, UnifiedMessageRole::User));
    }
}