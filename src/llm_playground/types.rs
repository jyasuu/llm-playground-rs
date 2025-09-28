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
    pub is_builtin: bool,
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
            // Built-in Fetch Tool
            FunctionTool {
                name: "fetch".to_string(),
                description: "A tool for making HTTP requests. Supports GET, POST, PUT, DELETE, and other HTTP methods with custom headers and payload.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "The URL to make the request to"
                        },
                        "method": {
                            "type": "string",
                            "description": "HTTP method to use (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS)"
                        },
                        "headers": {
                            "type": "object",
                            "description": "HTTP headers to include in the request"
                        },
                        "payload": {
                            "type": "string",
                            "description": "Request body payload (for POST, PUT, PATCH methods)"
                        }
                    },
                    "required": ["url"]
                }),
                mock_response: r#"{"status": 200, "headers": {"content-type": "application/json"}, "body": "{\"message\": \"success\"}"}"#.to_string(),
                enabled: true,
                category: "HTTP".to_string(),
                is_builtin: true,
            },

            // Task Agent Tool
            FunctionTool {
                name: "Task".to_string(),
                description: "Launch a new agent to handle complex, multi-step tasks autonomously. \n\nAvailable agent types and the tools they have access to:\n- general-purpose: General-purpose agent for researching complex questions, searching for code, and executing multi-step tasks. When you are searching for a keyword or file and are not confident that you will find the right match in the first few tries use this agent to perform the search for you. (Tools: *)\n\nWhen using the Task tool, you must specify the type of agent to use for the task.".to_string(),
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
                    "required": ["description", "prompt", "subagent_type"],
                    "additionalProperties": false,
                    "$schema": "http://json-schema.org/draft-07/schema#"
                }),
                mock_response: r#"{"task_id": "task_123", "status": "created", "agent_type": "general-purpose", "description": "Search for code"}"#.to_string(),
                enabled: true,
                category: "Agent".to_string(),
                is_builtin: false,
            },

            // Bash Tool
            FunctionTool {
                name: "Bash".to_string(),
                description: "Executes a given bash command in a persistent shell session with optional timeout, ensuring proper handling and security measures.\n\nBefore executing the command, please follow these steps:\n\n1. Directory Verification:\n   - If the command will create new directories or files, first use the LS tool to verify the parent directory exists and is the correct location\n   - For example, before running \"mkdir foo/bar\", first use LS to check that \"foo\" exists and is the intended parent directory\n\n2. Safety Checks:\n   - Verify you're in the correct working directory before running commands that modify files\n   - Use absolute paths when possible to avoid ambiguity\n   - Be cautious with destructive commands (rm, mv, etc.)\n\n3. Command Construction:\n   - Use proper shell escaping for arguments with spaces or special characters\n   - Consider using quotes around file paths and arguments\n   - Test complex commands in a safe environment first".to_string(),
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
                            "description": " Clear, concise description of what this command does in 5-10 words. Examples:\nInput: ls\nOutput: Lists files in current directory\n\nInput: git status\nOutput: Shows working tree status\n\nInput: npm install\nOutput: Installs package dependencies\n\nInput: mkdir foo\nOutput: Creates directory 'foo'"
                        }
                    },
                    "required": ["command"],
                    "additionalProperties": false,
                    "$schema": "http://json-schema.org/draft-07/schema#"
                }),
                mock_response: r#"{"stdout": "total 12\ndrwxr-xr-x 3 user user 4096 Jan 1 12:00 src\n-rw-r--r-- 1 user user 1234 Jan 1 12:00 Cargo.toml", "stderr": "", "exit_code": 0}"#.to_string(),
                enabled: true,
                category: "System".to_string(),
                is_builtin: false,
            },

            // Glob Tool
            FunctionTool {
                name: "Glob".to_string(),
                description: "- Fast file pattern matching tool that works with any codebase size\n- Supports glob patterns like \"**/*.js\" or \"src/**/*.ts\"\n- Returns matching file paths sorted by modification time\n- Use this tool when you need to find files by name patterns\n- When you are doing an open ended search that may require multiple rounds of globbing and grepping, use the Agent tool instead\n- You have the capability to call multiple tools in a single response. It is always better to search with glob first, then grep the results if needed".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "pattern": {
                            "type": "string",
                            "description": "The glob pattern to match files against"
                        },
                        "path": {
                            "type": "string",
                            "description": "The directory to search in. If not specified, the current working directory will be used. IMPORTANT: Omit this field to use the default directory. DO NOT enter \"undefined\" or \"null\" - simply omit it for the default behavior. Must be a valid directory path if provided."
                        }
                    },
                    "required": ["pattern"],
                    "additionalProperties": false,
                    "$schema": "http://json-schema.org/draft-07/schema#"
                }),
                mock_response: r#"{"files": ["src/main.rs", "src/lib.rs", "tests/integration.rs"], "count": 3}"#.to_string(),
                enabled: true,
                category: "File System".to_string(),
                is_builtin: false,
            },

            // Grep Tool
            FunctionTool {
                name: "Grep".to_string(),
                description: "A powerful search tool built on ripgrep\n\n  Usage:\n  - ALWAYS use Grep for search tasks. NEVER invoke `grep` or `rg` as a Bash command. The Grep tool has been optimized for correct permissions and access.\n  - Supports full regex syntax (e.g., \"log.*Error\", \"function\\s+\\w+\")\n  - Filter files with glob parameter (e.g., \"*.js\", \"**/*.tsx\") or type parameter (e.g., \"js\", \"py\", \"rust\")\n  - Output modes: \"content\" shows matching lines, \"files_with_matches\" shows file paths, \"count\" shows match counts\n  - Use context options (-A, -B, -C) to see surrounding lines when using \"content\" mode".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "pattern": {
                            "type": "string",
                            "description": "The regular expression pattern to search for in file contents"
                        },
                        "path": {
                            "type": "string",
                            "description": "File or directory to search in (rg PATH). Defaults to current working directory."
                        },
                        "glob": {
                            "type": "string",
                            "description": "Glob pattern to filter files (e.g. \"*.js\", \"*.{ts,tsx}\") - maps to rg --glob"
                        },
                        "output_mode": {
                            "type": "string",
                            "enum": ["content", "files_with_matches", "count"],
                            "description": "Output mode: \"content\" shows matching lines (supports -A/-B/-C context, -n line numbers, head_limit), \"files_with_matches\" shows file paths (supports head_limit), \"count\" shows match counts (supports head_limit). Defaults to \"files_with_matches\"."
                        },
                        "-B": {
                            "type": "number",
                            "description": "Number of lines to show before each match (rg -B). Requires output_mode: \"content\", ignored otherwise."
                        },
                        "-A": {
                            "type": "number",
                            "description": "Number of lines to show after each match (rg -A). Requires output_mode: \"content\", ignored otherwise."
                        },
                        "-C": {
                            "type": "number",
                            "description": "Number of lines to show before and after each match (rg -C). Requires output_mode: \"content\", ignored otherwise."
                        },
                        "-n": {
                            "type": "boolean",
                            "description": "Show line numbers in output (rg -n). Requires output_mode: \"content\", ignored otherwise."
                        },
                        "-i": {
                            "type": "boolean",
                            "description": "Case insensitive search (rg -i)"
                        },
                        "type": {
                            "type": "string",
                            "description": "File type to search (rg --type). Common types: js, py, rust, go, java, etc. More efficient than include for standard file types."
                        },
                        "head_limit": {
                            "type": "number",
                            "description": "Limit output to first N lines/entries, equivalent to \"| head -N\". Works across all output modes: content (limits output lines), files_with_matches (limits file paths), count (limits count entries). When unspecified, shows all results from ripgrep."
                        },
                        "multiline": {
                            "type": "boolean",
                            "description": "Enable multiline mode where . matches newlines and patterns can span lines (rg -U --multiline-dotall). Default: false."
                        }
                    },
                    "required": ["pattern"],
                    "additionalProperties": false,
                    "$schema": "http://json-schema.org/draft-07/schema#"
                }),
                mock_response: r#"{"matches": [{"file": "src/main.rs", "line": 42, "content": "fn main() {"}], "total_matches": 1}"#.to_string(),
                enabled: true,
                category: "Search".to_string(),
                is_builtin: false,
            },

            // LS Tool
            FunctionTool {
                name: "LS".to_string(),
                description: "Lists files and directories in a given path. The path parameter must be an absolute path, not a relative path. You can optionally provide an array of glob patterns to ignore with the ignore parameter. You should generally prefer the Glob and Grep tools, if you know which directories to search.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "The absolute path to the directory to list (must be absolute, not relative)"
                        },
                        "ignore": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "List of glob patterns to ignore"
                        }
                    },
                    "required": ["path"],
                    "additionalProperties": false,
                    "$schema": "http://json-schema.org/draft-07/schema#"
                }),
                mock_response: r#"{"entries": [{"name": "src", "type": "directory", "size": 4096}, {"name": "Cargo.toml", "type": "file", "size": 1234}]}"#.to_string(),
                enabled: true,
                category: "File System".to_string(),
                is_builtin: false,
            },

            // Read Tool
            FunctionTool {
                name: "Read".to_string(),
                description: "Reads a file from the local filesystem. You can access any file directly by using this tool.\nAssume this tool is able to read all files on the machine. If the User provides a path to a file assume that path is valid. It is okay to read a file that does not exist; an error will be returned.\n\nUsage:\n- The file_path parameter must be an absolute path, not a relative path\n- By default, it reads up to 2000 lines starting from the beginning of the file\n- You can optionally specify an offset to start reading from a specific line number\n- You can optionally specify a limit to read only a certain number of lines".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "The absolute path to the file to read"
                        },
                        "offset": {
                            "type": "number",
                            "description": "The line number to start reading from. Only provide if the file is too large to read at once"
                        },
                        "limit": {
                            "type": "number",
                            "description": "The number of lines to read. Only provide if the file is too large to read at once."
                        }
                    },
                    "required": ["file_path"],
                    "additionalProperties": false,
                    "$schema": "http://json-schema.org/draft-07/schema#"
                }),
                mock_response: r#"{"content": "use std::collections::HashMap;\n\nfn main() {\n    println!(\"Hello, world!\");\n}", "lines": 4, "truncated": false}"#.to_string(),
                enabled: true,
                category: "File System".to_string(),
                is_builtin: false,
            },

            // Edit Tool
            FunctionTool {
                name: "Edit".to_string(),
                description: "Performs exact string replacements in files. \n\nUsage:\n- You must use your `Read` tool at least once in the conversation before editing. This tool will error if you attempt an edit without reading the file. \n- When editing text from Read tool output, ensure you preserve the exact indentation (tabs/spaces) as it appears AFTER the line number prefix. The line number prefix format is: spaces + line number + tab. Everything after that tab is the actual file content to match against.\n- Always verify the exact string you want to replace by copying it directly from the Read tool output\n- Be careful with whitespace - trailing spaces, leading spaces, and newlines must match exactly".to_string(),
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
                            "description": "The text to replace it with (must be different from old_string)"
                        },
                        "replace_all": {
                            "type": "boolean",
                            "default": false,
                            "description": "Replace all occurences of old_string (default false)"
                        }
                    },
                    "required": ["file_path", "old_string", "new_string"],
                    "additionalProperties": false,
                    "$schema": "http://json-schema.org/draft-07/schema#"
                }),
                mock_response: r#"{"success": true, "replacements": 1, "file": "/path/to/file.rs"}"#.to_string(),
                enabled: true,
                category: "File System".to_string(),
                is_builtin: false,
            },

            // Write Tool
            FunctionTool {
                name: "Write".to_string(),
                description: "Writes a file to the local filesystem.\n\nUsage:\n- This tool will overwrite the existing file if there is one at the provided path.\n- If this is an existing file, you MUST use the Read tool first to read the file's contents. This tool will fail if you did not read the file first.\n- ALWAYS prefer editing existing files in the codebase. NEVER write new files unless explicitly required.\n- NEVER proactively create documentation files (*.md) or README files. Only create documentation when explicitly asked to do so by the user.\n- When writing code files, ensure proper formatting and follow the existing code style in the project".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "The absolute path to the file to write (must be absolute, not relative)"
                        },
                        "content": {
                            "type": "string",
                            "description": "The content to write to the file"
                        }
                    },
                    "required": ["file_path", "content"],
                    "additionalProperties": false,
                    "$schema": "http://json-schema.org/draft-07/schema#"
                }),
                mock_response: r#"{"success": true, "bytes_written": 1234, "file": "/path/to/file.rs"}"#.to_string(),
                enabled: true,
                category: "File System".to_string(),
                is_builtin: false,
            },

            // MultiEdit Tool
            FunctionTool {
                name: "MultiEdit".to_string(),
                description: "This is a tool for making multiple edits to a single file in one operation. It is built on top of the Edit tool and allows you to perform multiple find-and-replace operations efficiently. Prefer this tool over the Edit tool when you need to make multiple edits to the same file.\n\nBefore using this tool:\n\n1. Use the Read tool to understand the file's contents and context\n2. Verify the directory path is correct\n\nTo make multiple file edits, provide the following:\n1. The file path\n2. An array of edit operations (old_string, new_string pairs)\n\nEach edit operation will be applied sequentially to the file.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_path": {
                            "type": "string",
                            "description": "The absolute path to the file to modify"
                        },
                        "edits": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
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
                                        "default": false,
                                        "description": "Replace all occurences of old_string (default false)."
                                    }
                                },
                                "required": ["old_string", "new_string"],
                                "additionalProperties": false
                            },
                            "minItems": 1,
                            "description": "Array of edit operations to perform sequentially on the file"
                        }
                    },
                    "required": ["file_path", "edits"],
                    "additionalProperties": false,
                    "$schema": "http://json-schema.org/draft-07/schema#"
                }),
                mock_response: r#"{"success": true, "total_edits": 3, "file": "/path/to/file.rs"}"#.to_string(),
                enabled: true,
                category: "File System".to_string(),
                is_builtin: false,
            },

            // ExitPlanMode Tool
            FunctionTool {
                name: "ExitPlanMode".to_string(),
                description: "Use this tool when you are in plan mode and have finished presenting your plan and are ready to code. This will prompt the user to exit plan mode. \nIMPORTANT: Only use this tool when the task requires planning the implementation steps of a task that requires writing code. For research tasks where you're gathering information, searching files, reading files or in general trying to understand the codebase - do NOT use this tool.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "plan": {
                            "type": "string",
                            "description": "The plan you came up with, that you want to run by the user for approval. Supports markdown. The plan should be pretty concise."
                        }
                    },
                    "required": ["plan"],
                    "additionalProperties": false,
                    "$schema": "http://json-schema.org/draft-07/schema#"
                }),
                mock_response: r#"{"status": "plan_submitted", "message": "Plan presented to user for approval"}"#.to_string(),
                enabled: false,
                category: "Planning".to_string(),
                is_builtin: false,
            },

            // TodoWrite Tool
            FunctionTool {
                name: "TodoWrite".to_string(),
                description: "Use this tool to create and manage a structured task list for your current coding session. This helps you track progress, organize complex tasks, and demonstrate thoroughness to the user.\nIt also helps the user understand the progress of the task and overall progress of their requests.\n\n## When to Use This Tool\nUse this tool proactively in these scenarios:\n\n1. Complex multi-step tasks - When a task requires 3 or more distinct steps or actions\n2. Non-trivial and complex requests that benefit from organized tracking".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "todos": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "content": {
                                        "type": "string",
                                        "minLength": 1
                                    },
                                    "status": {
                                        "type": "string",
                                        "enum": ["pending", "in_progress", "completed"]
                                    },
                                    "priority": {
                                        "type": "string",
                                        "enum": ["high", "medium", "low"]
                                    },
                                    "id": {
                                        "type": "string"
                                    }
                                },
                                "required": ["content", "status", "priority", "id"],
                                "additionalProperties": false
                            },
                            "description": "The updated todo list"
                        }
                    },
                    "required": ["todos"],
                    "additionalProperties": false,
                    "$schema": "http://json-schema.org/draft-07/schema#"
                }),
                mock_response: r#"{"status": "updated", "total_todos": 5, "completed": 2, "pending": 3}"#.to_string(),
                enabled: false,
                category: "Planning".to_string(),
                is_builtin: false,
            },

            // WebFetch Tool
            FunctionTool {
                name: "WebFetch".to_string(),
                description: "\n- Fetches content from a specified URL and processes it using an AI model\n- Takes a URL and a prompt as input\n- Fetches the URL content, converts HTML to markdown\n- Processes the content with the prompt using a small, fast model\n- Returns the model's response about the content\n- Use this tool when you need to retrieve and analyze web content\n\nUsage notes:\n  - IMPORTANT: If an MCP-provided web fetch tool is available, prefer using that tool instead of this one, as it may have better access and capabilities\n  - This tool is useful for analyzing web pages, documentation, articles, etc.\n  - The prompt should be specific about what information you want to extract or how you want the content processed".to_string(),
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
                    "required": ["url", "prompt"],
                    "additionalProperties": false,
                    "$schema": "http://json-schema.org/draft-07/schema#"
                }),
                mock_response: "{\"title\": \"Example Page\", \"content\": \"# Example\\n\\nThis is example content from the webpage.\", \"summary\": \"A webpage about examples\"}".to_string(),
                enabled: true,
                category: "Web".to_string(),
                is_builtin: false,
            },

            // WebSearch Tool
            FunctionTool {
                name: "WebSearch".to_string(),
                description: "\n- Allows Claude to search the web and use the results to inform responses\n- Provides up-to-date information for current events and recent data\n- Returns search result information formatted as search result blocks\n- Use this tool for accessing information beyond Claude's knowledge cutoff\n- Searches are performed automatically within a single API call\n\nUsage notes:\n  - Domain filtering is supported to include or block specific websites\n  - Web search is only available when explicitly enabled by the user\n  - Results are automatically integrated into the conversation context".to_string(),
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
                    "required": ["query"],
                    "additionalProperties": false,
                    "$schema": "http://json-schema.org/draft-07/schema#"
                }),
                mock_response: r#"{"results": [{"title": "Rust Programming Language", "url": "https://rust-lang.org", "snippet": "A systems programming language..."}], "total": 1}"#.to_string(),
                enabled: true,
                category: "Web".to_string(),
                is_builtin: false,
            },

            // NotebookEdit Tool
            FunctionTool {
                name: "NotebookEdit".to_string(),
                description: "Completely replaces the contents of a specific cell in a Jupyter notebook (.ipynb file) with new source. Jupyter notebooks are interactive documents that combine code, text, and visualizations, commonly used for data analysis and scientific computing. The notebook_path parameter must be an absolute path, not a relative path. The cell_number is 0-indexed. Use edit_mode=insert to add a new cell at the index specified by cell_number. Use edit_mode=delete to delete the cell at the specified index.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "notebook_path": {
                            "type": "string",
                            "description": "The absolute path to the Jupyter notebook file to edit (must be absolute, not relative)"
                        },
                        "cell_id": {
                            "type": "string",
                            "description": "The ID of the cell to edit. When inserting a new cell, the new cell will be inserted after the cell with this ID, or at the beginning if not specified."
                        },
                        "new_source": {
                            "type": "string",
                            "description": "The new source for the cell"
                        },
                        "cell_type": {
                            "type": "string",
                            "enum": ["code", "markdown"],
                            "description": "The type of the cell (code or markdown). If not specified, it defaults to the current cell type. If using edit_mode=insert, this is required."
                        },
                        "edit_mode": {
                            "type": "string",
                            "enum": ["replace", "insert", "delete"],
                            "description": "The type of edit to make (replace, insert, delete). Defaults to replace."
                        }
                    },
                    "required": ["notebook_path", "new_source"],
                    "additionalProperties": false,
                    "$schema": "http://json-schema.org/draft-07/schema#"
                }),
                mock_response: r#"{"success": true, "cell_id": "abc123", "edit_mode": "replace", "notebook": "/path/to/notebook.ipynb"}"#.to_string(),
                enabled: false,
                category: "IDE".to_string(),
                is_builtin: false,
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
                    "required": ["location"],
                    "additionalProperties": false,
                    "$schema": "http://json-schema.org/draft-07/schema#"
                }),
                mock_response: r#"{"temperature": 22, "condition": "sunny", "humidity": 65, "wind_speed": 5, "location": "San Francisco, CA"}"#.to_string(),
                enabled: true,
                category: "Weather".to_string(),
                is_builtin: false,
            },

            // IDE Diagnostics Tool
            FunctionTool {
                name: "mcp__ide__getDiagnostics".to_string(),
                description: "Get language diagnostics from VS Code".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "uri": {
                            "type": "string",
                            "description": "Optional file URI to get diagnostics for. If not provided, gets diagnostics for all files."
                        }
                    },
                    "additionalProperties": false,
                    "$schema": "http://json-schema.org/draft-07/schema#"
                }),
                mock_response: r#"{"diagnostics": [{"file": "src/main.rs", "line": 42, "severity": "error", "message": "cannot find value `x` in this scope"}]}"#.to_string(),
                enabled: false,
                category: "IDE".to_string(),
                is_builtin: false,
            },

            // Execute Code Tool
            FunctionTool {
                name: "mcp__ide__executeCode".to_string(),
                description: "Execute python code in the Jupyter kernel for the current notebook file.\n    \n    All code will be executed in the current Jupyter kernel.\n    \n    Avoid declaring variables or modifying the state of the kernel unless the user\n    explicitly asks for it.\n    \n    Any code executed will persist across calls to this tool, unless the kernel\n    has been restarted.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "code": {
                            "type": "string",
                            "description": "The code to be executed on the kernel."
                        }
                    },
                    "required": ["code"],
                    "additionalProperties": false,
                    "$schema": "http://json-schema.org/draft-07/schema#"
                }),
                mock_response: r#"{"output": "Hello, World!\n", "execution_count": 1, "status": "ok"}"#.to_string(),
                enabled: false,
                category: "IDE".to_string(),
                is_builtin: false,
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