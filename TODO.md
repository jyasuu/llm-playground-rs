# help me implement change design of llm model usage

1. new usage way. we would have a default llm providor to access openai api.we can append new llm providors like other openai api compatible or gemini or openai router( it also  kind of openai api compatible) or ollama or webui ...
2. we can choose a model when a new session. it would not be changed. and default model would be openai api.
    - for example after session newed .and not start chat. we can select a models with a dropdownlist component.

## reference llm providers and models configuration from claude code

```json
{
  "Providers": [
    {
      "name": "openrouter",
      "api_base_url": "https://gateway.ai.cloudflare.com/v1/0177dfd3fc04f0bb51d422b49f2dad20/jyasu-demo/openrouter/v1/chat/completions",
      "api_key": "?",
      "models": [
        "deepseek/deepseek-chat-v3-0324:free"
      ],
      "transformer": {
        "use": ["openrouter"]
      }
    },
    {
      "name": "gemini",
      "api_base_url": "https://gateway.ai.cloudflare.com/v1/0177dfd3fc04f0bb51d422b49f2dad20/jyasu-demo/google-ai-studio/v1/models/",
      "api_key": "?",
      "models": ["gemini-2.5-flash", "gemini-2.5-pro"],
      "transformer": {
        "use": ["gemini"]
      }
    },
      {
        "name": "gemini",
        "api_base_url": "https://generativelanguage.googleapis.com/v1beta/models/",
        "api_key": "?",
        "models": ["gemini-2.5-flash", "gemini-2.5-pro"],
        "transformer": {
          "use": ["gemini"]
        }
      },
      {
        "name": "gemini",
        "api_base_url": "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions",
        "api_key": "?",
        "models": ["gemini-2.5-flash", "gemini-2.5-pro"],
        "transformer": {
          "use": ["openapi"]
        }
      },
      {
        "name": "gemini",
        "api_base_url": "https://gateway.ai.cloudflare.com/v1/0177dfd3fc04f0bb51d422b49f2dad20/jyasu-demo/google-ai-studio/v1beta/openai/chat/completions",
        "api_key": "?",
        "models": ["gemini-2.5-flash", "gemini-2.5-pro"],
        "transformer": {
          "use": ["openapi"]
        }
      }
  ],
  "Router": {
    "default": "openrouter,deepseek/deepseek-chat-v3-0324:free",
    "background": "openrouter,deepseek/deepseek-chat-v3-0324:free",
    "think": "openrouter,deepseek/deepseek-chat-v3-0324:free",
    "longContext": "openrouter,deepseek/deepseek-chat-v3-0324:free",
    "longContextThreshold": 60000,
    "webSearch": "openrouter,deepseek/deepseek-chat-v3-0324:free"
  }
}
```


## read @README.md first for know purpose

### you can help me management todo items in @README.md
