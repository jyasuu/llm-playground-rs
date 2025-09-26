// Type definitions for LLM Playground
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ApiProvider {
    Gemini,
    OpenAI,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApiConfig {
    pub gemini: GeminiConfig,
    pub openai: OpenAIConfig,
    pub current_provider: ApiProvider,
    pub shared_settings: SharedSettings,
    pub system_prompt: String,
    pub function_tools: Vec<FunctionTool>,
    pub structured_outputs: Vec<StructuredOutput>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeminiConfig {
    pub api_key: String,
    pub model: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SharedSettings {
    pub temperature: f32,
    pub max_tokens: u32,
    pub retry_delay: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FunctionTool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub mock_response: String,
    pub enabled: bool,
    pub category: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StructuredOutput {
    pub name: String,
    pub schema: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: f64,
    pub function_call: Option<serde_json::Value>,
    pub function_response: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Function,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub messages: Vec<Message>,
    pub created_at: f64,
    pub updated_at: f64,
    pub pinned: bool,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            gemini: GeminiConfig {
                api_key: String::new(),
                model: "gemini-2.5-flash-lite-preview-06-17".to_string(),
            },
            openai: OpenAIConfig {
                base_url: "https://api.openai.com/v1".to_string(),
                api_key: String::new(),
                model: "gpt-4o".to_string(),
            },
            current_provider: ApiProvider::Gemini,
            shared_settings: SharedSettings {
                temperature: 0.7,
                max_tokens: 2048,
                retry_delay: 2000,
            },
            system_prompt: "You are a helpful assistant that responds in markdown format. Always be concise and to the point.".to_string(),
            function_tools: Self::get_default_function_tools(),
            structured_outputs: vec![],
        }
    }
}

impl ApiConfig {
    /// Get default function tools with all tools from the specification
    pub fn get_default_function_tools() -> Vec<FunctionTool> {
        vec![
            // Task Agent Tool
            FunctionTool {
                name: "Task".to_string(),
                description: "Launch a new agent to handle complex, multi-step tasks autonomously. Available agent types: general-purpose (Tools: *)".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "description": {
                            "type": "string",
                            "description": "A short (3-5 word) description of the task"
                        },
                        "prompt": {
                            "type": "string",
                            "description": "The task for the agent to perform"
                        },
                        "subagent_type": {
                            "type": "string",
                            "description": "The type of specialized agent to use for this task"
                        }
                    },
                    "required": ["description", "prompt", "subagent_type"]
                }),
                mock_response: r#"{"task_id": "task_123", "status": "created", "agent_type": "general-purpose", "description": "Search for code"}"#.to_string(),
                enabled: true,
                category: "Agent".to_string(),
            },

            // Bash Tool
            FunctionTool {
                name: "Bash".to_string(),
                description: "Executes a given bash command in a persistent shell session with optional timeout, ensuring proper handling and security measures.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The command to execute"
                        },
                        "timeout": {
                            "type": "number",
                            "description": "Optional timeout in milliseconds (max 600000)"
                        },
                        "description": {
                            "type": "string",
                            "description": "Clear, concise description of what this command does in 5-10 words"
                        }
                    },
                    "required": ["command"]
                }),
                mock_response: r#"{"stdout": "total 12\ndrwxr-xr-x 3 user user 4096 Jan 1 12:00 src\n-rw-r--r-- 1 user user 1234 Jan 1 12:00 Cargo.toml", "stderr": "", "exit_code": 0}"#.to_string(),
                enabled: true,
                category: "System".to_string(),
            },

            // Glob Tool
            FunctionTool {
                name: "Glob".to_string(),
                description: "Fast file pattern matching tool that works with any codebase size. Supports glob patterns like '**/*.js' or 'src/**/*.ts'. Returns matching file paths sorted by modification time.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "pattern": {
                            "type": "string",
                            "description": "The glob pattern to match files against"
                        },
                        "path": {
                            "type": "string",
                            "description": "The directory to search in. If not specified, the current working directory will be used."
                        }
                    },
                    "required": ["pattern"]
                }),
                mock_response: r#"{"files": ["src/main.rs", "src/lib.rs", "tests/integration.rs"], "count": 3}"#.to_string(),
                enabled: true,
                category: "File System".to_string(),
            },

            // Grep Tool
            FunctionTool {
                name: "Grep".to_string(),
                description: "A powerful search tool built on ripgrep. Supports full regex syntax and various output modes.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "pattern": {
                            "type": "string",
                            "description": "The regular expression pattern to search for in file contents"
                        },
                        "path": {
                            "type": "string",
                            "description": "File or directory to search in. Defaults to current working directory."
                        },
                        "glob": {
                            "type": "string",
                            "description": "Glob pattern to filter files (e.g. '*.js', '*.{ts,tsx}')"
                        },
                        "output_mode": {
                            "type": "string",
                            "enum": ["content", "files_with_matches", "count"],
                            "description": "Output mode: 'content' shows matching lines, 'files_with_matches' shows file paths, 'count' shows match counts"
                        },
                        "-i": {
                            "type": "boolean",
                            "description": "Case insensitive search"
                        }
                    },
                    "required": ["pattern"]
                }),
                mock_response: r#"{"matches": [{"file": "src/main.rs", "line": 42, "content": "fn main() {"}], "total_matches": 1}"#.to_string(),
                enabled: true,
                category: "Search".to_string(),
            },

            // LS Tool
            FunctionTool {
                name: "LS".to_string(),
                description: "Lists files and directories in a given path. The path parameter must be an absolute path.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The absolute path to the directory to list"
                        },
                        "ignore": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "List of glob patterns to ignore"
                        }
                    },
                    "required": ["path"]
                }),
                mock_response: r#"{"entries": [{"name": "src", "type": "directory", "size": 4096}, {"name": "Cargo.toml", "type": "file", "size": 1234}]}"#.to_string(),
                enabled: true,
                category: "File System".to_string(),
            },

            // Read Tool
            FunctionTool {
                name: "Read".to_string(),
                description: "Reads a file from the local filesystem. Can read up to 2000 lines by default with optional offset and limit.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "The absolute path to the file to read"
                        },
                        "offset": {
                            "type": "number",
                            "description": "The line number to start reading from"
                        },
                        "limit": {
                            "type": "number",
                            "description": "The number of lines to read"
                        }
                    },
                    "required": ["file_path"]
                }),
                mock_response: r#"{"content": "use std::collections::HashMap;\n\nfn main() {\n    println!(\"Hello, world!\");\n}", "lines": 4, "truncated": false}"#.to_string(),
                enabled: true,
                category: "File System".to_string(),
            },

            // Edit Tool
            FunctionTool {
                name: "Edit".to_string(),
                description: "Performs exact string replacements in files. Must use Read tool before editing.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "The absolute path to the file to modify"
                        },
                        "old_string": {
                            "type": "string",
                            "description": "The text to replace"
                        },
                        "new_string": {
                            "type": "string",
                            "description": "The text to replace it with"
                        },
                        "replace_all": {
                            "type": "boolean",
                            "description": "Replace all occurrences of old_string"
                        }
                    },
                    "required": ["file_path", "old_string", "new_string"]
                }),
                mock_response: r#"{"success": true, "replacements": 1, "file": "/path/to/file.rs"}"#.to_string(),
                enabled: true,
                category: "File System".to_string(),
            },

            // Write Tool
            FunctionTool {
                name: "Write".to_string(),
                description: "Writes a file to the local filesystem. Will overwrite existing files.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "The absolute path to the file to write"
                        },
                        "content": {
                            "type": "string",
                            "description": "The content to write to the file"
                        }
                    },
                    "required": ["file_path", "content"]
                }),
                mock_response: r#"{"success": true, "bytes_written": 1234, "file": "/path/to/file.rs"}"#.to_string(),
                enabled: true,
                category: "File System".to_string(),
            },

            // WebFetch Tool
            FunctionTool {
                name: "WebFetch".to_string(),
                description: "Fetches content from a specified URL and processes it using an AI model. Converts HTML to markdown.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "format": "uri",
                            "description": "The URL to fetch content from"
                        },
                        "prompt": {
                            "type": "string",
                            "description": "The prompt to run on the fetched content"
                        }
                    },
                    "required": ["url", "prompt"]
                }),
                mock_response: "{\"title\": \"Example Page\", \"content\": \"# Example\\n\\nThis is example content from the webpage.\", \"summary\": \"A webpage about examples\"}".to_string(),
                enabled: true,
                category: "Web".to_string(),
            },

            // WebSearch Tool
            FunctionTool {
                name: "WebSearch".to_string(),
                description: "Allows searching the web and using results to inform responses. Provides up-to-date information.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "minLength": 2,
                            "description": "The search query to use"
                        },
                        "allowed_domains": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "Only include search results from these domains"
                        },
                        "blocked_domains": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "Never include search results from these domains"
                        }
                    },
                    "required": ["query"]
                }),
                mock_response: r#"{"results": [{"title": "Rust Programming Language", "url": "https://rust-lang.org", "snippet": "A systems programming language..."}], "total": 1}"#.to_string(),
                enabled: true,
                category: "Web".to_string(),
            },

            // Weather Tool (Enhanced)
            FunctionTool {
                name: "get_weather".to_string(),
                description: "Retrieves weather data for a specified location with temperature unit options.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "The location to get weather for"
                        },
                        "unit": {
                            "type": "string",
                            "enum": ["celsius", "fahrenheit"],
                            "description": "Temperature unit"
                        }
                    },
                    "required": ["location"]
                }),
                mock_response: r#"{"temperature": 22, "condition": "sunny", "humidity": 65, "wind_speed": 5, "location": "San Francisco, CA"}"#.to_string(),
                enabled: true,
                category: "Weather".to_string(),
            },

            // IDE Diagnostics Tool
            FunctionTool {
                name: "mcp__ide__getDiagnostics".to_string(),
                description: "Get language diagnostics from VS Code for syntax errors, warnings, and other issues.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "uri": {
                            "type": "string",
                            "description": "Optional file URI to get diagnostics for. If not provided, gets diagnostics for all files."
                        }
                    }
                }),
                mock_response: r#"{"diagnostics": [{"file": "src/main.rs", "line": 42, "severity": "error", "message": "cannot find value `x` in this scope"}]}"#.to_string(),
                enabled: false,
                category: "IDE".to_string(),
            },

            // Execute Code Tool
            FunctionTool {
                name: "mcp__ide__executeCode".to_string(),
                description: "Execute python code in the Jupyter kernel for the current notebook file. Code persists across calls.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "The code to be executed on the kernel."
                        }
                    },
                    "required": ["code"]
                }),
                mock_response: r#"{"output": "Hello, World!\n", "execution_count": 1, "status": "ok"}"#.to_string(),
                enabled: false,
                category: "IDE".to_string(),
            },
        ]
    }

    /// Toggle a function tool's enabled state
    pub fn toggle_function_tool(&mut self, tool_name: &str) {
        if let Some(tool) = self.function_tools.iter_mut().find(|t| t.name == tool_name) {
            tool.enabled = !tool.enabled;
        }
    }

    /// Get enabled function tools only
    pub fn get_enabled_function_tools(&self) -> Vec<&FunctionTool> {
        self.function_tools.iter().filter(|tool| tool.enabled).collect()
    }

    /// Get function tools by category
    pub fn get_function_tools_by_category(&self, category: &str) -> Vec<&FunctionTool> {
        self.function_tools.iter().filter(|tool| tool.category == category).collect()
    }

    /// Get all available categories
    pub fn get_function_tool_categories(&self) -> Vec<String> {
        let mut categories: Vec<String> = self.function_tools
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
}