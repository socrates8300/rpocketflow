Okay, let's refine that README to provide a clearer, more accurate, and developer-friendly experience, incorporating the fixes and decisions we've made.

**Key Improvements:**

1.  **Clear Separation:** Explicitly separate examples and explanations for Synchronous (`Flow`) and Asynchronous (`AsyncFlow`) workflows.
2.  **Corrected Examples:** Fix all examples that incorrectly mixed sync/async components. Async examples now exclusively use `AsyncFlow`, `AsyncNodeRef`, `async_node`, `async_flow!`, etc.
3.  **Accurate Installation:** Clarify which dependencies are needed for which features (core, async, specific integrations). Clarify the role of `mcpr`.
4.  **Refined Macro Section:** Remove the problematic `async_node_impl!` for now. Show better examples for `async_flow!`.
5.  **Streamlined MCP Section:** Consolidate redundant MCP information, keeping the comparison table and protocol details. Clearly label the server example as external.
6.  **Clarity and Readability:** Improved formatting, added subsections, and refined explanations.
7.  **Added Async Custom Node Example:** Provides a parallel to the sync version.
8.  **Updated Troubleshooting:** Includes points about the sync/async boundary.

```markdown
# RPocketFlow

A lightweight, flexible workflow orchestration library for Rust that simplifies the creation and execution of complex, potentially agent-based, workflows with minimal overhead. Supports both synchronous and asynchronous execution models.

[![Crates.io](https://img.shields.io/crates/v/rpocketflow.svg)](https://crates.io/crates/rpocketflow)
[![Docs.rs](https://docs.rs/rpocketflow/badge.svg)](https://docs.rs/rpocketflow)
<!-- Add other badges as appropriate: CI/CD status, license, etc. -->

## Overview

RPocketFlow allows you to define workflows as interconnected nodes. Each node performs a specific task, and the flow orchestrator manages the execution order, state transitions, data passing (via shared state), and error handling.

It supports two distinct execution models:

1.  **Synchronous (`Flow`, `SyncNode`, `NodeRef`):** Ideal for CPU-bound tasks or workflows using traditional blocking I/O. Executes steps sequentially on the current thread.
2.  **Asynchronous (`AsyncFlow`, `AsyncNode`, `AsyncNodeRef`):** Built on Tokio, perfect for I/O-bound tasks (network requests, timers, process communication) where non-blocking execution improves performance and responsiveness.

## Features

*   **Modular Node Architecture:** Build reusable workflow components.
*   **Dual Execution Models:** Native support for both synchronous and asynchronous (Tokio-based) workflows.
*   **Intuitive Flow Control:** Define linear sequences, conditional branching, and termination points.
*   **State Management:** Pass data between nodes using a shared state map (`HashMap<String, serde_json::Value>`).
*   **Retry Mechanisms:** Configure automatic retries with delays for fallible operations.
*   **Macros for DX:** `node_impl!`, `flow!`, `async_flow!`, and others reduce boilerplate.
*   **Error Handling:** Uses `thiserror` for structured error reporting (`FlowError`, `FlowResult`).
*   **Minimal Dependencies:** Core library is lightweight; async features depend on Tokio.
*   **Built-in MCP Integrations:**
    *   **Direct Claude API Client (`McpNode`):** Interact with Anthropic's Claude models easily.
    *   **MCP Protocol Client (`MCPProtocolNode`):** Connect to *any* standard MCP-compatible server for broader tool/model integration.

## Installation

Add RPocketFlow and necessary dependencies to your `Cargo.toml`:

```toml
[dependencies]
rpocketflow = "0.1.0" # Check for the latest version

# Core dependencies often used with rpocketflow
serde_json = "1.0"
log = "0.4" # Optional, but recommended for node/flow logging

# Required ONLY for Asynchronous features (AsyncFlow, AsyncNode, async_*, Tokio Mutex)
tokio = { version = "1", features = ["full"] } # Or specific features like "rt-multi-thread", "macros", "sync", "time"
async-trait = "0.1"

# Required ONLY for the direct Claude API integration (McpNode)
anthropic = "0.0.8" # Or latest compatible version
# serde = { version = "1.0", features = ["derive"] } # Often needed with anthropic types

# Required ONLY for the MCP Protocol client integration (MCPProtocolNode)
serde = { version = "1.0", features = ["derive"] } # Needed for MCP request/response serialization

# Optional, commonly used helpers
dotenv = "0.15" # For loading .env files (API keys, etc.)
env_logger = "0.10" # For initializing logging
once_cell = "1.18" # Used internally by rpocketflow for async statics
```

**Note on `mcpr` crate:** Examples showing how to *create* an MCP *server* might use the `mcpr` crate. `mcpr` is **not** a dependency required to *use* RPocketFlow's `MCPProtocolNode` client; it's only needed if you are building the server side yourself.

## Quick Start

### 1. Basic Synchronous Flow (`Flow`)

Ideal for CPU-bound tasks or simple sequences.

```rust
use rpocketflow::*;
use serde_json::{json, Value};
use std::collections::HashMap;

// Define a custom node using the macro
let node1 = node_impl! {
    name: "Start",
    exec: |_: &Value| {
        println!("Node 1 executing.");
        Ok(json!({"step": 1})) // Execution result (often Value::Null is fine)
    },
    post: |shared: &mut Shared, _, exec_res: &Value| {
        println!("Node 1 finished.");
        shared.insert("node1_result".to_string(), exec_res.clone());
        // Post returns the action (as JSON string or Null for default)
        Ok(json!("step_two")) // Named action
    }
};

let node2 = node_impl! {
    name: "StepTwo",
    exec: |_: &Value| {
        println!("Node 2 executing.");
        Ok(Value::Null)
    },
    post: |_, _, _| {
         println!("Node 2 finished.");
         Ok(json!("terminate")) // Terminate the flow
    }
};

let node_fallback = node_impl!{ name: "Fallback", exec: |_:&Value| {println!("Took fallback path!"); Ok(Value::Null)}};

fn main() {
    // Build the flow structure
    let flow = flow! {
        name: "SimpleSyncFlow",
        start: node1.clone(), // Start node
        connections: [
            // (FromNode, ActionName, ToNode)
            (node1.clone(), "step_two", node2.clone()),
            // Example: If node1 returned "default", it would go here
            (node1.clone(), "default", node_fallback.clone()),
            // node2 terminates, no successors needed after it
            (node_fallback.clone(), "default", node2.clone()) // Fallback also leads to node2
        ]
    };

    // Run the flow
    let mut shared_state = HashMap::new();
    println!("Running synchronous flow...");
    match flow.orchestrate(&mut shared_state, None) {
        Ok(_) => println!("Sync flow completed successfully."),
        Err(e) => println!("Sync flow failed: {}", e),
    }
    println!("Final Shared State: {:?}", shared_state);
}
```

### 2. Basic Asynchronous Flow (`AsyncFlow`)

Essential for I/O-bound tasks (network, timers, file system). Requires a Tokio runtime.

```rust
use rpocketflow::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use async_trait::async_trait;

// Define a custom AsyncNode struct
struct AsyncWaitNode {
    base: AsyncNodeImpl, // Embed base implementation
    message: String,
    delay_ms: u64,
}

impl AsyncWaitNode {
    fn new(name: impl Into<String>, message: impl Into<String>, delay_ms: u64) -> Self {
        Self {
            base: AsyncNodeImpl::new(name),
            message: message.into(),
            delay_ms,
        }
    }
}

// Implement the base Node trait (delegating to base)
impl Node for AsyncWaitNode {
    fn get_params(&self) -> &Params { self.base.get_params() }
    fn set_params(&mut self, params: Params) { self.base.set_params(params); }
    fn add_successor(&mut self, action: String, successor: NodeRef) { self.base.add_successor(action, successor); }
    fn get_successors(&self) -> &HashMap<String, NodeRef> { self.base.get_successors() }
    fn get_successors_mut(&mut self) -> &mut HashMap<String, NodeRef> { self.base.get_successors_mut() }
    fn get_name(&self) -> &str { self.base.get_name() }
    fn get_max_retries(&self) -> usize { self.base.get_max_retries() }
    fn get_wait_duration(&self) -> Duration { self.base.get_wait_duration() }
}

// Implement AsyncNode trait
#[async_trait]
impl AsyncNode for AsyncWaitNode {
    // Implement async successor methods (delegating to base)
    fn add_async_successor(&mut self, action: String, successor: AsyncNodeRef) { self.base.add_async_successor(action, successor); }
    fn get_async_successors(&self) -> &HashMap<String, AsyncNodeRef> { self.base.get_async_successors() }
    fn get_async_successors_mut(&mut self) -> &mut HashMap<String, AsyncNodeRef> { self.base.get_async_successors_mut() }

    // Implement the async execution logic
    async fn exec_async(&mut self, _prep_res: &Value) -> NodeResult<Value> {
        println!("Node '{}' starting async work...", self.get_name());
        sleep(Duration::from_millis(self.delay_ms)).await; // Non-blocking sleep
        println!("Node '{}': {}", self.get_name(), self.message);
        Ok(Value::Null)
    }
}

#[tokio::main]
async fn main() {
    // Create async nodes using the wrapper
    let node_a = async_node(AsyncWaitNode::new("A", "Finished A", 100));
    let node_b = async_node(AsyncWaitNode::new("B", "Finished B", 50));
    let node_c = async_node(AsyncWaitNode::new("C", "Finished C", 75));

    // Build the async flow structure using the macro's `nodes` variant
    // The macro handles the `async_then().await` calls internally.
    let flow = async_flow! {
        name: "SimpleAsyncFlow",
        nodes: [node_a.clone(), node_b.clone(), node_c.clone()] // Macro connects these linearly
    };

    // Run the flow
    let mut shared_state = HashMap::new();
    println!("Running asynchronous flow...");
    match flow.orchestrate(&mut shared_state, None).await { // Note the .await
        Ok(_) => println!("Async flow completed successfully."),
        Err(e) => println!("Async flow failed: {}", e),
    }
     println!("Final Shared State: {:?}", shared_state);
}
```

## MCP Integration (Asynchronous)

Interacting with AI models like Claude or MCP servers involves network I/O, making it inherently asynchronous. **These integrations MUST be used within an `AsyncFlow`.**

### 1. Using Direct Claude API (`McpNode`)

Connect directly to Anthropic's API.

```rust
use rpocketflow::*;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use async_trait::async_trait; // Needed for custom async node below
use std::time::Duration;      // Needed for Node trait default

// Simple node to provide input to Claude
struct UserInputNode { base: AsyncNodeImpl }
impl UserInputNode { fn new() -> Self { Self { base: AsyncNodeImpl::new("UserInput") } } }
impl Node for UserInputNode { /* Delegate basic methods to base */
    fn get_params(&self) -> &Params { self.base.get_params() } /* ... etc ... */
    fn set_params(&mut self, params: Params) { self.base.set_params(params); }
    fn add_successor(&mut self, action: String, successor: NodeRef) { self.base.add_successor(action, successor); }
    fn get_successors(&self) -> &HashMap<String, NodeRef> { self.base.get_successors() }
    fn get_successors_mut(&mut self) -> &mut HashMap<String, NodeRef> { self.base.get_successors_mut() }
    fn get_name(&self) -> &str { self.base.get_name() }
    fn get_max_retries(&self) -> usize { self.base.get_max_retries() }
    fn get_wait_duration(&self) -> Duration { self.base.get_wait_duration() }
}
#[async_trait]
impl AsyncNode for UserInputNode {
    fn add_async_successor(&mut self, action: String, successor: AsyncNodeRef) { self.base.add_async_successor(action, successor); } /* ... delegate other successors ... */
    fn get_async_successors(&self) -> &HashMap<String, AsyncNodeRef> { self.base.get_async_successors() }
    fn get_async_successors_mut(&mut self) -> &mut HashMap<String, AsyncNodeRef> { self.base.get_async_successors_mut() }
    async fn post_async(&mut self, shared: &mut Shared, _prep: &Value, _exec: &Value) -> NodeResult<Value> {
        shared.insert("mcp_input".to_string(), json!("Explain the benefits of Rust's borrow checker."));
        Ok(json!("default")) // Proceed to next node
    }
}

// Simple node to display Claude's output
struct DisplayOutputNode { base: AsyncNodeImpl }
impl DisplayOutputNode { fn new() -> Self { Self { base: AsyncNodeImpl::new("DisplayOutput") } } }
impl Node for DisplayOutputNode { /* Delegate basic methods to base */ /* ... */
    fn get_params(&self) -> &Params { self.base.get_params() }
    fn set_params(&mut self, params: Params) { self.base.set_params(params); }
    fn add_successor(&mut self, action: String, successor: NodeRef) { self.base.add_successor(action, successor); }
    fn get_successors(&self) -> &HashMap<String, NodeRef> { self.base.get_successors() }
    fn get_successors_mut(&mut self) -> &mut HashMap<String, NodeRef> { self.base.get_successors_mut() }
    fn get_name(&self) -> &str { self.base.get_name() }
    fn get_max_retries(&self) -> usize { self.base.get_max_retries() }
    fn get_wait_duration(&self) -> Duration { self.base.get_wait_duration() }
}
#[async_trait]
impl AsyncNode for DisplayOutputNode {
    fn add_async_successor(&mut self, action: String, successor: AsyncNodeRef) { self.base.add_async_successor(action, successor); } /* ... delegate other successors ... */
    fn get_async_successors(&self) -> &HashMap<String, AsyncNodeRef> { self.base.get_async_successors() }
    fn get_async_successors_mut(&mut self) -> &mut HashMap<String, AsyncNodeRef> { self.base.get_async_successors_mut() }
    async fn prep_async(&mut self, shared: &mut Shared) -> NodeResult<Value> {
        if let Some(output) = shared.get("mcp_output") {
            println!("--- Claude's Response ---");
            println!("{}", output.as_str().unwrap_or("Error retrieving response."));
            println!("-------------------------");
        } else {
            println!("No output found from Claude node.");
        }
        Ok(Value::Null)
    }
    async fn post_async(&mut self, _s: &mut Shared, _p: &Value, _e: &Value) -> NodeResult<Value> {
        Ok(json!("terminate")) // End flow after displaying
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok(); // Load .env file
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init(); // Setup logging

    let api_key = env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY must be set in .env file");

    // Create config for Claude node
    let mcp_config = McpConfig::new(api_key, Models::CLAUDE_3_HAIKU) // Use a fast model
        .with_system_prompt("You are a concise assistant.");

    // Create the nodes (all must be AsyncNodeRef)
    let input_node = async_node(UserInputNode::new());
    let claude_node = mcp_node("Claude", mcp_config); // Returns AsyncNodeRef
    let output_node = async_node(DisplayOutputNode::new());

    // Build the AsyncFlow
    let flow = async_flow! {
        name: "AsyncClaudeQA",
        nodes: [input_node.clone(), claude_node.clone(), output_node.clone()]
    };

    // Run the AsyncFlow
    let mut shared = HashMap::new();
    log::info!("Starting Async Claude Flow...");
    match flow.orchestrate(&mut shared, None).await {
        Ok(_) => log::info!("Async flow completed successfully."),
        Err(e) => log::error!("Async flow failed: {}", e),
    }
    Ok(())
}
```

### 2. Using MCP Protocol Client (`MCPProtocolNode`)

Connect to any standard MCP server (requires a server running separately or managed by the node).

```rust
use rpocketflow::*;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use async_trait::async_trait; // Needed for custom async node below
use std::time::Duration;      // Needed for Node trait default
use log; // Use the log crate facade

// Node to prepare the MCP tool call
struct PrepareToolCallNode { base: AsyncNodeImpl }
impl PrepareToolCallNode { fn new() -> Self { Self { base: AsyncNodeImpl::new("PrepareToolCall") } } }
impl Node for PrepareToolCallNode { /* Delegate basic methods */ /* ... */
    fn get_params(&self) -> &Params { self.base.get_params() }
    fn set_params(&mut self, params: Params) { self.base.set_params(params); }
    fn add_successor(&mut self, action: String, successor: NodeRef) { self.base.add_successor(action, successor); }
    fn get_successors(&self) -> &HashMap<String, NodeRef> { self.base.get_successors() }
    fn get_successors_mut(&mut self) -> &mut HashMap<String, NodeRef> { self.base.get_successors_mut() }
    fn get_name(&self) -> &str { self.base.get_name() }
    fn get_max_retries(&self) -> usize { self.base.get_max_retries() }
    fn get_wait_duration(&self) -> Duration { self.base.get_wait_duration() }
}
#[async_trait]
impl AsyncNode for PrepareToolCallNode {
    fn add_async_successor(&mut self, action: String, successor: AsyncNodeRef) { self.base.add_async_successor(action, successor); } /* Delegate successors */
    fn get_async_successors(&self) -> &HashMap<String, AsyncNodeRef> { self.base.get_async_successors() }
    fn get_async_successors_mut(&mut self) -> &mut HashMap<String, AsyncNodeRef> { self.base.get_async_successors_mut() }
    async fn post_async(&mut self, shared: &mut Shared, _prep: &Value, _exec: &Value) -> NodeResult<Value> {
        // Define the tool call structure for the MCPProtocolNode
        shared.insert("mcp_tool_call".to_string(), json!({
            "tool_name": "echo", // Assumes server has an 'echo' tool
            "params": { "message": "Hello from RPocketFlow MCP Client!" }
        }));
        Ok(json!("default"))
    }
}

// Node to display the MCP execution result
struct DisplayMcpResultNode { base: AsyncNodeImpl }
impl DisplayMcpResultNode { fn new() -> Self { Self { base: AsyncNodeImpl::new("DisplayMcpResult") } } }
impl Node for DisplayMcpResultNode { /* Delegate basic methods */ /* ... */
    fn get_params(&self) -> &Params { self.base.get_params() }
    fn set_params(&mut self, params: Params) { self.base.set_params(params); }
    fn add_successor(&mut self, action: String, successor: NodeRef) { self.base.add_successor(action, successor); }
    fn get_successors(&self) -> &HashMap<String, NodeRef> { self.base.get_successors() }
    fn get_successors_mut(&mut self) -> &mut HashMap<String, NodeRef> { self.base.get_successors_mut() }
    fn get_name(&self) -> &str { self.base.get_name() }
    fn get_max_retries(&self) -> usize { self.base.get_max_retries() }
    fn get_wait_duration(&self) -> Duration { self.base.get_wait_duration() }
}
#[async_trait]
impl AsyncNode for DisplayMcpResultNode {
    fn add_async_successor(&mut self, action: String, successor: AsyncNodeRef) { self.base.add_async_successor(action, successor); } /* Delegate successors */
    fn get_async_successors(&self) -> &HashMap<String, AsyncNodeRef> { self.base.get_async_successors() }
    fn get_async_successors_mut(&mut self) -> &mut HashMap<String, AsyncNodeRef> { self.base.get_async_successors_mut() }
    async fn prep_async(&mut self, shared: &mut Shared) -> NodeResult<Value> {
        if let Some(result) = shared.get("mcp_exec_result") {
            println!("--- MCP Execution Result ---");
            match result.get("status").and_then(|s| s.as_str()) {
                Some("success") => {
                    println!("Status: SUCCESS");
                    println!("Result: {}", result.get("result").unwrap_or(&json!("(No result field)")));
                }
                Some("error") => {
                    println!("Status: ERROR");
                    println!("Error: {}", result.get("error").unwrap_or(&json!("(No error field)")));
                }
                _ => println!("Status: UNKNOWN/MISSING"),
            }
             println!("Raw: {}", serde_json::to_string_pretty(result).unwrap_or_default());
             println!("--------------------------");
        } else {
            println!("No execution result found from MCP node.");
        }
        Ok(Value::Null)
    }
     async fn post_async(&mut self, _s: &mut Shared, _p: &Value, _e: &Value) -> NodeResult<Value> {
        Ok(json!("terminate"))
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Configure connection to an MCP server (e.g., running via stdio)
    // Replace "/path/to/your/mcp_server" with the actual command
    // Ensure the server executable exists and is runnable.
    let server_command = env::var("MCP_SERVER_COMMAND")
        .expect("MCP_SERVER_COMMAND must be set in .env (e.g., path to an 'echo' server)");

    let mcp_config = mcp_stdio_config(
        "RPocketFlowClient",
        env!("CARGO_PKG_VERSION"),
        &server_command, // Path to server executable
        &[] as &[&str] // No extra arguments in this case
    );

    // Create nodes (all AsyncNodeRef)
    let prep_node = async_node(PrepareToolCallNode::new());
    let protocol_node = mcp_protocol_node("MCPClient", mcp_config); // Returns AsyncNodeRef
    let display_node = async_node(DisplayMcpResultNode::new());

    // Build AsyncFlow using connections
    let flow = async_flow! {
        name: "AsyncMCPProtocolFlow",
        start: prep_node.clone(),
        connections: [
            (prep_node.clone(), "default", protocol_node.clone()),
            // MCPProtocolNode returns "success" or "error" actions
            (protocol_node.clone(), "success", display_node.clone()),
            (protocol_node.clone(), "error", display_node.clone()) // Also display errors
        ]
    };

     // Run the AsyncFlow
    let mut shared = HashMap::new();
    log::info!("Starting Async MCP Protocol Flow...");
    match flow.orchestrate(&mut shared, None).await {
        Ok(_) => log::info!("Async flow completed successfully."),
        Err(e) => log::error!("Async flow failed: {}", e),
    }
    Ok(())
}
```
*(**Note:** The MCP Protocol example requires a separate MCP server process conforming to the protocol, such as a simple echo server, specified by `MCP_SERVER_COMMAND`.)*

### Tool Usage with Claude (`McpNode`)

You can equip the `McpNode` with tools it can call:

```rust
// --- (Requires async context: #[tokio::main] async fn main()) ---
use rpocketflow::*;
use rpocketflow::mcp::tools::{Tool, ToolRegistry, string_param};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use async_trait::async_trait;
use std::time::Duration; // For Node trait

// (Assume ApiKey, McpConfig setup, UserInputNode, DisplayOutputNode as in previous Claude example)

// Define a Tool Handler Node (can be a SyncNode wrapped, or a dedicated AsyncNode)
// For simplicity, let's use node_impl! and wrap it.
let weather_tool_node = node_impl! {
    name: "WeatherToolHandler",
    exec: |prep_res: &Value| {
        // Extract location from the parameters Claude sent
        let location = prep_res
            .get("location")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown location");

        println!("[Tool] Getting weather for: {}", location);
        // Simulate API call
        Ok(json!({
            "tool_name": "get_current_weather", // Match the tool name Claude called
            "status": "success",
            "result": { // The actual data to return to Claude
                "temperature": "75F",
                "condition": "Sunny",
                "location": location
            }
        }))
    },
    post: |shared: &mut Shared, _, exec_res: &Value| {
        // Store the *tool execution result* for Claude to consume
        shared.insert("mcp_tool_result".to_string(), exec_res.clone());
        Ok(json!("tool_done")) // Signal tool execution is finished
    }
};

// --- Important: Wrap the sync tool handler node for use in AsyncFlow ---
// Define the SyncToAsyncNodeWrapper struct and its impls (as shown in previous responses)
// Or place it in a utility module. For brevity, assume it's available here.
// pub struct SyncToAsyncNodeWrapper<T: SyncNode + Send + Sync + 'static> { /* ... */ }
// impl<T...> Node for SyncToAsyncNodeWrapper<T> { /* ... */ }
// #[async_trait] impl<T...> AsyncNode for SyncToAsyncNodeWrapper<T> { /* ... */ }

let async_weather_tool_node = async_node(SyncToAsyncNodeWrapper::new(
    // Need to extract the inner SyncNode from the Arc<Mutex<...>> returned by node_impl!
    // This is awkward. A dedicated async tool handler node struct might be cleaner.
    // Or, modify node_impl! to optionally return the inner struct directly? (Complex)
    // Let's assume for now we have a way to get the inner T: SyncNode
     panic!("Need a way to get inner node from node_impl! or define tool handler directly as AsyncNode")
     // Placeholder: Replace panic with actual mechanism if available, or define handler manually.
     // For example, if node_impl! generated `struct ToolHandler { base: BaseNode }`
     // you might manually create `ToolHandler { base: BaseNode::new(...) }`
     // instead of using the macro here.
));


// Create the tool definition for Claude
let weather_tool = Tool::new(
    "get_current_weather",
    "Get the current weather for a specific location.",
    json!({
        "type": "object",
        "properties": {
            "location": string_param("The city and state, e.g., San Francisco, CA")
        },
        "required": ["location"]
    })
);

let mut tool_registry = ToolRegistry::new();
tool_registry.register(weather_tool);

// --- Build the AsyncFlow ---
// (Assume input_node, claude_node, output_node are AsyncNodeRefs)

let flow = async_flow! {
    name: "AsyncClaudeWithTools",
    start: input_node.clone(),
    connections: [
        // Input -> Claude
        (input_node.clone(), "default", claude_node.clone()),

        // Claude decides to use a tool -> Tool Handler Node
        // Action name must match the tool name!
        (claude_node.clone(), "get_current_weather", async_weather_tool_node.clone()),

        // Tool Handler finishes -> Back to Claude with results
        (async_weather_tool_node.clone(), "tool_done", claude_node.clone()),

        // Claude gives final answer -> Output
        (claude_node.clone(), "default", output_node.clone())
    ]
};

// --- Run the Flow ---
let mut shared = HashMap::new();
// IMPORTANT: Provide the tool registry and potentially the tool call parameters
// format expected by McpNode (needs checking in McpNode implementation details)
shared.insert("tool_registry".to_string(), json!(tool_registry.to_tool_declarations()));
shared.insert("mcp_input".to_string(), json!("What's the weather like in Boston?")); // Prompt Claude

log::info!("Starting Async Claude Flow with Tools...");
// ... rest of orchestrate call ...
```
*(**Note:** The tool usage example highlights remaining complexities, especially around integrating `node_impl!` results (SyncNodes) into async flows gracefully. Defining tool handlers directly as `AsyncNode` structs is often cleaner than wrapping.)*

## Macros for Simplified Usage

RPocketFlow includes macros to reduce boilerplate:

### Node Creation

*   **`node_impl!` (for Synchronous Nodes):** Creates simple `SyncNode` implementations.
    ```rust
    let simple_sync = node_impl! { name: "SyncLogger", exec: |_| { println!("Sync!"); Ok(Value::Null) } };
    ```
*   **Direct Implementation (for Async Nodes):** Currently recommended for `AsyncNode` due to macro complexities with async closures. Define a struct and use `#[async_trait]` to implement `AsyncNode`. Wrap with `async_node()`.
    ```rust
    struct MyAsyncNode { /* ... */ }
    impl Node for MyAsyncNode { /* ... */ }
    #[async_trait] impl AsyncNode for MyAsyncNode { /* ... */ }
    let my_async = async_node(MyAsyncNode::new( /* ... */ ));
    ```
    *(Note: A robust `async_node_impl!` macro might be added in the future.)*

### Flow Creation

*   **`flow!` (for Synchronous Flows):** Defines `Flow` structures.
    ```rust
    // Linear sync flow
    let sync_flow = flow! { name: "SyncSequence", nodes: [sync1, sync2] };
    // Branching sync flow
    let sync_branching = flow! { name: "SyncBranch", start: s_start, connections: [(s_start, "action", s_next)] };
    ```
*   **`async_flow!` (for Asynchronous Flows):** Defines `AsyncFlow` structures. Requires `.await` during linking if not using `nodes:` variant. Must be called from an `async` context.
    ```rust
    // Linear async flow (macro handles linking)
    let async_linear = async_flow! { name: "AsyncSequence", nodes: [async1.clone(), async2.clone()] };
    // Branching async flow (macro handles linking)
    let async_branching = async_flow! { name: "AsyncBranch", start: a_start.clone(), connections: [(a_start.clone(), "action", a_next.clone())] };
    // Manual linking (if needed) requires await
    // let async_manual_flow = async_flow! { name: "ManualLink", start: node_x.clone() };
    // async_then(&node_x, node_y.clone()).await;
    ```

### MCP Node Creation

*   **`claude_node_macro!`:** Creates an `McpNode` (Claude client, `AsyncNodeRef`).
    ```rust
    let claude_node = claude_node_macro! { name: "Claude", api_key: key, model: Models::CLAUDE_3_HAIKU };
    ```
*   **`mcp_protocol_node_macro!`:** Creates an `MCPProtocolNode` (Generic MCP client, `AsyncNodeRef`).
    ```rust
    let mcp_client = mcp_protocol_node_macro! { name: "MCPToolClient", server_command: "/path/to/server" };
    ```

### MCP Tool Handling

*   **`mcp_tool_handler!`:** Creates a `SyncNode` specifically designed to handle MCP tool calls *within a synchronous flow*. Use direct implementation or `SyncToAsyncNodeWrapper` for async flows.
    ```rust
    // Creates a SyncNode / NodeRef
    let sync_tool_handler = mcp_tool_handler! {
        name: "SyncWeatherHandler",
        tool_name: "get_weather",
        handler: |params| { /* ... sync logic ... */ Ok(json!(...)) }
    };
    ```

## Core Concepts

*   **`Node`:** Base trait for all workflow units.
*   **`SyncNode`:** Trait for nodes performing blocking operations. Used with `Flow` and `NodeRef` (`Arc<std::sync::Mutex<...>>`).
*   **`AsyncNode`:** Trait for nodes performing non-blocking I/O. Used with `AsyncFlow` and `AsyncNodeRef` (`Arc<tokio::sync::Mutex<...>>`).
*   **`Flow` / `AsyncFlow`:** Orchestrators for sync/async workflows, managing execution and state.
*   **`Action`:** Enum (`Default`, `Named(String)`, `Terminate`) returned by nodes to direct flow control.
*   **`Shared`:** `HashMap<String, Value>` passed mutably between nodes for data sharing.
*   **`NodeRef` / `AsyncNodeRef`:** Thread-safe, reference-counted pointers (`Arc<Mutex<...>>`) to node implementations.

## Advanced Usage

### Conditional Branching

*   **Sync:** Use `when(&node, "action").then(next_node)` within `flow!`.
*   **Async:** Use `async_when(&node, "action").then(next_node).await` within `async_flow!` or manual setup.

### Retries

Configure on nodes using builder methods or direct implementation:

```rust
// Sync (using node_impl!)
let node = node_impl!{ name: "Risky", exec: |_| Err("Failed".into()), max_retries: 3, wait_duration: Duration::from_secs(1) };

// Async (manual impl)
impl MyAsyncNode {
    fn with_retries(mut self, retries: usize, wait: Duration) -> Self {
        self.base = self.base.with_max_retries(retries).with_wait_duration(wait);
        self
    }
}
let async_node = async_node(MyAsyncNode::new(...).with_retries(3, Duration::from_secs(1)));
```

### Custom Node Implementation

*   **Sync:** Define struct, implement `Node`, implement `SyncNode`. Wrap with `node()`.
*   **Async:** Define struct (often embedding `AsyncNodeImpl`), implement `Node`, implement `AsyncNode` (using `#[async_trait]`). Wrap with `async_node()`.

### Sync Nodes in Async Flows

If you need to include a purely synchronous node (defined via `node_impl!` or implementing `SyncNode`) within an `AsyncFlow`:

```rust
use rpocketflow::*; // Assuming SyncToAsyncNodeWrapper is available (see previous responses)

// 1. Create your sync node (returns NodeRef)
let my_sync_node_ref = node_impl! { name: "SyncTask", exec: |_| { println!("Blocking work!"); Ok(Value::Null) }};

// 2. Get the inner SyncNode (This is the tricky part, depends on how node_impl returns)
//    Option A: If node_impl! only returns Arc<Mutex<...>>, you might need manual struct definition.
//    Option B: Assume you have the inner struct instance `my_sync_node_instance`.

// Let's assume manual definition for clarity:
struct MySyncTask { base: BaseNode }
impl MySyncTask { fn new() -> Self { Self{ base: BaseNode::new("SyncTask")} } }
impl Node for MySyncTask { /* ... delegates ... */ }
impl SyncNode for MySyncTask { fn exec(&mut self, _:&Value) -> NodeResult<Value> { println!("Blocking work!"); Ok(Value::Null) }}
// ... add missing Node delegates ...

// 3. Wrap the sync node instance for async use
let my_async_wrapped_node = async_node(SyncToAsyncNodeWrapper::new( MySyncTask::new() ));

// 4. Use the wrapped node in AsyncFlow
// let async_flow = async_flow!{ name: "MixedFlow", nodes: [start_async.clone(), my_async_wrapped_node.clone()] };
// async_flow.orchestrate(&mut shared, None).await;
```
*(**Note:** Be mindful that the wrapped sync node will block the async thread it runs on.)*

## Troubleshooting Common Issues

*   **Type Mismatches (`NodeRef` vs `AsyncNodeRef`):** Ensure you are using `Flow` exclusively with `NodeRef` (sync) and `AsyncFlow` exclusively with `AsyncNodeRef` (async). Do not mix them directly in flow definitions. Use the `SyncToAsyncNodeWrapper` only when necessary.
*   **Blocking in Async Code:** Avoid calling blocking functions directly within `async fn` methods of `AsyncNode`. Use `.await` for async I/O. For long CPU-bound work or blocking sync I/O libraries, use `tokio::task::spawn_blocking`.
*   **Lifetimes in Closures (`async move`):** If using macros that take async closures, be mindful of captured references. Clone data *before* `.await` points if the reference might not live long enough. Direct `AsyncNode` implementation often avoids macro-related lifetime complexity.
*   **MCP Connection Issues:** Verify server addresses/commands, API keys (use `dotenv`), and whether the server uses stdio or network sockets. Check logs on both client and server.
*   **No MCP Output:** Ensure the flow correctly passes input (`mcp_input` or `mcp_tool_call`) to the MCP node and retrieves output (`mcp_output` or `mcp_exec_result`). Check for errors returned by the MCP node.

## Best Practices

*   **Choose the Right Flow:** Use `Flow` for sync, `AsyncFlow` for async.
*   **Environment Variables:** Use `.env` and `dotenv` for configuration (API keys, URLs).
*   **Logging:** Implement logging (`log` crate + `env_logger` or similar) to trace flow execution.
*   **Error Handling:** Nodes should return meaningful `FlowError`s. Flows should handle potential orchestratioon errors.
*   **State Management:** Keep the `Shared` state reasonably sized. Pass only necessary data. Consider alternatives for very large data blobs.
*   **Node Granularity:** Design nodes to perform logical, reusable units of work.

## License

RPocketFlow is licensed under the MIT License. See the `LICENSE` file for details.

## Contributing

Contributions are welcome! Please follow standard GitHub fork & pull request workflows. Ensure tests pass (`cargo test --workspace`) and code is formatted (`cargo fmt`) and linted (`cargo clippy --workspace -- -D warnings`).
```
