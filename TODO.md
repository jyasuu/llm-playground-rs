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



## ISSUES

### ✅ RESOLVED: when chat with gemini openapi (gemini is works correct)

~~error displayed: ❌ API Error: Failed to parse response: missing field content at line 1 column 232~~

**FIXED**: Updated OpenAI client to handle optional `content` field and properly parse `tool_calls` from response. The issue was that when LLMs make function calls via OpenAI-compatible APIs, the response message may not have a `content` field - only `tool_calls`.

#### request

```json
{"max_tokens":2048,"messages":[{"content":"You are a helpful assistant that responds in markdown format. Always be concise and to the point.","role":"system"},{"content":"hello","role":"user"},{"content":"Hello! How can I assist you today?\n","role":"assistant"},{"content":"help me get weather in taipei","role":"user"}],"model":"gemini-2.5-flash","temperature":0.699999988079071,"tools":[{"function":{"description":"Retrieves weather data for a specified location.","name":"get_weather","parameters":{"properties":{"location":{"description":"The location to get weather for","type":"string"},"unit":{"description":"Temperature unit","enum":["celsius","fahrenheit"],"type":"string"}},"required":["location"],"type":"object"}},"type":"function"}]}
```

#### response

```json
{
    "choices": [
        {
            "finish_reason": "tool_calls",
            "index": 0,
            "message": {
                "role": "assistant",
                "tool_calls": [
                    {
                        "function": {
                            "arguments": "{\"location\":\"Taipei\"}",
                            "name": "get_weather"
                        },
                        "id": "function-call-5471613915444200547",
                        "type": "function"
                    }
                ]
            }
        }
    ],
    "created": 1758544829,
    "id": "vUPRaKiLM52w1MkPu_CK2AQ",
    "model": "gemini-2.5-flash",
    "object": "chat.completion",
    "usage": {
        "completion_tokens": 16,
        "prompt_tokens": 111,
        "total_tokens": 183
    }
}
```

### maybe need reference @chat-cli/src/openai.rs