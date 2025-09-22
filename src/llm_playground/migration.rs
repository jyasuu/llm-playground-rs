// Migration utilities for converting old config to new flexible config
use crate::llm_playground::{ApiConfig, ApiProvider, FlexibleApiConfig, ProviderConfig, TransformerConfig};

pub fn migrate_old_config_to_flexible(old_config: ApiConfig) -> FlexibleApiConfig {
    let mut flexible_config = FlexibleApiConfig::default();
    
    // Migrate settings
    flexible_config.shared_settings = old_config.shared_settings;
    flexible_config.system_prompt = old_config.system_prompt;
    flexible_config.function_tools = old_config.function_tools;
    flexible_config.structured_outputs = old_config.structured_outputs;
    
    // Update provider configurations with user's API keys and models
    for provider in &mut flexible_config.providers {
        match provider.name.as_str() {
            "gemini" | "gemini-openai" => {
                if !old_config.gemini.api_key.is_empty() {
                    provider.api_key = old_config.gemini.api_key.clone();
                }
                // Update model if it's not in the default list
                if !provider.models.contains(&old_config.gemini.model) && !old_config.gemini.model.is_empty() {
                    provider.models.insert(0, old_config.gemini.model.clone());
                }
            },
            "openai" => {
                if !old_config.openai.api_key.is_empty() {
                    provider.api_key = old_config.openai.api_key.clone();
                }
                if !old_config.openai.base_url.is_empty() && old_config.openai.base_url != "https://api.openai.com/v1" {
                    provider.api_base_url = format!("{}/chat/completions", old_config.openai.base_url.trim_end_matches("/chat/completions").trim_end_matches("/"));
                }
                // Update model if it's not in the default list
                if !provider.models.contains(&old_config.openai.model) && !old_config.openai.model.is_empty() {
                    provider.models.insert(0, old_config.openai.model.clone());
                }
            },
            _ => {}
        }
    }
    
    // Set the current session provider based on the old current provider
    match old_config.current_provider {
        ApiProvider::Gemini => {
            flexible_config.set_session_provider("gemini", &old_config.gemini.model);
        },
        ApiProvider::OpenAI => {
            flexible_config.set_session_provider("openai", &old_config.openai.model);
        },
    }
    
    flexible_config
}

pub fn detect_old_config_exists() -> bool {
    use gloo_storage::{LocalStorage, Storage};
    LocalStorage::get::<String>("llm_playground_config").is_ok()
}

pub fn migrate_if_needed() -> Option<FlexibleApiConfig> {
    use gloo_storage::{LocalStorage, Storage};
    
    // Check if old config exists and new config doesn't
    if let Ok(old_config_str) = LocalStorage::get::<String>("llm_playground_config") {
        if LocalStorage::get::<String>("llm_playground_flexible_config").is_err() {
            // Migration needed
            if let Ok(old_config) = serde_json::from_str::<ApiConfig>(&old_config_str) {
                let flexible_config = migrate_old_config_to_flexible(old_config);
                
                // Save the new config
                if let Ok(new_config_str) = serde_json::to_string(&flexible_config) {
                    let _ = LocalStorage::set("llm_playground_flexible_config", new_config_str);
                }
                
                // Optionally remove old config (keep it for now for safety)
                // let _ = LocalStorage::delete("llm_playground_config");
                
                return Some(flexible_config);
            }
        }
    }
    
    None
}