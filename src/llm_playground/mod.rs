// LLM Playground module
pub mod api_clients;
pub mod builtin_tools;
pub mod components;
pub mod flexible_client;
pub mod flexible_playground;
pub mod hooks;
pub mod mcp_client;
pub mod provider_config;
pub mod storage;
pub mod types;

pub use api_clients::*;
pub use components::*;
pub use flexible_playground::FlexibleLLMPlayground;
pub use hooks::*;
pub use provider_config::FlexibleApiConfig;
pub use storage::*;
pub use types::*;

use crate::llm_playground::api_clients::{GeminiClient, OpenAIClient};
use gloo_console::log;
use std::collections::HashMap;
use yew::prelude::*;
