# llm-playground-rs

A modern Rust-based LLM playground built with Yew framework for WebAssembly. This project provides a sophisticated web interface for testing and experimenting with various Large Language Model APIs, featuring a flexible provider system, advanced function tools, and MCP (Model Context Protocol) integration.

## 🚀 Features

### Core Capabilities
- **Flexible Provider System**: Support for OpenAI, Gemini, OpenRouter, Ollama, and custom providers
- **Real-time Streaming**: Interactive conversation UI with streaming responses
- **Advanced Function Tools**: Comprehensive built-in tools (HTTP, file system, search, agents) and custom function calling
- **MCP Integration**: Model Context Protocol support for enhanced AI capabilities
- **Session Management**: Persistent chat sessions with local storage
- **Visual Function Editor**: Interactive interface for creating and editing function tools
- **Multi-Modal Support**: Support for different content types and structured outputs

### Built-in Function Tools
- **HTTP Tools**: Fetch API for web requests
- **File System**: Read, write, edit files and directories
- **Search Tools**: Grep, glob pattern matching
- **System Tools**: Bash command execution
- **Agent Tools**: Launch specialized sub-agents for complex tasks
- **Web Tools**: Web search and content fetching
- **Weather API**: Real-time weather information
- **Planning Tools**: TODO management and task planning

## 🏗️ Architecture Overview

### Project Structure
```
src/
├── main.rs                          # Application entry point
└── llm_playground/
    ├── mod.rs                       # Module exports and core logic
    ├── types.rs                     # Core type definitions
    ├── flexible_playground.rs       # Main playground component
    ├── flexible_client.rs          # Flexible LLM client implementation
    ├── provider_config.rs          # Provider configuration system
    ├── storage.rs                   # Browser storage utilities
    ├── builtin_tools.rs            # Built-in function tool implementations
    ├── mcp_client.rs               # Model Context Protocol client
    ├── migration.rs                # Configuration migration utilities
    ├── api_clients/                # LLM provider implementations
    │   ├── mod.rs
    │   ├── traits.rs               # Common API traits
    │   ├── openai_client.rs        # OpenAI API client
    │   └── gemini_client.rs        # Gemini API client
    ├── components/                 # UI components
    │   ├── mod.rs
    │   ├── chat_header.rs         # Chat session header
    │   ├── chatroom.rs            # Main chat interface
    │   ├── chat_room.rs           # Alternative chat room component
    │   ├── sidebar.rs             # Settings sidebar
    │   ├── flexible_settings_panel.rs  # Flexible configuration panel
    │   ├── settings_panel.rs      # Legacy settings panel
    │   ├── mcp_settings_panel.rs  # MCP configuration
    │   ├── model_selector.rs      # Model selection dropdown
    │   ├── message_bubble.rs      # Individual message display
    │   ├── input_bar.rs           # Message input component
    │   ├── function_tool_editor.rs     # Function tool editor
    │   ├── visual_function_tool_editor.rs  # Visual tool editor
    │   ├── function_call_handler.rs    # Function execution handler
    │   └── notification.rs        # Notification system
    ├── hooks/                     # Custom Yew hooks
    │   ├── mod.rs
    │   └── use_llm_chat.rs        # LLM communication hook
    └── examples/                  # Usage examples
        └── mcp_integration_example.rs
```

## 🛠️ Quick Start

### Prerequisites

- **Rust** (latest stable version)
- **Trunk** for WebAssembly development and serving
- A modern web browser with WebAssembly support

### Installation & Setup

1. **Clone the repository:**
```bash
git clone https://github.com/yourusername/llm-playground-rs.git
cd llm-playground-rs
```

2. **Install dependencies:**
```bash
# Install trunk for serving (recommended)
cargo install trunk

# Add WebAssembly target
rustup target add wasm32-unknown-unknown
```

3. **Development server:**
```bash
# Start development server with hot reload
trunk serve

# Or specify a custom port
trunk serve --port 8080
```

4. **Production build:**
```bash
trunk build --release
```

### Quick Setup Commands
```bash
# Alternative manual setup (if needed)
wget https://github.com/trunk-rs/trunk/releases/download/v0.21.14/trunk-x86_64-unknown-linux-gnu.tar.gz
tar -xvf trunk-x86_64-unknown-linux-gnu.tar.gz
./trunk serve
```

### Configuration

1. **Provider Setup**: Configure your LLM providers in the settings panel:
   - **OpenAI**: Add API key from https://platform.openai.com/api-keys
   - **Gemini**: Get API key from https://makersuite.google.com/app/apikey
   - **OpenRouter**: Register at https://openrouter.ai/ for access to multiple models
   - **Ollama**: Install locally for offline model access

2. **Function Tools**: Enable built-in tools or create custom ones using the visual editor

3. **MCP Integration**: Configure Model Context Protocol servers for enhanced capabilities

## 🎯 Core Components

### Main Application (`FlexibleLLMPlayground`)
The central component managing application state, provider configuration, and coordination between UI elements.

**Key Features:**
- Session management with persistent storage
- Flexible provider switching
- Function tool execution coordination
- MCP client integration
- Real-time streaming support

### API Clients (`api_clients/`)

**Unified Client Interface (`traits.rs`)**
```rust
pub trait LLMClient {
    fn send_message(...) -> Future<Output = Result<LLMResponse, String>>;
    fn send_message_stream(...) -> Future<Output = Result<(), String>>;
    fn get_available_models(...) -> Future<Output = Result<Vec<String>, String>>;
}
```

**Provider Implementations:**
- `OpenAIClient`: OpenAI API integration with function calling
- `GeminiClient`: Google Gemini API with streaming support
- Extensible architecture for additional providers

### Flexible Provider System (`provider_config.rs`)

**Multi-Provider Configuration:**
```rust
pub struct FlexibleApiConfig {
    pub providers: Vec<ProviderConfig>,     // Multiple provider configs
    pub router: RouterConfig,               // Provider routing rules
    pub function_tools: Vec<FunctionTool>,  // Available function tools
    pub mcp_config: McpConfig,             // MCP integration settings
}
```

**Provider Types:**
- OpenRouter (multiple models, free tier available)
- Gemini (Google's models with OpenAI-compatible endpoint)
- OpenAI (official API)
- Ollama (local models)
- Custom providers via configuration

### Function Tools System (`builtin_tools.rs`)

**Built-in Categories:**
- **HTTP**: Web requests and API calls
- **File System**: File and directory operations
- **Search**: Content and pattern matching
- **System**: Command execution
- **Agent**: Sub-agent task delegation
- **Web**: Web search and content fetching
- **Weather**: Real-time weather data
- **Planning**: Task and TODO management
- **IDE**: Development environment integration

**Tool Execution:**
```rust
pub async fn execute_builtin_tool(
    tool_name: &str,
    arguments: &Value,
    mcp_client: Option<&McpClient>,
) -> Result<Value, String>
```

### UI Components (`components/`)

**Chat Interface:**
- `Chatroom`: Main conversation display
- `MessageBubble`: Individual message rendering with markdown support
- `InputBar`: Message composition with auto-resize
- `FunctionCallHandler`: Function execution visualization

**Configuration:**
- `FlexibleSettingsPanel`: Provider and model configuration
- `ModelSelector`: Dynamic model selection
- `VisualFunctionToolEditor`: Interactive tool creation
- `McpSettingsPanel`: MCP server configuration

**Session Management:**
- `ChatHeader`: Session info and controls
- `Sidebar`: Session list and navigation
- `Notification`: System alerts and feedback

### MCP Integration (`mcp_client.rs`)

Model Context Protocol support for enhanced AI capabilities:
- Server connection management
- Tool discovery and registration
- Resource access coordination
- Protocol compliance and error handling

## 🔧 Development Guide

### Building

```bash
# Development build with debug info
trunk serve --port 8000

# Production optimized build
trunk build --release

# Manual WebAssembly build
wasm-pack build --target web --dev
```

### Testing

```bash
# Run Rust unit tests
cargo test

# Run with specific features
cargo test --features "test-mode"

# Browser testing (requires wasm-pack-test)
wasm-pack test --headless --firefox
```

### Adding New Providers

1. **Implement the `LLMClient` trait:**
```rust
pub struct CustomClient;

impl LLMClient for CustomClient {
    // Implement required methods
}
```

2. **Add provider configuration:**
```rust
ProviderConfig {
    name: "custom".to_string(),
    api_base_url: "https://api.custom.com/v1".to_string(),
    api_key: String::new(),
    models: vec!["custom-model".to_string()],
    transformer: TransformerConfig {
        r#use: vec!["openai".to_string()], // or "gemini"
    },
}
```

3. **Register in the flexible client:**
Update `FlexibleLLMClient` to handle your new provider.

### Creating Custom Function Tools

**Via Code:**
```rust
FunctionTool {
    name: "my_tool".to_string(),
    description: "Description of what the tool does".to_string(),
    parameters: serde_json::json!({
        "type": "object",
        "properties": {
            "input": {
                "type": "string",
                "description": "Input parameter"
            }
        },
        "required": ["input"]
    }),
    mock_response: r#"{"result": "success"}"#.to_string(),
    enabled: true,
    category: "Custom".to_string(),
    is_builtin: false,
}
```

**Via Visual Editor:**
Use the integrated visual function tool editor for interactive creation and testing.

## 📋 Type System

### Core Types (`types.rs`)

**Message Structure:**
```rust
pub struct Message {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: f64,
    pub function_call: Option<serde_json::Value>,
    pub function_response: Option<serde_json::Value>,
}
```

**Configuration:**
```rust
pub struct ApiConfig {
    pub gemini: GeminiConfig,
    pub openai: OpenAIConfig,
    pub current_provider: ApiProvider,
    pub shared_settings: SharedSettings,
    pub function_tools: Vec<FunctionTool>,
    pub mcp_config: McpConfig,
}
```

**Function Tools:**
```rust
pub struct FunctionTool {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
    pub mock_response: String,
    pub enabled: bool,
    pub category: String,
    pub is_builtin: bool,
}
```

## 🔍 Key Features Deep Dive

### Streaming Support
Real-time response streaming with token-by-token display and function call handling during streaming.

### Session Persistence
Automatic saving of chat sessions, provider configurations, and user preferences to browser localStorage.

### Error Handling & Retry Logic
Intelligent retry mechanisms for rate limits and network errors with exponential backoff.

### Function Call Visualization
Rich display of function calls, parameters, and responses with syntax highlighting and collapsible views.

### MCP Protocol Integration
Full support for Model Context Protocol servers, enabling advanced AI capabilities and resource access.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes with proper documentation
4. Add tests for new functionality
5. Ensure all tests pass (`cargo test`)
6. Submit a pull request with a clear description

### Development Guidelines
- Follow Rust naming conventions and best practices
- Add comprehensive documentation for public APIs
- Include unit tests for new functionality
- Use the existing component patterns for UI consistency
- Update this README for significant architectural changes

## 🔗 References

### Development Tools & Dependencies
- **Yew Framework**: https://yew.rs/
- **Trunk**: https://trunkrs.dev/
- **WebAssembly**: https://webassembly.org/

### Referenced Projects
- **MCP Rust SDK**: https://github.com/modelcontextprotocol/rust-sdk
- **Yew Demo Examples**: https://github.com/jyasuu/yew-demo
- **Chat CLI Reference**: https://github.com/jyasuu/chat-cli

### API Documentation
- **OpenAI API**: https://platform.openai.com/docs
- **Gemini API**: https://developers.generativeai.google/
- **OpenRouter**: https://openrouter.ai/docs
- **Model Context Protocol**: https://modelcontextprotocol.io/
