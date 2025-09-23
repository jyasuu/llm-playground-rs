// Local storage utilities for LLM Playground
use gloo_storage::{LocalStorage, Storage};
use std::collections::HashMap;
use super::{ChatSession, ApiConfig};

const SESSIONS_KEY: &str = "llm_playground_sessions";
const CONFIG_KEY: &str = "llm_playground_config";
const CURRENT_SESSION_KEY: &str = "llm_playground_current_session";

pub struct StorageManager;

impl StorageManager {
    pub fn save_sessions(sessions: &HashMap<String, ChatSession>) -> Result<(), gloo_storage::errors::StorageError> {
        LocalStorage::set(SESSIONS_KEY, sessions)
    }

    pub fn load_sessions() -> Result<HashMap<String, ChatSession>, gloo_storage::errors::StorageError> {
        LocalStorage::get(SESSIONS_KEY)
    }

    pub fn save_config(config: &ApiConfig) -> Result<(), gloo_storage::errors::StorageError> {
        LocalStorage::set(CONFIG_KEY, config)
    }

    pub fn load_config() -> Result<ApiConfig, gloo_storage::errors::StorageError> {
        LocalStorage::get(CONFIG_KEY)
    }

    pub fn save_current_session_id(session_id: &str) -> Result<(), gloo_storage::errors::StorageError> {
        LocalStorage::set(CURRENT_SESSION_KEY, session_id)
    }

    pub fn load_current_session_id() -> Result<String, gloo_storage::errors::StorageError> {
        LocalStorage::get(CURRENT_SESSION_KEY)
    }

    pub fn clear_all() -> Result<(), gloo_storage::errors::StorageError> {
        LocalStorage::delete(SESSIONS_KEY);
        LocalStorage::delete(CONFIG_KEY);
        LocalStorage::delete(CURRENT_SESSION_KEY);
        Ok(())
    }

    pub fn export_data() -> Result<String, Box<dyn std::error::Error>> {
        let sessions = Self::load_sessions().unwrap_or_default();
        let config = Self::load_config().unwrap_or_default();
        
        let export_data = serde_json::json!({
            "sessions": sessions,
            "config": config,
            "exported_at": js_sys::Date::now()
        });
        
        Ok(serde_json::to_string_pretty(&export_data)?)
    }

    pub fn import_data(json_data: &str) -> Result<(), Box<dyn std::error::Error>> {
        let import_data: serde_json::Value = serde_json::from_str(json_data)?;
        
        if let Some(sessions) = import_data.get("sessions") {
            let sessions: HashMap<String, ChatSession> = serde_json::from_value(sessions.clone())?;
            Self::save_sessions(&sessions)?;
        }
        
        if let Some(config) = import_data.get("config") {
            let config: ApiConfig = serde_json::from_value(config.clone())?;
            Self::save_config(&config)?;
        }
        
        Ok(())
    }
}