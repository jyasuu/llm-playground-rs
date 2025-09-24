# seems openai_client function call tool_call_id not match, remember don't generate tool_call_id by self. just use origin from llm

```
{"model":"gemini-2.5-flash","messages":[{"role":"system","content":"You are a helpful assistant that responds in markdown format. Always be concise and to the point."},{"role":"user","content":"hello"},{"role":"assistant","content":"Hello! How can I help you today?\n"},{"role":"user","content":"help me get weather in taipei,tokyo"},{"role":"assistant","tool_calls":[{"id":"function-call-1758708579654","type":"function","function":{"name":"get_weather","arguments":"{\"location\":\"taipei\"}"}},{"id":"function-call-1758708579654","type":"function","function":{"name":"get_weather","arguments":"{\"location\":\"tokyo\"}"}}]},{"role":"tool","content":"{\"condition\":\"sunny\",\"humidity\":65,\"temperature\":22}","name":"get_weather","tool_call_id":"function-call-1621171984081311078"},{"role":"tool","content":"{\"condition\":\"sunny\",\"humidity\":65,\"temperature\":22}","name":"get_weather","tool_call_id":"function-call-1621171984081310699"}],"temperature":0.7,"max_tokens":2048,"tools":[{"function":{"description":"Retrieves weather data for a specified location.","name":"get_weather","parameters":{"properties":{"location":{"description":"The location to get weather for","type":"string"},"unit":{"description":"Temperature unit","enum":["celsius","fahrenheit"],"type":"string"}},"required":["location"],"type":"object"}},"type":"function"}],"tool_choice":"auto"}
```

## reference full function call requset

```sh

curl "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions" \
-H "Content-Type: application/json" \
-H "Authorization: Bearer " \
-d '{
  "model": "gemini-2.0-flash",
  "messages": [
    {
      "role": "user",
      "content": "What'\''s the weather like in Chicago today?"
    },
    {"role":"assistant","tool_calls":[{"function":{"arguments":"{\"location\":\"Chicago, IL\"}","name":"get_weather"},"id":"function-call-8809181845727874956","type":"function"}]},
    {
        "content": "{\"condition\":\"sunny\",\"humidity\":65,\"temperature\":22}",
        "name": "get_weather",
        "role": "tool",
        "tool_call_id": "function-call-8809181845727874956"
    }
  ],
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "get_weather",
        "description": "Get the current weather in a given location",
        "parameters": {
          "type": "object",
          "properties": {
            "location": {
              "type": "string",
              "description": "The city and state, e.g. Chicago, IL"
            }
          },
          "required": ["location"]
        }
      }
    }
  ],
  "tool_choice": "auto"
}'
```