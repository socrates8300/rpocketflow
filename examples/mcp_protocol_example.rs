#![allow(unused)]
use log::{error, info};
use rpocketflow::*;
use rpocketflow::async_node::{AsyncNode, async_node, async_then, async_when, AsyncNodeImpl};
use rpocketflow::mcp::mcp_stdio_config;
use serde_json::json;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use async_trait::async_trait;

/// A simple async node for handling user input in the MCP protocol example
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

// Minimal SyncNode implementation for compatibility
impl SyncNode for AsyncUserInputNode {}

#[async_trait]
impl AsyncNode for AsyncUserInputNode {
    async fn exec_async(&mut self, _prep_res: &serde_json::Value) -> NodeResult<serde_json::Value> {
        // User interface to get tool name and parameters
        println!("\nEnter the tool name to call (or 'exit' to quit, 'list' to view tools):");
        print!("> ");
        std::io::stdout().flush().map_err(|e| format!("Failed to flush stdout: {}", e))?;

        // Use spawn_blocking for I/O operations to avoid blocking the async runtime
        let input = tokio::task::spawn_blocking(|| {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).expect("Failed to read input");
            input.trim().to_string()
        }).await.expect("Input task failed");

        if input == "exit" {
            return Ok(json!("exit"));
        }

        if input == "list" {
            return Ok(json!("list"));
        }

        // Ask for parameters if it's a tool call
        println!("Enter the parameters in JSON format (or press Enter for empty params):");
        print!("> ");
        std::io::stdout().flush().map_err(|e| format!("Failed to flush stdout: {}", e))?;

        let params_input = tokio::task::spawn_blocking(|| {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).expect("Failed to read parameters");
            input.trim().to_string()
        }).await.expect("Parameter input task failed");

        let params_value = if params_input.is_empty() {
            json!({})
        } else {
            match serde_json::from_str(&params_input) {
                Ok(v) => v,
                Err(e) => {
                    error!("Invalid JSON parameters: {}", e);
                    println!("Invalid JSON parameters. Using empty params.");
                    json!({})
                }
            }
        };

        Ok(json!({
            "tool_name": input,
            "params": params_value
        }))
    }
    
    async fn post_async(&mut self, shared: &mut Shared, _prep_res: &serde_json::Value, exec_res: &serde_json::Value) -> NodeResult<serde_json::Value> {
        if exec_res.as_str() == Some("exit") {
            return Ok(json!("terminate"));
        }

        if exec_res.as_str() == Some("list") {
            // Display available tools if we have them
            if let Some(tools) = shared.get("mcp_tools") {
                if let Some(tools_obj) = tools.as_object() {
                    println!("\nAvailable tools:");
                    for (name, desc) in tools_obj {
                        println!("  - {} - {}", name, desc);
                    }
                }
            } else {
                println!("Tool list not available yet. Try again after initialization.");
            }
            return Ok(json!("continue"));
        }

        // Store the tool call info in shared state
        shared.insert("mcp_tool_call".to_string(), exec_res.clone());
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

/// A simple async node for displaying MCP tool results
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

// Minimal SyncNode implementation for compatibility
impl SyncNode for AsyncOutputNode {}

#[async_trait]
impl AsyncNode for AsyncOutputNode {
    async fn exec_async(&mut self, _prep_res: &serde_json::Value) -> NodeResult<serde_json::Value> {
        // No processing needed
        Ok(json!(null))
    }
    
    async fn post_async(&mut self, shared: &mut Shared, _prep_res: &serde_json::Value, _exec_res: &serde_json::Value) -> NodeResult<serde_json::Value> {
        // Display the result from the MCP call
        if let Some(result) = shared.get("mcp_exec_result") {
            println!("\nMCP Result:");
            println!("{}", serde_json::to_string_pretty(result).unwrap());
        }

        Ok(json!("continue"))
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
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    // Check if the server executable exists
    let server_path = Path::new("./target/debug/examples/mcp_server_example");
    if !server_path.exists() {
        error!("Server executable not found. Please run 'cargo build --example mcp_server_example' first.");
        return Ok(());
    }

    // Create MCP client configuration using the helper function for stdio-based MCP servers
    let mcp_config = mcp_stdio_config(
        "RPocketFlow Example", 
        "0.1.0",
        server_path.to_string_lossy().to_string(), 
        &[] as &[&str] // No additional arguments
    );

    // Create nodes with proper async implementations
    let protocol_node = mcp_protocol_node("MCPProtocolNode", mcp_config);
    let async_input_node = async_node(AsyncUserInputNode::new("UserInputNode"));
    let async_output_node = async_node(AsyncOutputNode::new("OutputNode"));

    // Create an async flow
    let async_flow = async_flow! {
        name: "MCP Protocol Async Flow",
        start: async_input_node.clone()
    };

    // Set up async connections
    async_when(&async_input_node, "default").then(protocol_node.clone()).await;
    async_when(&protocol_node, "success").then(async_output_node.clone()).await;
    async_when(&protocol_node, "error").then(async_output_node.clone()).await;
    async_when(&async_output_node, "continue").then(async_input_node.clone()).await;

    // Initialize shared state
    let mut shared = HashMap::new();

    // Run the async flow
    info!("Starting MCP Protocol flow...");
    match async_flow.orchestrate(&mut shared, None).await {
        Ok(_) => info!("Flow completed successfully"),
        Err(e) => error!("Flow failed: {}", e),
    }

    Ok(())
}

