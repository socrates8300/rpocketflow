//! Tool functionality for MCP nodes
//!
//! This module provides the `Tool` and `ToolRegistry` types for creating and
//! managing tools that can be called by MCP models.

use std::collections::HashMap;
use tracing::{debug, warn, error};
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
    /// The handler state
    #[serde(skip)]
    handler_state: HandlerState,
}

/// State of the tool handler
enum HandlerState {
    /// Handler is available
    Present(ToolHandler),
    /// Handler was never set
    None,
    /// Handler was removed during clone
    RemovedDuringClone,
}

impl std::fmt::Debug for HandlerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandlerState::Present(_) => write!(f, "Present(...)"),
            HandlerState::None => write!(f, "None"),
            HandlerState::RemovedDuringClone => write!(f, "RemovedDuringClone"),
        }
    }
}

impl Default for HandlerState {
    fn default() -> Self {
        HandlerState::None
    }
}

impl std::fmt::Debug for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tool")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("parameters", &self.parameters)
            .field("handler_available", &self.handler_available())
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
            handler_state: HandlerState::RemovedDuringClone,
        }
    }
}

impl Tool {
    pub fn new(name: impl Into<String>, description: impl Into<String>, parameters: Value) -> Self {
        Tool {
            name: name.into(),
            description: description.into(),
            parameters,
            handler_state: HandlerState::None,
        }
    }

    pub fn with_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(Value) -> Result<Value, String> + Send + Sync + 'static,
    {
        self.handler_state = HandlerState::Present(Box::new(handler));
        self
    }
    
    /// Check if the tool has a handler available
    pub fn handler_available(&self) -> bool {
        matches!(self.handler_state, HandlerState::Present(_))
    }
    
    /// Get handler state description
    pub fn handler_state_description(&self) -> &'static str {
        match self.handler_state {
            HandlerState::Present(_) => "available",
            HandlerState::None => "not set",
            HandlerState::RemovedDuringClone => "removed during clone",
        }
    }

    /// Convert to API function declaration format
    pub fn to_function_declaration(&self) -> Value {
        json!({
            "name": self.name,
            "description": self.description,
            "parameters": self.parameters
        })
    }

    /// Execute the tool with the given arguments
    pub fn execute(&self, args: Value) -> Result<Value, String> {
        match &self.handler_state {
            HandlerState::Present(handler) => {
                debug!(target: "rpocketflow::mcp::tools", tool = %self.name, "Executing tool");
                handler(args)
            },
            HandlerState::None => {
                error!(target: "rpocketflow::mcp::tools", tool = %self.name, "No handler defined for tool");
                Err(format!("No handler defined for tool {}", self.name))
            },
            HandlerState::RemovedDuringClone => {
                error!(
                    target: "rpocketflow::mcp::tools", 
                    tool = %self.name,
                    "Handler was removed during cloning"
                );
                Err(format!(
                    "Handler for tool {} was removed during cloning. Use original tool instance for execution.", 
                    self.name
                ))
            },
        }
    }
}

/// A registry of tools that can be used in an MCP conversation
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        ToolRegistry {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Tool) {
        if !tool.handler_available() {
            warn!(
                target: "rpocketflow::mcp::tools",
                tool = %tool.name,
                handler_state = %tool.handler_state_description(),
                "Registering tool without available handler"
            );
        }
        
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
        debug!(
            target: "rpocketflow::mcp::tools",
            tool = %name,
            "Processing tool call"
        );
        
        // Try to execute the function
        let result = match self.get(name) {
            Some(tool) => {
                if tool.handler_available() {
                    tool.execute(args.clone())
                } else {
                    error!(
                        target: "rpocketflow::mcp::tools",
                        tool = %name,
                        handler_state = %tool.handler_state_description(),
                        "Tool found but handler not available"
                    );
                    Err(format!(
                        "Tool handler not available: {} ({})", 
                        name, 
                        tool.handler_state_description()
                    ))
                }
            },
            None => {
                error!(
                    target: "rpocketflow::mcp::tools",
                    tool = %name,
                    "Tool not found"
                );
                Err(format!("Tool not found: {}", name))
            },
        };
        
        // Convert the result to a response
        match result {
            Ok(value) => match serde_json::to_value(value) {
                Ok(val) => val,
                Err(e) => {
                    error!(
                        target: "rpocketflow::mcp::tools",
                        tool = %name,
                        error = %e,
                        "Failed to serialize result"
                    );
                    json!({"error": format!("Failed to serialize result: {}", e)})
                }
            },
            Err(e) => {
                error!(
                    target: "rpocketflow::mcp::tools",
                    tool = %name,
                    error = %e,
                    "Tool execution failed"
                );
                json!({"error": e})
            },
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

