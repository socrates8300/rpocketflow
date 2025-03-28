use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::{Value, json};

/// Represents a function that can be called by the model
#[derive(Serialize, Deserialize)]
pub struct Tool {
    /// The name of the tool
    pub name: String,
    /// A description of what the tool does
    pub description: String,
    /// The parameters the tool accepts in JSON Schema format
    pub parameters: Value,
    /// The function to execute when the tool is called
    #[serde(skip)]
    pub handler: Option<ToolHandler>,
}

impl std::fmt::Debug for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tool")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("parameters", &self.parameters)
            .field("handler", &self.handler.is_some())
            .finish()
    }
}

/// Function handler for tools
pub type ToolHandler = Box<dyn Fn(Value) -> Result<Value, String> + Send + Sync>;

impl Clone for Tool {
    fn clone(&self) -> Self {
        // Create a new tool without the handler
        Tool {
            name: self.name.clone(),
            description: self.description.clone(),
            parameters: self.parameters.clone(),
            handler: None,
        }
    }
}

impl Tool {
    pub fn new(name: impl Into<String>, description: impl Into<String>, parameters: Value) -> Self {
        Tool {
            name: name.into(),
            description: description.into(),
            parameters,
            handler: None,
        }
    }

    pub fn with_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(Value) -> Result<Value, String> + Send + Sync + 'static,
    {
        self.handler = Some(Box::new(handler));
        self
    }

    /// Convert to Anthropic's tool format
    pub fn to_function_declaration(&self) -> Value {
        json!({
            "name": self.name,
            "description": self.description,
            "parameters": self.parameters
        })
    }

    /// Execute the tool with the given arguments
    pub fn execute(&self, args: Value) -> Result<Value, String> {
        match &self.handler {
            Some(handler) => handler(args),
            None => Err(format!("No handler defined for tool {}", self.name)),
        }
    }
}

/// A registry of tools that can be used in an MCP conversation
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        ToolRegistry {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Tool) {
        self.tools.insert(tool.name.clone(), tool);
    }

    pub fn get(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }

    pub fn get_all(&self) -> Vec<&Tool> {
        self.tools.values().collect()
    }

    pub fn to_tool_declarations(&self) -> Vec<Value> {
        self.tools.values()
            .map(|tool| tool.to_function_declaration())
            .collect()
    }

    /// Process tool calls from a model response
    pub fn process_tool_call(&self, name: &str, args: Value) -> Value {
        // Try to execute the function
        let result = match self.get(name) {
            Some(tool) => tool.execute(args.clone()),
            None => Err(format!("Tool not found: {}", name)),
        };
        
        // Convert the result to a response
        match result {
            Ok(value) => match serde_json::to_value(value) {
                Ok(val) => val,
                Err(_) => json!({"error": "Failed to serialize result"})
            },
            Err(e) => json!({"error": e}),
        }
    }
}

/// Helper function to create a simple parameter schema for a string
pub fn string_param(description: &str) -> Value {
    json!({
        "type": "string",
        "description": description
    })
}

/// Helper function to create a simple parameter schema for a number
pub fn number_param(description: &str) -> Value {
    json!({
        "type": "number",
        "description": description
    })
}

/// Helper function to create a simple parameter schema for a boolean
pub fn boolean_param(description: &str) -> Value {
    json!({
        "type": "boolean",
        "description": description
    })
}

/// Helper function to create a simple parameter schema for an object
pub fn object_param(description: &str, properties: HashMap<&str, Value>) -> Value {
    json!({
        "type": "object",
        "description": description,
        "properties": properties
    })
}