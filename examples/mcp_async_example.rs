use rpocketflow::*;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use async_trait::async_trait;

use rpocketflow::mcp::tools::{string_param, Tool, ToolRegistry};

/// A simple node for reading user input asynchronously
struct AsyncUserInputNode {
    base: AsyncNodeImpl,
}

impl AsyncUserInputNode {
    fn new(name: impl Into<String>) -> Self {
        Self { 
            base: AsyncNodeImpl::new(name) 
        }
    }
}

impl Node for AsyncUserInputNode {
    fn get_params(&self) -> &Params { self.base.get_params() }
    fn set_params(&mut self, params: Params) { self.base.set_params(params); }
    fn add_successor(&mut self, action: String, successor: NodeRef) { self.base.add_successor(action, successor); }
    fn get_successors(&self) -> &HashMap<String, NodeRef> { self.base.get_successors() }
    fn get_successors_mut(&mut self) -> &mut HashMap<String, NodeRef> { self.base.get_successors_mut() }
    fn get_name(&self) -> &str { self.base.get_name() }
    fn get_max_retries(&self) -> usize { self.base.get_max_retries() }
    fn get_wait_duration(&self) -> std::time::Duration { self.base.get_wait_duration() }
}

// Minimal SyncNode implementation
impl SyncNode for AsyncUserInputNode {}

#[async_trait]
impl AsyncNode for AsyncUserInputNode {
    async fn exec_async(&mut self, _prep_res: &serde_json::Value) -> NodeResult<serde_json::Value> {
        println!("Ask about the weather in a specific location:");
        
        // Create a separate task for reading input to avoid blocking the async runtime
        let input = tokio::task::spawn_blocking(|| {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).expect("Failed to read input");
            input.trim().to_string()
        }).await.expect("Input task failed");
        
        Ok(json!(input))
    }
    
    async fn post_async(&mut self, shared: &mut Shared, _prep_res: &serde_json::Value, exec_res: &serde_json::Value) -> NodeResult<serde_json::Value> {
        shared.insert("mcp_input".to_string(), exec_res.clone());
        Ok(json!("default"))
    }
    
    fn add_async_successor(&mut self, action: String, successor: AsyncNodeRef) {
        self.base.add_async_successor(action, successor);
    }
    
    fn get_async_successors(&self) -> &HashMap<String, AsyncNodeRef> {
        self.base.get_async_successors()
    }
    
    fn get_async_successors_mut(&mut self) -> &mut HashMap<String, AsyncNodeRef> {
        self.base.get_async_successors_mut()
    }
}

/// A simple node for displaying output and getting continuation asynchronously
struct AsyncOutputNode {
    base: AsyncNodeImpl,
}

impl AsyncOutputNode {
    fn new(name: impl Into<String>) -> Self {
        Self { 
            base: AsyncNodeImpl::new(name) 
        }
    }
}

impl Node for AsyncOutputNode {
    fn get_params(&self) -> &Params { self.base.get_params() }
    fn set_params(&mut self, params: Params) { self.base.set_params(params); }
    fn add_successor(&mut self, action: String, successor: NodeRef) { self.base.add_successor(action, successor); }
    fn get_successors(&self) -> &HashMap<String, NodeRef> { self.base.get_successors() }
    fn get_successors_mut(&mut self) -> &mut HashMap<String, NodeRef> { self.base.get_successors_mut() }
    fn get_name(&self) -> &str { self.base.get_name() }
    fn get_max_retries(&self) -> usize { self.base.get_max_retries() }
    fn get_wait_duration(&self) -> std::time::Duration { self.base.get_wait_duration() }
}

// Minimal SyncNode implementation
impl SyncNode for AsyncOutputNode {}

#[async_trait]
impl AsyncNode for AsyncOutputNode {
    async fn exec_async(&mut self, _prep_res: &serde_json::Value) -> NodeResult<serde_json::Value> {
        // Doesn't need to do any processing
        Ok(json!(null))
    }
    
    async fn post_async(&mut self, shared: &mut Shared, _prep_res: &serde_json::Value, _exec_res: &serde_json::Value) -> NodeResult<serde_json::Value> {
        // Display the MCP output
        if let Some(output) = shared.get("mcp_output") {
            if let Some(text) = output.as_str() {
                println!("\nClaude's response:");
                println!("{}\n", text);
            }
        }

        // Ask if the user wants to continue
        println!("Do you want to continue? (yes/no)");
        
        // Create a separate task for reading input to avoid blocking
        let input = tokio::task::spawn_blocking(|| {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).expect("Failed to read input");
            input.trim().to_lowercase()
        }).await.expect("Input task failed");
        
        if input == "yes" {
            Ok(json!("continue"))
        } else {
            Ok(json!("terminate"))
        }
    }
    
    fn add_async_successor(&mut self, action: String, successor: AsyncNodeRef) {
        self.base.add_async_successor(action, successor);
    }
    
    fn get_async_successors(&self) -> &HashMap<String, AsyncNodeRef> {
        self.base.get_async_successors()
    }
    
    fn get_async_successors_mut(&mut self) -> &mut HashMap<String, AsyncNodeRef> {
        self.base.get_async_successors_mut()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    simple_logger::init_with_level(log::Level::Info)
        .map_err(|e| format!("Failed to initialize logger: {}", e))?;

    // Get API key from environment
    let api_key =
        env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY environment variable must be set");

    // Initialize MCP config
    let mcp_config = McpConfig::new(api_key, Models::CLAUDE_3_HAIKU)
        .with_system_prompt("You are a helpful assistant. Be concise and informative.")
        .with_max_tokens(1000)
        .with_temperature(0.7);

    // Create a tool registry
    let mut registry = ToolRegistry::new();

    // Create a weather lookup tool
    let weather_tool = Tool::new(
        "get_weather",
        "Get the current weather for a location",
        json!({
            "type": "object",
            "properties": {
                "location": string_param("The city and state, e.g. San Francisco, CA"),
                "unit": string_param("The temperature unit to use: 'celsius' or 'fahrenheit'")
            },
            "required": ["location"]
        }),
    )
    .with_handler(|args| {
        let location = args
            .get("location")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let unit = args
            .get("unit")
            .and_then(|v| v.as_str())
            .unwrap_or("celsius");

        let temp = if unit == "fahrenheit" { 72 } else { 22 };
        let condition = "sunny";

        Ok(json!({
            "temperature": temp,
            "unit": unit,
            "condition": condition,
            "location": location,
        }))
    });

    // Register the tool
    registry.register(weather_tool);

    // Create pure async nodes
    let async_input_node = async_node(AsyncUserInputNode::new("AsyncInputNode"));
    let async_output_node = async_node(AsyncOutputNode::new("AsyncOutputNode"));
    
    // Create MCP node (already an AsyncNode)
    let mcp_node = mcp_node("ClaudeNode", mcp_config);

    // Create an async flow
    let async_flow = async_flow! {
        name: "MCP Async Conversation Flow",
        start: async_input_node.clone()
    };

    // Set up async connections
    async_when(&async_input_node, "default").then(mcp_node.clone()).await;
    async_when(&mcp_node, "default").then(async_output_node.clone()).await;
    async_when(&async_output_node, "continue").then(async_input_node.clone()).await;

    // Initialize shared state
    let mut shared = HashMap::new();

    // Run the async flow
    match async_flow.orchestrate(&mut shared, None).await {
        Ok(_) => println!("Flow completed successfully"),
        Err(e) => eprintln!("Flow failed: {}", e),
    }

    Ok(())
}