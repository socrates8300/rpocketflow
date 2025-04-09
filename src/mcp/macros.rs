/// Create a simple MCP node with minimal configuration requirements.
///
/// # Examples
///
/// ```rust
/// use rpocketflow::{mcp_simple, Models};
/// use std::env;
///
/// // Create a basic MCP node using Claude
/// let api_key = env::var("ANTHROPIC_API_KEY").expect("API key missing");
/// let claude = mcp_simple!("ClaudeAssistant", api_key, Models::CLAUDE_3_HAIKU);
/// ```
#[macro_export]
macro_rules! mcp_simple {
    ($name:expr, $api_key:expr, $model:expr) => {{
        use $crate::mcp::{McpConfig, mcp_node};
        
        let config = McpConfig::new($api_key, $model);
        mcp_node($name, config)
    }};
    
    ($name:expr, $api_key:expr, $model:expr, $system_prompt:expr) => {{
        use $crate::mcp::{McpConfig, mcp_node};
        
        let config = McpConfig::new($api_key, $model)
            .with_system_prompt($system_prompt);
        mcp_node($name, config)
    }};
    
    ($name:expr, $api_key:expr, $model:expr, $system_prompt:expr, $max_tokens:expr) => {{
        use $crate::mcp::{McpConfig, mcp_node};
        
        let config = McpConfig::new($api_key, $model)
            .with_system_prompt($system_prompt)
            .with_max_tokens($max_tokens);
        mcp_node($name, config)
    }};
    
    ($name:expr, $api_key:expr, $model:expr, $system_prompt:expr, $max_tokens:expr, $temperature:expr) => {{
        use $crate::mcp::{McpConfig, mcp_node};
        
        let config = McpConfig::new($api_key, $model)
            .with_system_prompt($system_prompt)
            .with_max_tokens($max_tokens)
            .with_temperature($temperature);
        mcp_node($name, config)
    }};
}

/// Create a tool for MCP function calling with simpler syntax than manually creating
/// Tool objects. Supports only string parameters for simplicity.
///
/// # Examples
///
/// ```rust
/// use rpocketflow::mcp_tool;
/// use serde_json::json;
///
/// // Create a weather tool
/// let weather_tool = mcp_tool!("get_weather", "Get the weather for a location", [
///     ("location", "The city and country to get weather for")
/// ], |args| {
///     let location = args["location"].as_str().unwrap_or("unknown");
///     Ok(json!({
///         "temperature": 72,
///         "condition": "sunny",
///         "location": location
///     }))
/// });
/// ```
#[macro_export]
macro_rules! mcp_tool {
    ($name:expr, $description:expr, [
        $(($param_name:expr, $param_desc:expr)),+ $(,)?
    ], $handler:expr) => {{
        use $crate::mcp::tools::{Tool, string_param};
        use serde_json::json;
        
        let parameters = json!({
            "type": "object",
            "properties": {
                $( $param_name: string_param($param_desc), )+
            },
            "required": [ $( $param_name, )+ ]
        });
        
        Tool::new($name, $description, parameters)
            .with_handler($handler)
    }};
}

/// Create a complete MCP conversation flow with input and output handling in one macro.
///
/// # Examples
///
/// ```rust
/// use rpocketflow::{mcp_flow, Models};
/// use std::env;
///
/// // Create an MCP conversation flow
/// let api_key = env::var("ANTHROPIC_API_KEY").expect("API key missing");
/// let flow = mcp_flow!("ConversationFlow", api_key, Models::CLAUDE_3_SONNET,
///     system: "You are a helpful assistant that provides concise answers."
/// );
/// ```
#[macro_export]
macro_rules! mcp_flow {
    ($name:expr, $api_key:expr, $model:expr, 
     system: $system:expr
     $(, max_tokens: $max_tokens:expr)?
     $(, temperature: $temp:expr)?
    ) => {{
        use $crate::{async_flow, create_node, mcp_simple};
        use serde_json::json;
        
        // Create input node
        let input_node = create_node!("InputNode", 
            exec: |_| {
                Ok(json!("continue"))
            }
        );
        
        // Create MCP node
        let mcp_node = mcp_simple!("McpNode", $api_key, $model, $system
            $(, $max_tokens)?
            $(, $temp)?
        );
        
        // Create output node
        let output_node = create_node!("OutputNode", 
            exec: |_| {
                Ok(json!("done"))
            }
        );
        
        // Connect nodes in a flow
        async_flow!($name, input_node, mcp_node, output_node)
    }};
}

/// Register multiple tools for MCP function calling with a simple syntax.
///
/// # Examples
///
/// ```rust
/// use rpocketflow::{register_tools, mcp_tool};
/// use serde_json::json;
///
/// // Create a tool registry with multiple tools
/// let registry = register_tools! {
///     mcp_tool!("get_weather", "Get weather information", [
///         ("location", "City name")
///     ], |args| {
///         let location = args["location"].as_str().unwrap_or("unknown");
///         Ok(json!({"temp": 72, "condition": "sunny"}))
///     }),
///     
///     mcp_tool!("calculate", "Perform basic math", [
///         ("expression", "Mathematical expression to evaluate")
///     ], |args| {
///         let expr = args["expression"].as_str().unwrap_or("0");
///         // In real code, you'd evaluate the expression
///         Ok(json!({"result": 42}))
///     })
/// };
/// ```
#[macro_export]
macro_rules! register_tools {
    ($($tool:expr),+ $(,)?) => {{
        use $crate::mcp::tools::ToolRegistry;
        
        let mut registry = ToolRegistry::new();
        $(
            registry.register($tool);
        )+
        
        registry
    }};
}

