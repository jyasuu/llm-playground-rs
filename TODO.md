# help me implement a built-in function tools

1. it is a real function tool instead just for mock
2. it is not able to edit and delete, but disable is possible
3. you may not have to put it in settings panel.

## here a built-in function tools idea

### name: fetch

help me complete function tools description prompts

#### description: it is a tool for http request

#### parameters: url , method, headers, payload

#### return: response headers, body



## read @README.md first for know purpose

### you can help me management todo items in @README.md



## ISSUES


### when using gemini as provider (openai api is works)
```json
{
    "error": {
        "code": 400,
        "message": "Invalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[0].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[0].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[0].parameters.properties[0].value': Cannot find field.\nInvalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[1].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[1].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[2].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[2].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[3].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[3].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[4].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[4].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[5].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[5].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[6].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[6].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[7].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[7].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[8].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[8].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[9].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[9].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[9].parameters.properties[0].value.items': Cannot find field.\nInvalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[10].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[10].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[11].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[11].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[12].parameters': Cannot find field.\nInvalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[12].parameters': Cannot find field.",
        "status": "INVALID_ARGUMENT",
        "details": [
            {
                "@type": "type.googleapis.com/google.rpc.BadRequest",
                "fieldViolations": [
                    {
                        "field": "tools[0].function_declarations[0].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[0].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[0].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[0].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[0].parameters.properties[0].value",
                        "description": "Invalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[0].parameters.properties[0].value': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[1].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[1].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[1].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[1].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[2].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[2].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[2].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[2].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[3].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[3].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[3].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[3].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[4].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[4].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[4].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[4].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[5].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[5].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[5].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[5].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[6].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[6].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[6].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[6].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[7].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[7].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[7].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[7].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[8].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[8].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[8].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[8].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[9].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[9].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[9].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[9].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[9].parameters.properties[0].value.items",
                        "description": "Invalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[9].parameters.properties[0].value.items': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[10].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[10].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[10].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[10].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[11].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[11].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[11].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[11].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[12].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"$schema\" at 'tools[0].function_declarations[12].parameters': Cannot find field."
                    },
                    {
                        "field": "tools[0].function_declarations[12].parameters",
                        "description": "Invalid JSON payload received. Unknown name \"additionalProperties\" at 'tools[0].function_declarations[12].parameters': Cannot find field."
                    }
                ]
            }
        ]
    }
}
```