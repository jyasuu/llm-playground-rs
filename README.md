# llm-playground-rs


# original

# help me generate a application specification

## develop tools

it is a only frontend web application . no server side.
any data storage requirement should consider use browser local storage.

1. rust
2. yew framework
3. html + tailwindcss



## here is what I think.

1. it  a llm api playgound
2. it support gemini api and open api compatible
3. there is some ui components features
    1. chat room
    2. session history and switch
    3. llm configuration 
        1. gemini key and models
        2. openapi url and token and models
        3. temperature
        4. retry and rate limit delay
        5. system prompts define
        6. function call tools define(just a  mock define ,so when llm decide to use it just display it in chatroom and return a configurable value to llm)
        7. structure output define(just a  mock define ,so when llm decide to use it just display it in chatroom )
        8. sse mcp server

## advice me others features


# specification

Got it ✅ You’re designing a **frontend-only LLM API playground** using **Rust + Yew + TailwindCSS**, with **local storage** as the persistence layer.

Here’s a **refined application specification** with your listed features plus additional ideas that would make the playground powerful, flexible, and easy to use:

---

# Application Specification: LLM Playground (Frontend Only)

## Development Tools

* **Rust**: core language
* **Yew**: frontend web framework
* **TailwindCSS**: styling and layout
* **Local Storage**: persistence for configs, sessions, and history

---

## Core Features

### 1. **Chat Interface**

* Multi-turn chat UI (assistant + user bubbles).
* Support markdown rendering (code blocks, tables, links).
* Support streaming responses (if API provides SSE or chunked responses).
* Message retry + edit & resend.
* Inline display of structured outputs and mock tool calls.

---

### 2. **Session Management**

* Create, switch, and delete chat sessions.
* Persist session history in local storage.
* Search/filter across past sessions.
* Export/import sessions (JSON file).
* Pin sessions for quick access.

---

### 3. **LLM Configuration Panel**

* **Gemini API**

  * Key input
  * Model selection
* **OpenAI-compatible API**

  * Base URL
  * Token
  * Model selection
* **Shared settings**

  * Temperature slider
  * Max tokens
  * Retry & rate-limit delay
  * System prompt editor
  * Function call mock definitions
  * Structured output mock definitions
  * SSE MCP server config

---

## Extended Features (Recommendations)

### 4. **UI Enhancements**

* **Tabs for multiple models** in a single session (compare responses).
* **Drag & drop reorder** messages.
* **Collapsible messages** (useful for very long outputs).
* **Syntax highlighting** for code snippets.
* **Copy-to-clipboard** buttons for outputs.
* **Dark/Light theme toggle**.

---

### 5. **Mock Tools / Function Calling**

* Define a list of tools with:

  * name, description, expected input/output format
  * mock return values (static JSON, random values, or user-provided input)
* When LLM “calls” the tool:

  * Show tool invocation in chat stream
  * Return the configured mock response to continue conversation

---

### 6. **Structured Output Preview**

* Define output schema (JSON structure).
* When LLM responds with structured data, parse & render as:

  * JSON viewer
  * Table or form (if schema is simple enough)

---

### 7. **Multi-Model Playground**

* Side-by-side comparison of outputs from:

  * Gemini model vs. OpenAI-compatible model
  * Different configurations (temperature, system prompts)

---

### 8. **Data Persistence & Portability**

* Local Storage for:

  * sessions
  * configs
  * tool definitions
* Import/Export:

  * full config
  * individual sessions
* Optionally, allow **IndexedDB** for larger histories.

---

### 9. **Developer Features**

* **Request/Response Inspector**:

  * Show raw request JSON sent to API.
  * Show raw API response (before formatting).
* **Rate Limit Monitor**:

  * Track request counts & delay injection logs.
* **Error Handling UI**:

  * Display API errors in a readable format.

---

### 10. **Security Considerations**

* Warn user: API keys stored only in local storage (not secure for shared browsers).
* Option to **mask keys** in UI until revealed.

---

### 11. **Future Expandability**

* Support additional LLM APIs (Anthropic, Ollama, local inference endpoints).
* Add **plugin system** so users can define new tool mocks easily.
* Provide **hotkeys** for power users (send, new session, switch).

---

## Suggested Feature Roadmap

* **MVP**: Chat room, session storage, config (API + key + model), system prompt, temperature.
* **Phase 2**: Function calling mocks, structured outputs, export/import.
* **Phase 3**: Multi-model comparison, raw inspector, session search.
* **Phase 4**: Theming, extensibility, plugin system.

---

Perfect 🚀 Let’s map out a **Yew component architecture diagram** for your **LLM Playground**.
This will show **component hierarchy + responsibilities + local storage interaction**.

---

# Yew Component Architecture

```
App
│
├── Navbar
│    ├── Title / Logo
│    ├── ThemeToggle
│    └── ConfigButton (open/close LLMConfigPanel)
│
├── Sidebar (SessionManager)
│    ├── SessionList
│    │    ├── SessionItem (select / delete / pin)
│    ├── NewSessionButton
│    └── SearchBox
│
├── MainArea
│    ├── ChatRoom
│    │    ├── MessageList
│    │    │    ├── MessageBubble (user / assistant)
│    │    │    │    ├── MarkdownRenderer
│    │    │    │    ├── CodeBlock (syntax highlighting + copy)
│    │    │    │    └── ToolCallView / StructuredOutputView
│    │    └── InputBar
│    │         ├── TextArea
│    │         ├── SendButton
│    │         └── Edit & Resend / Retry
│    │
│    └── MultiModelCompare (optional future)
│
└── FloatingPanels
     ├── LLMConfigPanel
     │    ├── ApiConfig (Gemini + OpenAI-compatible)
     │    ├── SharedSettings (temperature, retry, tokens)
     │    ├── SystemPromptEditor
     │    ├── FunctionCallMockEditor
     │    └── StructuredOutputMockEditor
     │
     ├── ImportExportPanel
     └── InspectorPanel (raw API request/response)
```

---

# Component Responsibilities

### **App**

* Root state manager (sessions, configs, UI state).
* LocalStorage load/save.
* Route state to children.

### **Navbar**

* Global actions: theme toggle, open config.
* Displays app title.

### **Sidebar / SessionManager**

* Manages session list.
* CRUD sessions (create, rename, delete).
* Search & pinning.
* Persist session metadata in local storage.

### **ChatRoom**

* Displays current session chat.
* Sends API requests via fetch/SSE.
* Handles streaming & retries.

### **MessageList / MessageBubble**

* Renders conversation history.
* Supports markdown + code highlighting.
* Detects special message types:

  * **Tool Call** → show as `ToolCallView`.
  * **Structured Output** → show as `StructuredOutputView`.

### **InputBar**

* User input area.
* Multi-line editing.
* Buttons for send, retry, edit/resend.

### **LLMConfigPanel**

* Manage API endpoints + tokens + models.
* Adjust generation params (temp, tokens).
* Define system prompts.
* Define mock tool calls + mock structured outputs.
* Save to local storage.

### **InspectorPanel**

* Show raw API requests/responses for debugging.
* Optional toggle in config.

### **ImportExportPanel**

* Export sessions/configs as JSON.
* Import back into local storage.

---

# Local Storage Schema (Draft)

```json
{
  "sessions": {
    "session_id_1": {
      "title": "Gemini test",
      "messages": [
        { "role": "system", "content": "..." },
        { "role": "user", "content": "Hello" },
        { "role": "assistant", "content": "Hi there!" }
      ],
      "created_at": "2025-09-18T12:00:00Z"
    }
  },
  "config": {
    "gemini": { "api_key": "xxx", "model": "gemini-1.5-pro" },
    "openai": { "url": "https://api.openai.com/v1", "token": "xxx", "model": "gpt-4o" },
    "shared": { "temperature": 0.7, "max_tokens": 2048, "retry_delay": 2000 },
    "system_prompt": "You are a helpful assistant",
    "function_tools": [ { "name": "search", "return": "{...}" } ],
    "structured_outputs": [ { "schema": "{...}" } ]
  }
}
```

---

# State Flow Example

1. User opens `App` → loads sessions/configs from LocalStorage.
2. User selects a session in `Sidebar` → `ChatRoom` loads its messages.
3. User sends a message in `InputBar` → `ChatRoom` triggers API call.
4. Response streams in → `MessageList` updates UI.
5. If tool call → `ToolCallView` shows → mock value returned → appended as assistant message.
6. All changes persisted back to LocalStorage.
