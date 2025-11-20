# AGENTS.md - Development Guidelines

## 🏗️ SOLID Principles Guidelines for LLM Playground

This document provides guidelines for maintaining and improving code quality following SOLID principles. All developers and AI agents working on this project should follow these patterns.

---

## 📋 Current Architecture Issues

> **⚠️ Important**: Before making any changes, review [SOLID_VIOLATIONS_ANALYSIS.md](./SOLID_VIOLATIONS_ANALYSIS.md) for current code issues.

### Quick Issue Summary:
- **Configuration classes** have too many responsibilities
- **Main playground component** is a "god object"
- **Provider system** uses hardcoded logic instead of extensible patterns
- **Interfaces** are too large and force unnecessary implementations
- **Dependencies** are tightly coupled with concrete implementations

---

## 🎯 SOLID Principles Application

### 1. Single Responsibility Principle (SRP)

#### ✅ **DO: Split Large Classes**

```rust
// ❌ BEFORE: Too many responsibilities
struct ApiConfig {
    pub gemini: GeminiConfig,
    pub openai: OpenAIConfig,
    pub function_tools: Vec<FunctionTool>,
    pub mcp_config: McpConfig,
    // ... 20+ fields and methods
}

// ✅ AFTER: Separate concerns
struct ProviderConfigs {
    pub gemini: GeminiConfig,
    pub openai: OpenAIConfig,
}

struct FunctionToolRegistry {
    tools: Vec<FunctionTool>,
}

struct McpConfiguration {
    config: McpConfig,
}

struct PlaygroundSettings {
    pub temperature: f32,
    pub max_tokens: u32,
    pub system_prompt: String,
}
```

#### ✅ **DO: Extract Services from Components**

```rust
// ❌ BEFORE: Component doing everything
#[function_component(FlexibleLLMPlayground)]
pub fn flexible_llm_playground() -> Html {
    // State management
    // Storage operations  
    // API calls
    // MCP initialization
    // UI rendering
    // Event handling
}

// ✅ AFTER: Separate services
pub struct SessionService;
pub struct StorageService;
pub struct McpService;
pub struct ProviderService;

#[function_component(FlexibleLLMPlayground)]
pub fn flexible_llm_playground() -> Html {
    // Only UI rendering and coordination
}
```

### 2. Open/Closed Principle (OCP)

#### ✅ **DO: Use Trait-based Extension**

```rust
// ❌ BEFORE: Hardcoded provider selection
fn get_client_for_provider(&self, provider: &ProviderConfig) -> Box<dyn LLMClient> {
    if provider.transformer.r#use.contains(&"gemini".to_string()) {
        Box::new(GeminiClient::new())
    } else {
        Box::new(OpenAIClient::new())
    }
}

// ✅ AFTER: Registry-based extension
pub trait ProviderFactory {
    fn create_client(&self, config: &ProviderConfig) -> Result<Box<dyn LLMClient>, String>;
    fn supports_provider(&self, provider_type: &str) -> bool;
}

pub struct ProviderRegistry {
    factories: Vec<Box<dyn ProviderFactory>>,
}

impl ProviderRegistry {
    pub fn register_factory(&mut self, factory: Box<dyn ProviderFactory>) {
        self.factories.push(factory);
    }
    
    pub fn create_client(&self, config: &ProviderConfig) -> Result<Box<dyn LLMClient>, String> {
        for factory in &self.factories {
            if factory.supports_provider(&config.provider_type) {
                return factory.create_client(config);
            }
        }
        Err("Unsupported provider type".to_string())
    }
}
```

#### ✅ **DO: Plugin-based Function Tools**

```rust
// ❌ BEFORE: Hardcoded function tools
fn get_default_function_tools() -> Vec<FunctionTool> {
    vec![
        FunctionTool { name: "fetch".to_string(), /* ... */ },
        FunctionTool { name: "bash".to_string(), /* ... */ },
        // ... 20+ hardcoded tools
    ]
}

// ✅ AFTER: Registry pattern
pub trait FunctionToolPlugin {
    fn get_tools(&self) -> Vec<FunctionTool>;
    fn execute(&self, tool_name: &str, args: &Value) -> Result<Value, String>;
}

pub struct FunctionToolRegistry {
    plugins: Vec<Box<dyn FunctionToolPlugin>>,
}

impl FunctionToolRegistry {
    pub fn register_plugin(&mut self, plugin: Box<dyn FunctionToolPlugin>) {
        self.plugins.push(plugin);
    }
}
```

### 3. Liskov Substitution Principle (LSP)

#### ✅ **DO: Ensure Contract Compliance**

```rust
// ✅ All implementations must behave consistently
pub trait LLMClient {
    /// Send message and return response
    /// 
    /// # Guarantees:
    /// - Returns Ok(response) on successful API call
    /// - Returns Err(msg) on network/API errors
    /// - Never panics under normal conditions
    /// - Respects timeout configurations
    fn send_message(&self, ...) -> Pin<Box<dyn Future<Output = Result<LLMResponse, String>>>>;
}

// ✅ Both implementations follow the same contract
impl LLMClient for OpenAIClient {
    fn send_message(&self, ...) -> Pin<Box<dyn Future<Output = Result<LLMResponse, String>>>> {
        // Consistent error handling
        // Proper timeout handling
        // Standard response format
    }
}

impl LLMClient for GeminiClient {
    fn send_message(&self, ...) -> Pin<Box<dyn Future<Output = Result<LLMResponse, String>>>> {
        // Same behavior guarantees as OpenAI client
        // Consistent error format
        // Same timeout handling
    }
}
```

### 4. Interface Segregation Principle (ISP)

#### ✅ **DO: Split Large Interfaces**

```rust
// ❌ BEFORE: Fat interface
pub trait LLMClient {
    fn send_message(...) -> Future<...>;
    fn send_message_stream(...) -> Future<...>;
    fn get_available_models(...) -> Future<...>;
    fn convert_legacy_messages(...) -> Vec<...>;
    // More methods...
}

// ✅ AFTER: Segregated interfaces
pub trait MessageSender {
    fn send_message(...) -> Future<...>;
}

pub trait StreamingSender {
    fn send_message_stream(...) -> Future<...>;
}

pub trait ModelProvider {
    fn get_available_models(...) -> Future<...>;
}

pub trait MessageConverter {
    fn convert_legacy_messages(...) -> Vec<...>;
}

// Compose as needed
pub trait FullLLMClient: MessageSender + StreamingSender + ModelProvider {}
```

### 5. Dependency Inversion Principle (DIP)

#### ✅ **DO: Use Dependency Injection**

```rust
// ❌ BEFORE: Direct dependencies
pub struct FlexibleLLMClient;

impl FlexibleLLMClient {
    fn get_client_for_provider(&self, provider: &ProviderConfig) -> Box<dyn LLMClient> {
        Box::new(GeminiClient::new()) // Direct instantiation
    }
}

// ✅ AFTER: Dependency injection
pub struct FlexibleLLMClient {
    provider_registry: Arc<dyn ProviderRegistry>,
    storage_service: Arc<dyn StorageService>,
}

impl FlexibleLLMClient {
    pub fn new(
        provider_registry: Arc<dyn ProviderRegistry>,
        storage_service: Arc<dyn StorageService>,
    ) -> Self {
        Self {
            provider_registry,
            storage_service,
        }
    }
}
```

---

## 🛠️ Refactoring Patterns

### Pattern 1: Extract Service Classes

**When to use**: Component/struct has multiple responsibilities

```rust
// Step 1: Identify responsibilities
// - Data storage/loading
// - API communication  
// - State management
// - UI rendering

// Step 2: Extract services
pub struct StorageService {
    storage: Box<dyn Storage>,
}

pub struct ApiService {
    client: Box<dyn LLMClient>,
}

// Step 3: Inject into component
#[derive(Properties, PartialEq)]
pub struct PlaygroundProps {
    pub storage_service: StorageService,
    pub api_service: ApiService,
}
```

### Pattern 2: Registry Pattern for Extensions

**When to use**: Adding new types requires modifying existing code

```rust
// Step 1: Define plugin interface
pub trait ProviderPlugin {
    fn provider_type(&self) -> &str;
    fn create_client(&self, config: &ProviderConfig) -> Result<Box<dyn LLMClient>, String>;
}

// Step 2: Create registry
pub struct ProviderRegistry {
    plugins: HashMap<String, Box<dyn ProviderPlugin>>,
}

// Step 3: Register plugins at startup
fn initialize_providers(registry: &mut ProviderRegistry) {
    registry.register(Box::new(OpenAIProviderPlugin));
    registry.register(Box::new(GeminiProviderPlugin));
    // Add new providers without modifying core code
}
```

### Pattern 3: Builder Pattern for Complex Configuration

**When to use**: Objects have many optional parameters

```rust
// ✅ AFTER: Builder pattern
pub struct ApiConfigBuilder {
    provider_configs: Option<ProviderConfigs>,
    function_tools: Option<FunctionToolRegistry>,
    mcp_config: Option<McpConfiguration>,
    settings: Option<PlaygroundSettings>,
}

impl ApiConfigBuilder {
    pub fn new() -> Self { /* ... */ }
    
    pub fn with_providers(mut self, providers: ProviderConfigs) -> Self {
        self.provider_configs = Some(providers);
        self
    }
    
    pub fn with_function_tools(mut self, tools: FunctionToolRegistry) -> Self {
        self.function_tools = Some(tools);
        self
    }
    
    pub fn build(self) -> Result<ApiConfig, String> { /* ... */ }
}
```

---

## 📏 Code Quality Rules

### File Size Limits
- **Components**: Max 300 lines
- **Service classes**: Max 200 lines  
- **Structs**: Max 10 fields, Max 15 methods
- **Functions**: Max 50 lines

### Responsibility Guidelines
- **Each struct** should have only one reason to change
- **Each trait** should have a single, cohesive purpose
- **Each component** should only handle UI rendering and user interaction

### Dependency Rules
- **Never** instantiate concrete types directly (use factories/DI)
- **Always** depend on abstractions, not concretions
- **Avoid** circular dependencies

### Testing Requirements
- **Each service** must be unit testable in isolation
- **Mock all external dependencies** in tests
- **Component tests** should not require network calls

---

## 🔧 Refactoring Checklist

### Before Making Changes
- [ ] Read the SOLID violations analysis
- [ ] Identify which principles are violated
- [ ] Plan the refactoring approach
- [ ] Write tests for current behavior

### During Refactoring
- [ ] Extract services from large components
- [ ] Split large interfaces into focused ones
- [ ] Replace hardcoded logic with registry patterns
- [ ] Use dependency injection for external dependencies
- [ ] Ensure each class has single responsibility

### After Refactoring
- [ ] Verify all tests pass
- [ ] Check file sizes are within limits
- [ ] Validate dependencies flow correctly (DIP)
- [ ] Ensure new functionality can be added without modifying existing code (OCP)

---

## 📚 Implementation Examples

### Example 1: Session Management Refactor

```rust
// BEFORE: Mixed in main component
#[function_component(FlexibleLLMPlayground)]
pub fn flexible_llm_playground() -> Html {
    let sessions = use_state(|| HashMap::<String, ChatSession>::new());
    // Load from storage
    // Save to storage  
    // Manage current session
    // UI rendering
}

// AFTER: Extracted service
pub struct SessionService {
    storage: Arc<dyn StorageService>,
    sessions: HashMap<String, ChatSession>,
}

impl SessionService {
    pub fn load_sessions(&mut self) -> Result<(), String> { /* ... */ }
    pub fn save_session(&self, session: &ChatSession) -> Result<(), String> { /* ... */ }
    pub fn create_session(&mut self, title: String) -> String { /* ... */ }
    pub fn delete_session(&mut self, id: &str) -> Result<(), String> { /* ... */ }
}

// Component only handles UI
#[derive(Properties, PartialEq)]
pub struct PlaygroundProps {
    pub session_service: UseStateHandle<SessionService>,
}
```

### Example 2: Provider System Refactor

```rust
// AFTER: Extensible provider system
pub trait ProviderFactory {
    fn supports(&self, provider_type: &str) -> bool;
    fn create(&self, config: &ProviderConfig) -> Result<Box<dyn MessageSender>, String>;
}

pub struct OpenAIProviderFactory;

impl ProviderFactory for OpenAIProviderFactory {
    fn supports(&self, provider_type: &str) -> bool {
        provider_type == "openai"
    }
    
    fn create(&self, config: &ProviderConfig) -> Result<Box<dyn MessageSender>, String> {
        Ok(Box::new(OpenAIClient::new(config.api_key.clone())))
    }
}

// Easy to add new providers
pub struct AnthropicProviderFactory;
impl ProviderFactory for AnthropicProviderFactory {
    fn supports(&self, provider_type: &str) -> bool {
        provider_type == "anthropic"
    }
    
    fn create(&self, config: &ProviderConfig) -> Result<Box<dyn MessageSender>, String> {
        Ok(Box::new(AnthropicClient::new(config.api_key.clone())))
    }
}
```

---

## 🚨 Anti-Patterns to Avoid

### ❌ God Objects
```rust
// DON'T: Single struct doing everything
struct MegaConfig {
    pub providers: Vec<ProviderConfig>,
    pub tools: Vec<FunctionTool>,
    pub sessions: HashMap<String, ChatSession>,
    pub ui_state: UIState,
    pub storage: LocalStorage,
    pub api_clients: Vec<Box<dyn LLMClient>>,
    // ... 50+ fields
}
```

### ❌ Hardcoded Dependencies  
```rust
// DON'T: Direct instantiation
impl MyService {
    fn new() -> Self {
        Self {
            storage: LocalStorage::new(), // Hardcoded!
            client: OpenAIClient::new(),   // Hardcoded!
        }
    }
}
```

### ❌ Large Interfaces
```rust
// DON'T: Interface with too many methods
pub trait EverythingService {
    fn handle_storage(&self);
    fn send_messages(&self);
    fn manage_sessions(&self);
    fn render_ui(&self);
    fn validate_config(&self);
    // ... 20+ methods
}
```

### ❌ Violation of Encapsulation
```rust
// DON'T: Exposing internal structure
pub struct BadConfig {
    pub internal_state: HashMap<String, Value>, // Too exposed!
}
```

---

## 🎯 Success Metrics

### Code Quality Indicators
- **Average file size** < 250 lines
- **Average method length** < 20 lines
- **Cyclomatic complexity** < 10 per method
- **Dependency depth** < 4 levels

### Architecture Quality
- **Testability**: Can test each service in isolation
- **Extensibility**: Can add new providers without changing core code
- **Maintainability**: Changes have minimal ripple effects
- **Readability**: Clear separation of concerns

### Refactoring Progress
- [ ] Configuration split into focused classes
- [ ] Provider system uses registry pattern  
- [ ] Main component < 300 lines
- [ ] All external dependencies injected
- [ ] Interfaces have < 5 methods each

---

*This document is a living guideline. Update it as the architecture evolves and new patterns emerge.*