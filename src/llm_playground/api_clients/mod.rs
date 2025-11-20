// API client modules - Legacy modules (for backward compatibility)
pub mod conversation;
pub mod gemini_client;
pub mod openai_client;
pub mod traits;

// New SOLID-compliant modules
pub mod client_service;
pub mod factory_setup;
pub mod gemini_factory;
pub mod interfaces;
pub mod message_service;
pub mod openai_factory;
pub mod provider_factory;
pub mod refactored_openai_client;
pub mod refactored_flexible_client;
pub mod example_usage;

// Legacy exports (for backward compatibility)
pub use gemini_client::GeminiClient;
pub use openai_client::OpenAIClient;
pub use traits::{
    FunctionCallRequest, FunctionResponse, LLMClient,
    LLMResponse, MessageConverter, MessageSender, ModelProvider, NamedClient, StreamCallback,
    StreamingSender, UnifiedMessage, UnifiedMessageRole,
};

// New SOLID-compliant exports
pub use client_service::{ClientService, ClientServiceBuilder};
pub use factory_setup::{initialize_provider_system, create_test_provider_system};
pub use interfaces::{
    FunctionCaller,
};
pub use message_service::MessageConversionService;
pub use provider_factory::{ProviderFactory, ProviderRegistry};
pub use refactored_flexible_client::RefactoredFlexibleLLMClient;
