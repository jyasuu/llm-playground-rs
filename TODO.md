# help me refactor llm client for below both llm api flow



## gemini example
```json
{
  "contents": [
    {
      "parts": [
        {
          "text": "help me use fetch tool to access https://httpbin.org/get and https://httpbin.org/post"
        }
      ],
      "role": "user"
    },
    {
      "parts": [
        {
          "functionCall": {
            "name": "fetch",
            "args": {
              "url": "https://httpbin.org/get"
            }
          }
        },
        {
          "functionCall": {
            "name": "fetch",
            "args": {
              "method": "POST",
              "url": "https://httpbin.org/post"
            }
          }
        }
      ],
      "role": "model"
    },
    {
      "role": "user",
      "parts": [
        {
          "functionResponse": {
            "id": "fetch-1752212231618-6d563cdd578fe",
            "name": "fetch",
            "response": {
              "output": "{\n  \"args\": {}, \n  \"headers\": {\n    \"Accept\": \"*/*\", \n    \"Accept-Encoding\": \"gzip, deflate, br, zstd\", \n    \"Accept-Language\": \"zh-TW,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6,ja;q=0.5\", \n    \"Cache-Control\": \"no-cache\", \n    \"Host\": \"httpbin.org\", \n    \"Origin\": \"https://jyasuu.github.io\", \n    \"Pragma\": \"no-cache\", \n    \"Priority\": \"u=1, i\", \n    \"Referer\": \"https://jyasuu.github.io/\", \n    \"Sec-Ch-Ua\": \"\\\"Microsoft Edge\\\";v=\\\"141\\\", \\\"Not?A_Brand\\\";v=\\\"8\\\", \\\"Chromium\\\";v=\\\"141\\\"\", \n    \"Sec-Ch-Ua-Mobile\": \"?0\", \n    \"Sec-Ch-Ua-Platform\": \"\\\"Windows\\\"\", \n    \"Sec-Fetch-Dest\": \"empty\", \n    \"Sec-Fetch-Mode\": \"cors\", \n    \"Sec-Fetch-Site\": \"cross-site\", \n    \"User-Agent\": \"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36 Edg/141.0.0.0\", \n    \"X-Amzn-Trace-Id\": \"Root=1-690424c1-433375544a27bf202b76e336\"\n  }, \n  \"origin\": \"118.163.189.205\", \n  \"url\": \"https://httpbin.org/get\"\n}\n"
            }
          }
        },
        {
          "functionResponse": {
            "id": "fetch-1752212231618-6d563cdd578ff",
            "name": "fetch",
            "response": {
              "output": "{\n  \"args\": {}, \n  \"headers\": {\n    \"Accept\": \"*/*\", \n    \"Accept-Encoding\": \"gzip, deflate, br, zstd\", \n    \"Accept-Language\": \"zh-TW,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6,ja;q=0.5\", \n    \"Cache-Control\": \"no-cache\", \n    \"Host\": \"httpbin.org\", \n    \"Origin\": \"https://jyasuu.github.io\", \n    \"Pragma\": \"no-cache\", \n    \"Priority\": \"u=1, i\", \n    \"Referer\": \"https://jyasuu.github.io/\", \n    \"Sec-Ch-Ua\": \"\\\"Microsoft Edge\\\";v=\\\"141\\\", \\\"Not?A_Brand\\\";v=\\\"8\\\", \\\"Chromium\\\";v=\\\"141\\\"\", \n    \"Sec-Ch-Ua-Mobile\": \"?0\", \n    \"Sec-Ch-Ua-Platform\": \"\\\"Windows\\\"\", \n    \"Sec-Fetch-Dest\": \"empty\", \n    \"Sec-Fetch-Mode\": \"cors\", \n    \"Sec-Fetch-Site\": \"cross-site\", \n    \"User-Agent\": \"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36 Edg/141.0.0.0\", \n    \"X-Amzn-Trace-Id\": \"Root=1-690424c1-433375544a27bf202b76e336\"\n  }, \n  \"origin\": \"118.163.189.205\", \n  \"url\": \"https://httpbin.org/post\"\n}\n"
            }
          }
        }
      ]
    }
  ],
  "generationConfig": {
    "temperature": 0.7,
    "topP": 0.95,
    "topK": 40,
    "maxOutputTokens": 2048
  },
  "systemInstruction": {
    "parts": [
      {
        "text": "You are a helpful assistant that responds in markdown format. Always be concise and to the point."
      }
    ]
  },
  "tools": [
    {
      "functionDeclarations": [
        {
          "name": "fetch",
          "description": "A tool for making HTTP requests. Supports GET, POST, PUT, DELETE, and other HTTP methods with custom headers and payload.",
          "parameters": {
            "properties": {
              "headers": {
                "description": "HTTP headers to include in the request",
                "type": "object"
              },
              "method": {
                "description": "HTTP method to use (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS)",
                "type": "string"
              },
              "payload": {
                "description": "Request body payload (for POST, PUT, PATCH methods)",
                "type": "string"
              },
              "url": {
                "description": "The URL to make the request to",
                "type": "string"
              }
            },
            "required": ["url"],
            "type": "object"
          }
        }
      ]
    }
  ]
}

```

## openai example 

```json

{
  "model": "sono",
  "messages": [
    {
      "role": "system",
      "content": "You are a helpful assistant that responds in markdown format. Always be concise and to the point."
    },
    { "role": "user", "content": "help me use fetch tool to access https://httpbin.org/get and https://httpbin.org/post" },
    {
        "content": "",
        "role": "assistant",
        "tool_calls": [
          {
            "function": {
              "arguments": "{\"url\":\"https://httpbin.org/get\"}",
              "name": "fetch"
            },
            "id": "call_24802198",
            "type": "function"
          },
          {
            "function": {
              "arguments": "{\"url\":\"https://httpbin.org/post\"}",
              "name": "fetch"
            },
            "id": "call_81618027",
            "type": "function"
          }
        ]
      },
    {
      "role": "tool",
      "content": "{\n  \"args\": {}, \n  \"headers\": {\n    \"Accept\": \"*/*\", \n    \"Accept-Encoding\": \"gzip, deflate, br, zstd\", \n    \"Accept-Language\": \"zh-TW,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6,ja;q=0.5\", \n    \"Cache-Control\": \"no-cache\", \n    \"Host\": \"httpbin.org\", \n    \"Origin\": \"https://jyasuu.github.io\", \n    \"Pragma\": \"no-cache\", \n    \"Priority\": \"u=1, i\", \n    \"Referer\": \"https://jyasuu.github.io/\", \n    \"Sec-Ch-Ua\": \"\\\"Microsoft Edge\\\";v=\\\"141\\\", \\\"Not?A_Brand\\\";v=\\\"8\\\", \\\"Chromium\\\";v=\\\"141\\\"\", \n    \"Sec-Ch-Ua-Mobile\": \"?0\", \n    \"Sec-Ch-Ua-Platform\": \"\\\"Windows\\\"\", \n    \"Sec-Fetch-Dest\": \"empty\", \n    \"Sec-Fetch-Mode\": \"cors\", \n    \"Sec-Fetch-Site\": \"cross-site\", \n    \"User-Agent\": \"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36 Edg/141.0.0.0\", \n    \"X-Amzn-Trace-Id\": \"Root=1-690424c1-433375544a27bf202b76e336\"\n  }, \n  \"origin\": \"118.163.189.205\", \n  \"url\": \"https://httpbin.org/get\"\n}\n",
      "name": "fetch",
      "tool_call_id": "call_24802198"
    },
    {
      "role": "tool",
      "content": "{\n  \"args\": {}, \n  \"headers\": {\n    \"Accept\": \"*/*\", \n    \"Accept-Encoding\": \"gzip, deflate, br, zstd\", \n    \"Accept-Language\": \"zh-TW,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6,ja;q=0.5\", \n    \"Cache-Control\": \"no-cache\", \n    \"Host\": \"httpbin.org\", \n    \"Origin\": \"https://jyasuu.github.io\", \n    \"Pragma\": \"no-cache\", \n    \"Priority\": \"u=1, i\", \n    \"Referer\": \"https://jyasuu.github.io/\", \n    \"Sec-Ch-Ua\": \"\\\"Microsoft Edge\\\";v=\\\"141\\\", \\\"Not?A_Brand\\\";v=\\\"8\\\", \\\"Chromium\\\";v=\\\"141\\\"\", \n    \"Sec-Ch-Ua-Mobile\": \"?0\", \n    \"Sec-Ch-Ua-Platform\": \"\\\"Windows\\\"\", \n    \"Sec-Fetch-Dest\": \"empty\", \n    \"Sec-Fetch-Mode\": \"cors\", \n    \"Sec-Fetch-Site\": \"cross-site\", \n    \"User-Agent\": \"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36 Edg/141.0.0.0\", \n    \"X-Amzn-Trace-Id\": \"Root=1-690424c1-433375544a27bf202b76e336\"\n  }, \n  \"origin\": \"118.163.189.205\", \n  \"url\": \"https://httpbin.org/post\"\n}\n",
      "name": "fetch",
      "tool_call_id": "call_81618027"
    }
  ],
  "temperature": 0.7,
  "max_tokens": 2048,
  "tools": [
    {
      "function": {
        "description": "A tool for making HTTP requests. Supports GET, POST, PUT, DELETE, and other HTTP methods with custom headers and payload.",
        "name": "fetch",
        "parameters": {
          "properties": {
            "headers": {
              "description": "HTTP headers to include in the request",
              "type": "object"
            },
            "method": {
              "description": "HTTP method to use (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS)",
              "type": "string"
            },
            "payload": {
              "description": "Request body payload (for POST, PUT, PATCH methods)",
              "type": "string"
            },
            "url": {
              "description": "The URL to make the request to",
              "type": "string"
            }
          },
          "required": ["url"],
          "type": "object"
        }
      },
      "type": "function"
    }
  ],
  "tool_choice": "auto"
}
```



## i have some think

1. gemini and openai have there own api data structure
2. there difference
  1. system prompt 
    - openai: put at first element in .messages[] 
    - gemini: put at systemInstruction.parts
  2. different data structure
    - openai: message , special different use multiple message to send tool result
    - gemini: content , special different use one content to send tool result
  3. function call id
    - openai: must follow function call id from llm
    - gemini: need generate by system self
3. my idea is
  1. for system prompt: put system prompt in @src\llm_playground\api_clients\gemini_client.rs and @src\llm_playground\api_clients\openai_client.rs instead caller
  2. design a data structure for caller and llm client, and llm client is responsible to data model convention between this generic data model and there own data model. special for function call is use one generic model record to generate multiple message
  3. for function call id: maybe make it just proccess in llm client. for example openai just follow the tool order to mapping there function call id in @src\llm_playground\api_clients\openai_client.rs . for gemini is just simple to generate there id in @src\llm_playground\api_clients\gemini_client.rs

4. maybe this generic data model responsible to consider ui display and data store to browser local storage


## reference @README.md
## reference @src\llm_playground\hooks\use_llm_chat.rs
## reference @src\llm_playground\flexible_client.rs
## reference @src\llm_playground\flexible_playground.rs
