// New flexible provider configuration system
use crate::llm_playground::mcp_client::McpConfig;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub api_base_url: String,
    pub api_key: String,
    pub models: Vec<String>,
    pub transformer: TransformerConfig,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransformerConfig {
    pub r#use: Vec<String>, // "use" is a keyword, so we need r#use
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RouterConfig {
    pub default: String,
    pub background: String,
    pub think: String,
    pub long_context: String,
    pub long_context_threshold: u32,
    pub web_search: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FlexibleApiConfig {
    pub providers: Vec<ProviderConfig>,
    pub router: RouterConfig,
    pub shared_settings: SharedSettings,
    pub system_prompt: String,
    pub function_tools: Vec<FunctionTool>,
    pub structured_outputs: Vec<StructuredOutput>,
    pub mcp_config: McpConfig,
    // Session-specific settings
    pub current_session_provider: Option<String>, // Format: "provider_name,model_name"
}

// Re-export from types to avoid duplication
pub use crate::llm_playground::types::SharedSettings;

// Re-export from types to avoid duplication
pub use crate::llm_playground::types::FunctionTool;

// Re-export from types to avoid duplication
pub use crate::llm_playground::types::StructuredOutput;

impl Default for FlexibleApiConfig {
    fn default() -> Self {
        Self {
            providers: vec![
                ProviderConfig {
                    name: "openrouter".to_string(),
                    api_base_url: "https://openrouter.ai/api/v1".to_string(),
                    api_key: String::new(),
                    models: vec![
                        "deepseek/deepseek-chat-v3-0324:free".to_string(),
                        "microsoft/phi-3-mini-128k-instruct:free".to_string(),
                        "meta-llama/llama-3.1-8b-instruct:free".to_string(),
                    ],
                    transformer: TransformerConfig {
                        r#use: vec!["openai".to_string()],
                    },
                },
                ProviderConfig {
                    name: "gemini".to_string(),
                    api_base_url: "https://generativelanguage.googleapis.com/v1beta/models/".to_string(),
                    api_key: String::new(),
                    models: vec![
                        "gemini-2.5-flash".to_string(),
                        "gemini-2.5-pro".to_string(),
                        "gemini-2.5-flash-lite".to_string(),
                        "gemini-1.5-flash".to_string(),
                        "gemini-1.5-pro".to_string(),
                        "gemini-2.0-flash".to_string(),
                        "gemini-2.0-flash-lite".to_string(),
                    ],
                    transformer: TransformerConfig {
                        r#use: vec!["gemini".to_string()],
                    },
                },
                ProviderConfig {
                    name: "gemini-openai".to_string(),
                    api_base_url: "https://generativelanguage.googleapis.com/v1beta/openai".to_string(),
                    api_key: String::new(),
                    models: vec![
                        "gemini-2.5-flash".to_string(),
                        "gemini-2.5-pro".to_string(),
                        "gemini-2.5-flash-lite".to_string(),
                        "gemini-1.5-flash".to_string(),
                        "gemini-1.5-pro".to_string(),
                        "gemini-2.0-flash".to_string(),
                        "gemini-2.0-flash-lite".to_string(),
                    ],
                    transformer: TransformerConfig {
                        r#use: vec!["openai".to_string()],
                    },
                },
                ProviderConfig {
                    name: "openai".to_string(),
                    api_base_url: "https://api.openai.com/v1".to_string(),
                    api_key: String::new(),
                    models: vec![
                        "gpt-4o".to_string(),
                        "gpt-4o-mini".to_string(),
                        "gpt-4-turbo".to_string(),
                        "gpt-3.5-turbo".to_string(),
                    ],
                    transformer: TransformerConfig {
                        r#use: vec!["openai".to_string()],
                    },
                },
                ProviderConfig {
                    name: "ollama".to_string(),
                    api_base_url: "http://localhost:11434/v1".to_string(),
                    api_key: "ollama".to_string(), // Ollama doesn't need a real key
                    models: vec![
                        "llama3.2:latest".to_string(),
                        "llama3.1:latest".to_string(),
                        "mistral:latest".to_string(),
                        "codellama:latest".to_string(),
                    ],
                    transformer: TransformerConfig {
                        r#use: vec!["openai".to_string()],
                    },
                },
            ],
            router: RouterConfig {
                default: "openrouter,deepseek/deepseek-chat-v3-0324:free".to_string(),
                background: "openrouter,deepseek/deepseek-chat-v3-0324:free".to_string(),
                think: "openrouter,deepseek/deepseek-chat-v3-0324:free".to_string(),
                long_context: "openrouter,deepseek/deepseek-chat-v3-0324:free".to_string(),
                long_context_threshold: 60000,
                web_search: "openrouter,deepseek/deepseek-chat-v3-0324:free".to_string(),
            },
            shared_settings: SharedSettings {
                temperature: 0.7,
                max_tokens: 2048,
                retry_delay: 2000,
            },
            system_prompt: "You are a helpful assistant that responds in markdown format. Always be concise and to the point.".to_string(),
            function_tools: Self::get_default_function_tools(),
            structured_outputs: vec![],
            mcp_config: McpConfig::default(),
            current_session_provider: None,
        }
    }
}

impl FlexibleApiConfig {
    /// Get provider by name
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.iter().find(|p| p.name == name)
    }

    /// Get provider and model from session setting or default
    pub fn get_current_provider_and_model(&self) -> (String, String) {
        if let Some(ref session_provider) = self.current_session_provider {
            if let Some((provider, model)) = session_provider.split_once(',') {
                return (provider.to_string(), model.to_string());
            }
        }

        // Fall back to router default
        if let Some((provider, model)) = self.router.default.split_once(',') {
            (provider.to_string(), model.to_string())
        } else {
            // Ultimate fallback
            (
                "openrouter".to_string(),
                "deepseek/deepseek-chat-v3-0324:free".to_string(),
            )
        }
    }

    /// Set the current session provider and model
    pub fn set_session_provider(&mut self, provider_name: &str, model_name: &str) {
        self.current_session_provider = Some(format!("{},{}", provider_name, model_name));
    }

    /// Get all available provider-model combinations
    pub fn get_all_provider_models(&self) -> Vec<(String, String)> {
        let mut combinations = Vec::new();
        for provider in &self.providers {
            for model in &provider.models {
                combinations.push((provider.name.clone(), model.clone()));
            }
        }
        combinations
    }

    /// Get models for a specific provider
    pub fn get_models_for_provider(&self, provider_name: &str) -> Vec<String> {
        if let Some(provider) = self.get_provider(provider_name) {
            provider.models.clone()
        } else {
            vec![]
        }
    }

    /// Check if a provider uses a specific transformer
    pub fn provider_uses_transformer(&self, provider_name: &str, transformer: &str) -> bool {
        if let Some(provider) = self.get_provider(provider_name) {
            provider
                .transformer
                .r#use
                .contains(&transformer.to_string())
        } else {
            false
        }
    }

    /// Get default function tools (reuse from ApiConfig)
    pub fn get_default_function_tools() -> Vec<FunctionTool> {
        // Delegate to ApiConfig implementation to avoid duplication
        crate::llm_playground::types::ApiConfig::get_default_function_tools()
    }

    /// Toggle a function tool's enabled state
    pub fn toggle_function_tool(&mut self, tool_name: &str) {
        if let Some(tool) = self.function_tools.iter_mut().find(|t| t.name == tool_name) {
            tool.enabled = !tool.enabled;
        }
    }

    /// Get enabled function tools only
    pub fn get_enabled_function_tools(&self) -> Vec<&FunctionTool> {
        self.function_tools
            .iter()
            .filter(|tool| tool.enabled)
            .collect()
    }

    /// Get function tools by category
    pub fn get_function_tools_by_category(&self, category: &str) -> Vec<&FunctionTool> {
        self.function_tools
            .iter()
            .filter(|tool| tool.category == category)
            .collect()
    }

    /// Get all available categories
    pub fn get_function_tool_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self
            .function_tools
            .iter()
            .map(|tool| tool.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        categories.sort();
        categories
    }

    /// Add a new function tool
    pub fn add_function_tool(&mut self, tool: FunctionTool) {
        self.function_tools.push(tool);
    }

    /// Remove a function tool by name
    pub fn remove_function_tool(&mut self, tool_name: &str) {
        self.function_tools.retain(|tool| tool.name != tool_name);
    }

    /// Update a function tool's mock response
    pub fn update_tool_mock_response(&mut self, tool_name: &str, mock_response: String) {
        if let Some(tool) = self.function_tools.iter_mut().find(|t| t.name == tool_name) {
            tool.mock_response = mock_response;
        }
    }

    /// Enable all tools in a category
    pub fn enable_category(&mut self, category: &str) {
        for tool in self.function_tools.iter_mut() {
            if tool.category == category {
                tool.enabled = true;
            }
        }
    }

    /// Disable all tools in a category
    pub fn disable_category(&mut self, category: &str) {
        for tool in self.function_tools.iter_mut() {
            if tool.category == category {
                tool.enabled = false;
            }
        }
    }

    /// Get tool statistics
    pub fn get_tool_stats(&self) -> (usize, usize, usize) {
        let total = self.function_tools.len();
        let enabled = self.function_tools.iter().filter(|t| t.enabled).count();
        let categories = self.get_function_tool_categories().len();
        (total, enabled, categories)
    }

    /// Add MCP tools to the function tools list
    pub fn add_mcp_tools(&mut self, mcp_tools: Vec<FunctionTool>) {
        // Remove existing MCP tools first
        self.function_tools
            .retain(|tool| !tool.name.starts_with("mcp_"));

        // Add new MCP tools
        self.function_tools.extend(mcp_tools);
    }

    /// Get MCP configuration
    pub fn get_mcp_config(&self) -> &McpConfig {
        &self.mcp_config
    }

    /// Update MCP configuration
    pub fn update_mcp_config(&mut self, config: McpConfig) {
        self.mcp_config = config;
    }

    /// Get all function tools including MCP tools
    pub fn get_all_function_tools(&self) -> Vec<&FunctionTool> {
        self.function_tools.iter().collect()
    }
}
