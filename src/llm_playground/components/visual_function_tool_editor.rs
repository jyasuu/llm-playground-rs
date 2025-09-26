use yew::prelude::*;
use web_sys::HtmlInputElement;
use crate::llm_playground::FunctionTool;
use serde_json::json;

// Helper function to generate JSON schema from visual parameters
fn generate_json_schema(parameters: &[Parameter], mock_fields: &[MockResponseField]) -> (String, String) {
    // Generate parameters schema
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();
    
    for param in parameters {
        if !param.name.trim().is_empty() {
            let mut param_schema = serde_json::Map::new();
            param_schema.insert("type".to_string(), json!(param.param_type));
            
            if !param.description.trim().is_empty() {
                param_schema.insert("description".to_string(), json!(param.description));
            }
            
            if !param.enum_values.is_empty() {
                param_schema.insert("enum".to_string(), json!(param.enum_values));
            }
            
            properties.insert(param.name.clone(), serde_json::Value::Object(param_schema));
            
            if param.required {
                required.push(param.name.clone());
            }
        }
    }
    
    let schema = json!({
        "type": "object",
        "properties": properties,
        "required": required
    });
    
    // Generate mock response
    let mut response = serde_json::Map::new();
    for field in mock_fields {
        if !field.name.trim().is_empty() {
            let value = match field.field_type.as_str() {
                "number" => {
                    if let Ok(num) = field.value.parse::<f64>() {
                        json!(num)
                    } else {
                        json!(0)
                    }
                },
                "boolean" => json!(field.value == "true"),
                _ => json!(field.value)
            };
            response.insert(field.name.clone(), value);
        }
    }
    
    let mock_response = serde_json::Value::Object(response);
    
    (
        serde_json::to_string_pretty(&schema).unwrap_or_else(|_| "{}".to_string()),
        serde_json::to_string_pretty(&mock_response).unwrap_or_else(|_| "{}".to_string())
    )
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub required: bool,
    pub enum_values: Vec<String>, // For enum type parameters
}

impl Default for Parameter {
    fn default() -> Self {
        Self {
            name: String::new(),
            param_type: "string".to_string(),
            description: String::new(),
            required: false,
            enum_values: vec![],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MockResponseField {
    pub name: String,
    pub value: String,
    pub field_type: String, // "string", "number", "boolean", "array", "object"
}

impl Default for MockResponseField {
    fn default() -> Self {
        Self {
            name: String::new(),
            value: String::new(),
            field_type: "string".to_string(),
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct VisualFunctionToolEditorProps {
    pub tool: Option<FunctionTool>,
    pub on_save: Callback<FunctionTool>,
    pub on_cancel: Callback<()>,
}

#[function_component(VisualFunctionToolEditor)]
pub fn visual_function_tool_editor(props: &VisualFunctionToolEditorProps) -> Html {
    let function_name = use_state(|| String::new());
    let function_description = use_state(|| String::new());
    let parameters = use_state(|| Vec::<Parameter>::new());
    let mock_fields = use_state(|| vec![MockResponseField::default()]);
    let show_json_preview = use_state(|| false);

    // Initialize from existing tool if editing
    {
        let function_name = function_name.clone();
        let function_description = function_description.clone();
        let parameters = parameters.clone();
        let mock_fields = mock_fields.clone();
        let tool = props.tool.clone();
        
        use_effect_with(tool, move |tool| {
            if let Some(tool) = tool {
                function_name.set(tool.name.clone());
                function_description.set(tool.description.clone());
                
                // Parse existing parameters
                if let Some(properties) = tool.parameters.get("properties") {
                    if let Some(props_obj) = properties.as_object() {
                        let required_fields: Vec<String> = tool.parameters
                            .get("required")
                            .and_then(|r| r.as_array())
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                            .unwrap_or_default();
                        
                        let parsed_params: Vec<Parameter> = props_obj.iter().map(|(name, param)| {
                            let param_type = param.get("type")
                                .and_then(|t| t.as_str())
                                .unwrap_or("string")
                                .to_string();
                            
                            let description = param.get("description")
                                .and_then(|d| d.as_str())
                                .unwrap_or("")
                                .to_string();
                            
                            let enum_values = param.get("enum")
                                .and_then(|e| e.as_array())
                                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                                .unwrap_or_default();
                            
                            Parameter {
                                name: name.clone(),
                                param_type,
                                description,
                                required: required_fields.contains(name),
                                enum_values,
                            }
                        }).collect();
                        
                        parameters.set(parsed_params);
                    }
                }
                
                // Parse existing mock response
                if let Ok(mock_obj) = serde_json::from_str::<serde_json::Value>(&tool.mock_response) {
                    if let Some(obj) = mock_obj.as_object() {
                        let parsed_fields: Vec<MockResponseField> = obj.iter().map(|(name, value)| {
                            let (field_type, string_value) = match value {
                                serde_json::Value::String(s) => ("string".to_string(), s.clone()),
                                serde_json::Value::Number(n) => ("number".to_string(), n.to_string()),
                                serde_json::Value::Bool(b) => ("boolean".to_string(), b.to_string()),
                                serde_json::Value::Array(_) => ("array".to_string(), value.to_string()),
                                serde_json::Value::Object(_) => ("object".to_string(), value.to_string()),
                                serde_json::Value::Null => ("string".to_string(), "null".to_string()),
                            };
                            
                            MockResponseField {
                                name: name.clone(),
                                value: string_value,
                                field_type,
                            }
                        }).collect();
                        
                        if !parsed_fields.is_empty() {
                            mock_fields.set(parsed_fields);
                        }
                    }
                }
            }
            || ()
        });
    }

    html! {
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div class="bg-white dark:bg-gray-800 rounded-lg w-full max-w-4xl max-h-[90vh] overflow-hidden flex flex-col">
                // Header
                <div class="flex justify-between items-center p-6 border-b border-gray-200 dark:border-gray-700">
                    <h3 class="text-xl font-semibold">
                        {if props.tool.is_some() { "Edit Function Tool" } else { "Create Function Tool" }}
                    </h3>
                    <button 
                        onclick={
                            let on_cancel = props.on_cancel.clone();
                            Callback::from(move |_| on_cancel.emit(()))
                        }
                        class="text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                    >
                        <i class="fas fa-times text-xl"></i>
                    </button>
                </div>

                // Content - scrollable
                <div class="flex-1 overflow-y-auto p-6 space-y-6">
                    // Basic Information Section
                    <div class="bg-gray-50 dark:bg-gray-700 p-4 rounded-lg">
                        <h4 class="text-lg font-medium mb-4 flex items-center">
                            <i class="fas fa-info-circle text-blue-500 mr-2"></i>
                            {"Basic Information"}
                        </h4>
                        
                        <div class="grid grid-cols-1 gap-4">
                            <div>
                                <label class="block text-sm font-medium mb-2">{"Function Name"}</label>
                                <input 
                                    type="text"
                                    value={(*function_name).clone()}
                                    oninput={
                                        let function_name = function_name.clone();
                                        Callback::from(move |e: InputEvent| {
                                            let input: HtmlInputElement = e.target_unchecked_into();
                                            function_name.set(input.value());
                                        })
                                    }
                                    placeholder="e.g., get_weather, calculate_price, send_email"
                                    class="w-full p-3 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800"
                                />
                                <p class="text-xs text-gray-500 mt-1">{"Use lowercase with underscores, like: get_weather"}</p>
                            </div>
                            
                            <div>
                                <label class="block text-sm font-medium mb-2">{"Description"}</label>
                                <textarea 
                                    value={(*function_description).clone()}
                                    oninput={
                                        let function_description = function_description.clone();
                                        Callback::from(move |e: InputEvent| {
                                            let input: HtmlInputElement = e.target_unchecked_into();
                                            function_description.set(input.value());
                                        })
                                    }
                                    placeholder="Describe what this function does and when to use it"
                                    rows="3"
                                    class="w-full p-3 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800"
                                />
                                <p class="text-xs text-gray-500 mt-1">{"Be specific about what the function does and when the AI should use it"}</p>
                            </div>
                        </div>
                    </div>

                    // Parameters Section
                    <div class="bg-gray-50 dark:bg-gray-700 p-4 rounded-lg">
                        <h4 class="text-lg font-medium mb-4 flex items-center">
                            <i class="fas fa-cog text-green-500 mr-2"></i>
                            {"Parameters"}
                        </h4>
                        
                        <div class="space-y-4">
                            {for parameters.iter().enumerate().map(|(index, param)| {
                                let parameters = parameters.clone();
                                let delete_param = {
                                    let parameters = parameters.clone();
                                    Callback::from(move |_| {
                                        let mut new_params = (*parameters).clone();
                                        new_params.remove(index);
                                        parameters.set(new_params);
                                    })
                                };

                                html! {
                                    <div key={index} class="border border-gray-200 dark:border-gray-600 rounded-md p-4 bg-white dark:bg-gray-800">
                                        <div class="flex justify-between items-start mb-3">
                                            <h5 class="font-medium">{"Parameter " }{index + 1}</h5>
                                            <button 
                                                onclick={delete_param}
                                                class="text-red-500 hover:text-red-700 text-sm"
                                                title="Delete parameter"
                                            >
                                                <i class="fas fa-trash"></i>
                                            </button>
                                        </div>
                                        
                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                            <div>
                                                <label class="block text-sm font-medium mb-1">{"Name"}</label>
                                                <input 
                                                    type="text"
                                                    value={param.name.clone()}
                                                    oninput={
                                                        let parameters = parameters.clone();
                                                        Callback::from(move |e: InputEvent| {
                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                            let mut new_params = (*parameters).clone();
                                                            new_params[index].name = input.value();
                                                            parameters.set(new_params);
                                                        })
                                                    }
                                                    placeholder="parameter_name"
                                                    class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-sm"
                                                />
                                            </div>
                                            
                                            <div>
                                                <label class="block text-sm font-medium mb-1">{"Type"}</label>
                                                <select 
                                                    value={param.param_type.clone()}
                                                    onchange={
                                                        let parameters = parameters.clone();
                                                        Callback::from(move |e: Event| {
                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                            let mut new_params = (*parameters).clone();
                                                            new_params[index].param_type = input.value();
                                                            parameters.set(new_params);
                                                        })
                                                    }
                                                    class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-sm"
                                                >
                                                    <option value="string">{"Text (string)"}</option>
                                                    <option value="number">{"Number"}</option>
                                                    <option value="boolean">{"True/False (boolean)"}</option>
                                                    <option value="array">{"List (array)"}</option>
                                                </select>
                                            </div>
                                            
                                            <div class="md:col-span-2">
                                                <label class="block text-sm font-medium mb-1">{"Description"}</label>
                                                <input 
                                                    type="text"
                                                    value={param.description.clone()}
                                                    oninput={
                                                        let parameters = parameters.clone();
                                                        Callback::from(move |e: InputEvent| {
                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                            let mut new_params = (*parameters).clone();
                                                            new_params[index].description = input.value();
                                                            parameters.set(new_params);
                                                        })
                                                    }
                                                    placeholder="Describe what this parameter is for"
                                                    class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-sm"
                                                />
                                            </div>
                                            
                                            <div class="md:col-span-2">
                                                <label class="flex items-center">
                                                    <input 
                                                        type="checkbox"
                                                        checked={param.required}
                                                        onchange={
                                                            let parameters = parameters.clone();
                                                            Callback::from(move |e: Event| {
                                                                let input: HtmlInputElement = e.target_unchecked_into();
                                                                let mut new_params = (*parameters).clone();
                                                                new_params[index].required = input.checked();
                                                                parameters.set(new_params);
                                                            })
                                                        }
                                                        class="mr-2"
                                                    />
                                                    <span class="text-sm">{"Required parameter"}</span>
                                                </label>
                                            </div>
                                        </div>
                                    </div>
                                }
                            })}
                            
                            <button 
                                onclick={
                                    let parameters = parameters.clone();
                                    Callback::from(move |_| {
                                        let mut new_params = (*parameters).clone();
                                        new_params.push(Parameter::default());
                                        parameters.set(new_params);
                                    })
                                }
                                class="w-full p-3 border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-md text-gray-500 dark:text-gray-400 hover:border-green-500 hover:text-green-500 dark:hover:border-green-400 dark:hover:text-green-400 transition-colors"
                            >
                                <i class="fas fa-plus mr-2"></i> {"Add Parameter"}
                            </button>
                        </div>
                    </div>

                    // Mock Response Section
                    <div class="bg-gray-50 dark:bg-gray-700 p-4 rounded-lg">
                        <h4 class="text-lg font-medium mb-4 flex items-center">
                            <i class="fas fa-reply text-purple-500 mr-2"></i>
                            {"Mock Response"}
                        </h4>
                        <p class="text-sm text-gray-600 dark:text-gray-300 mb-4">
                            {"Define what the function should return when called. This helps the AI understand the expected output format."}
                        </p>
                        
                        <div class="space-y-4">
                            {for mock_fields.iter().enumerate().map(|(index, field)| {
                                let mock_fields = mock_fields.clone();
                                let delete_field = {
                                    let mock_fields = mock_fields.clone();
                                    Callback::from(move |_| {
                                        let mut new_fields = (*mock_fields).clone();
                                        if new_fields.len() > 1 {
                                            new_fields.remove(index);
                                            mock_fields.set(new_fields);
                                        }
                                    })
                                };

                                html! {
                                    <div key={index} class="border border-gray-200 dark:border-gray-600 rounded-md p-4 bg-white dark:bg-gray-800">
                                        <div class="flex justify-between items-start mb-3">
                                            <h5 class="font-medium">{"Field " }{index + 1}</h5>
                                            {if mock_fields.len() > 1 {
                                                html! {
                                                    <button 
                                                        onclick={delete_field}
                                                        class="text-red-500 hover:text-red-700 text-sm"
                                                        title="Delete field"
                                                    >
                                                        <i class="fas fa-trash"></i>
                                                    </button>
                                                }
                                            } else {
                                                html! {}
                                            }}
                                        </div>
                                        
                                        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                                            <div>
                                                <label class="block text-sm font-medium mb-1">{"Field Name"}</label>
                                                <input 
                                                    type="text"
                                                    value={field.name.clone()}
                                                    oninput={
                                                        let mock_fields = mock_fields.clone();
                                                        Callback::from(move |e: InputEvent| {
                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                            let mut new_fields = (*mock_fields).clone();
                                                            new_fields[index].name = input.value();
                                                            mock_fields.set(new_fields);
                                                        })
                                                    }
                                                    placeholder="result, status, data..."
                                                    class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-sm"
                                                />
                                            </div>
                                            
                                            <div>
                                                <label class="block text-sm font-medium mb-1">{"Type"}</label>
                                                <select 
                                                    value={field.field_type.clone()}
                                                    onchange={
                                                        let mock_fields = mock_fields.clone();
                                                        Callback::from(move |e: Event| {
                                                            let input: HtmlInputElement = e.target_unchecked_into();
                                                            let mut new_fields = (*mock_fields).clone();
                                                            new_fields[index].field_type = input.value();
                                                            mock_fields.set(new_fields);
                                                        })
                                                    }
                                                    class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-sm"
                                                >
                                                    <option value="string">{"Text"}</option>
                                                    <option value="number">{"Number"}</option>
                                                    <option value="boolean">{"True/False"}</option>
                                                </select>
                                            </div>
                                            
                                            <div>
                                                <label class="block text-sm font-medium mb-1">{"Value"}</label>
                                                {if field.field_type == "boolean" {
                                                    html! {
                                                        <select 
                                                            value={field.value.clone()}
                                                            onchange={
                                                                let mock_fields = mock_fields.clone();
                                                                Callback::from(move |e: Event| {
                                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                                    let mut new_fields = (*mock_fields).clone();
                                                                    new_fields[index].value = input.value();
                                                                    mock_fields.set(new_fields);
                                                                })
                                                            }
                                                            class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-sm"
                                                        >
                                                            <option value="true">{"true"}</option>
                                                            <option value="false">{"false"}</option>
                                                        </select>
                                                    }
                                                } else {
                                                    html! {
                                                        <input 
                                                            type={if field.field_type == "number" { "number" } else { "text" }}
                                                            value={field.value.clone()}
                                                            oninput={
                                                                let mock_fields = mock_fields.clone();
                                                                Callback::from(move |e: InputEvent| {
                                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                                    let mut new_fields = (*mock_fields).clone();
                                                                    new_fields[index].value = input.value();
                                                                    mock_fields.set(new_fields);
                                                                })
                                                            }
                                                            placeholder={if field.field_type == "number" { "42" } else { "example value" }}
                                                            class="w-full p-2 border border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 text-sm"
                                                        />
                                                    }
                                                }}
                                            </div>
                                        </div>
                                    </div>
                                }
                            })}
                            
                            <button 
                                onclick={
                                    let mock_fields = mock_fields.clone();
                                    Callback::from(move |_| {
                                        let mut new_fields = (*mock_fields).clone();
                                        new_fields.push(MockResponseField::default());
                                        mock_fields.set(new_fields);
                                    })
                                }
                                class="w-full p-3 border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-md text-gray-500 dark:text-gray-400 hover:border-purple-500 hover:text-purple-500 dark:hover:border-purple-400 dark:hover:text-purple-400 transition-colors"
                            >
                                <i class="fas fa-plus mr-2"></i> {"Add Response Field"}
                            </button>
                        </div>
                    </div>

                    // JSON Preview Section (Optional Toggle)
                    <div class="bg-gray-50 dark:bg-gray-700 p-4 rounded-lg">
                        <div class="flex items-center justify-between mb-4">
                            <h4 class="text-lg font-medium flex items-center">
                                <i class="fas fa-code text-gray-500 mr-2"></i>
                                {"JSON Preview"}
                            </h4>
                            <button 
                                onclick={
                                    let show_json_preview = show_json_preview.clone();
                                    Callback::from(move |_| {
                                        show_json_preview.set(!*show_json_preview);
                                    })
                                }
                                class="text-sm px-3 py-1 bg-gray-200 dark:bg-gray-600 rounded-md hover:bg-gray-300 dark:hover:bg-gray-500"
                            >
                                {if *show_json_preview { "Hide" } else { "Show" }}
                            </button>
                        </div>
                        
                        {if *show_json_preview {
                            let json_schema = generate_json_schema(&parameters, &mock_fields);
                            html! {
                                <div class="space-y-4">
                                    <div>
                                        <h5 class="font-medium mb-2">{"Parameters Schema:"}</h5>
                                        <pre class="bg-gray-800 text-green-400 p-4 rounded-md text-xs overflow-x-auto">
                                            {json_schema.0}
                                        </pre>
                                    </div>
                                    <div>
                                        <h5 class="font-medium mb-2">{"Mock Response:"}</h5>
                                        <pre class="bg-gray-800 text-blue-400 p-4 rounded-md text-xs overflow-x-auto">
                                            {json_schema.1}
                                        </pre>
                                    </div>
                                </div>
                            }
                        } else {
                            html! {}
                        }}
                    </div>
                </div>

                // Footer
                <div class="flex justify-end space-x-2 p-6 border-t border-gray-200 dark:border-gray-700">
                    <button 
                        onclick={
                            let on_cancel = props.on_cancel.clone();
                            Callback::from(move |_| on_cancel.emit(()))
                        }
                        class="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-md hover:bg-gray-50 dark:hover:bg-gray-700"
                    >
                        {"Cancel"}
                    </button>
                    <button 
                        onclick={
                            let function_name = function_name.clone();
                            let function_description = function_description.clone();
                            let parameters = parameters.clone();
                            let mock_fields = mock_fields.clone();
                            let on_save = props.on_save.clone();
                            
                            Callback::from(move |_| {
                                // Validate required fields
                                if function_name.trim().is_empty() || function_description.trim().is_empty() {
                                    return;
                                }
                                
                                // Generate JSON schema from visual parameters
                                let (schema_json, mock_response_json) = generate_json_schema(&parameters, &mock_fields);
                                
                                // Parse schema back to Value for storage
                                let schema = serde_json::from_str(&schema_json)
                                    .unwrap_or_else(|_| json!({
                                        "type": "object",
                                        "properties": {},
                                        "required": []
                                    }));
                                
                                // Create the function tool
                                let tool = FunctionTool {
                                    name: (*function_name).clone(),
                                    description: (*function_description).clone(),
                                    parameters: schema,
                                    mock_response: mock_response_json,
                                    enabled: true,
                                    category: "Custom".to_string(),
                                };
                                
                                on_save.emit(tool);
                            })
                        }
                        disabled={function_name.trim().is_empty() || function_description.trim().is_empty()}
                        class={classes!(
                            "px-4", "py-2", "rounded-md", "text-white",
                            if !function_name.trim().is_empty() && !function_description.trim().is_empty() {
                                "bg-primary-600 hover:bg-primary-700"
                            } else {
                                "bg-gray-400 cursor-not-allowed"
                            }
                        )}
                    >
                        {"Save"}
                    </button>
                </div>
            </div>
        </div>
    }
}