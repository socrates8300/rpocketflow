I'll update the README.md by resolving the merge conflicts and integrating both versions into a coherent document:

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
*   **Enhanced Macros:** `create_node!`, `sequential_flow!`, `pipeline!`, and more for even better ergonomics.
*   **Error Handling:** Uses `thiserror` for structured error reporting (`FlowError`, `FlowResult`).
*   **Minimal Dependencies:** Core library is lightweight; async features depend on Tokio.
*   **Built-in MCP Integrations:** Direct Claude API Client and MCP Protocol Client for broader tool/model integration.
*   **Tracing Integration:** Better logging using the `tracing` crate instead of println.

## Installation

Add RPocketFlow and necessary dependencies to your `Cargo.toml`:

```toml
[dependencies]
rpocketflow = "0.1.0" # Check for the latest version

# Core dependencies often used with rpocketflow
serde_json = "1.0"
tracing = "0.1" # Recommended for node/flow logging

# Required ONLY for Asynchronous features (AsyncFlow, AsyncNode, async_*, Tokio Mutex)
tokio = { version = "1", features = ["full"] } # Or specific features like "rt-multi-thread", "macros", "sync", "time"
async-trait = "0.1"

# Required ONLY for the direct Claude API integration (McpNode)
anthropic = "0.0.8" # Or latest compatible version
# serde = { version = "1.0", features = ["derive"] } # Often needed with anthropic types

# Optional, commonly used helpers
dotenv = "0.15" # For loading .env files (API keys, etc.)
tracing-subscriber = "0.3" # For initializing tracing
```

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
    async fn exec_async(&mut self, _prep_res: &Value) -> NodeResult<Value> {
        println!("Node '{}' starting async work...", self.get_name());
        sleep(Duration::from_millis(self.delay_ms)).await; // Non-blocking sleep
        println!("Node '{}': {}", self.get_name(), self.message);
        Ok(Value::Null)
    }
}

#[tokio::main]
async fn main() {
    // Create async nodes
    let node_a = node(AsyncWaitNode::new("A", "Finished A", 100));
    let node_b = node(AsyncWaitNode::new("B", "Finished B", 50));
    let node_c = node(AsyncWaitNode::new("C", "Finished C", 75));

    // Create and run the flow
    let flow = async_flow!("SimpleAsyncFlow", node_a, node_b, node_c);
    
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

Interacting with AI models like Claude or MCP servers involves network I/O, making it inherently asynchronous.

### Using Direct Claude API (`McpNode`)

Connect directly to Anthropic's API.

```rust
use rpocketflow::*;
use serde_json::json;
use std::collections::HashMap;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize environment
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    // Get API key from environment
    let api_key = env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");
    
    // Create a config for Claude
    let mcp_config = McpConfig::new(api_key, Models::CLAUDE_3_HAIKU)
        .with_system_prompt("You are a helpful assistant specialized in Rust programming.")
        .with_max_tokens(1000)
        .with_temperature(0.7);
    
    // Create an MCP node to interact with Claude
    let mcp_node = mcp_node("ClaudeNode", mcp_config);
    
    // Create input and output nodes using the improved macros
    let input_node = create_node!("UserInput", 
        exec: |_| {
            Ok(json!("What are the advantages of using Rust over C++?"))
        },
        post: |shared, _, exec_res| {
            shared.insert("mcp_input".to_string(), exec_res.clone());
            Ok(json!("default"))
        }
    );
    
    let output_node = create_node!("ResponseOutput", 
        exec: |_| Ok(json!(null)),
        post: |shared, _, _| {
            if let Some(output) = shared.get("mcp_output") {
                println!("Claude's response:\n{}", output);
            }
            Ok(json!("terminate"))
        }
    );
    
    // Build the flow with the simpler sequential_flow! macro
    let flow = sequential_flow!("ClaudeQAFlow", input_node, mcp_node, output_node);
    
    // Run the flow
    let mut shared = HashMap::new();
    match flow.orchestrate(&mut shared, None) {
        Ok(_) => println!("Flow completed successfully"),
        Err(e) => println!("Flow failed: {}", e),
    }
    
    Ok(())
}
```

### Tool Usage with Claude (`McpNode`)

You can equip the `McpNode` with tools it can call:

```rust
use rpocketflow::*;
use serde_json::json;
use std::collections::HashMap;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize environment
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();
    
    // Get API key from environment
    let api_key = env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");
        
    // Create tools for function calling
    let tools = mcp_tools! {
        mcp_tool!("get_weather", "Get weather information", [
            ("location", "City name")
        ], |args| {
            let location = args["location"].as_str().unwrap_or("unknown");
            println!("Tool called: get_weather for {}", location);
            Ok(json!({
                "temperature": 72,
                "condition": "sunny",
                "humidity": 45,
                "location": location
            }))
        }),
        
        mcp_tool!("calculate", "Perform basic math", [
            ("expression", "Mathematical expression to evaluate")
        ], |args| {
            let expr = args["expression"].as_str().unwrap_or("0");
            println!("Tool called: calculate with expression {}", expr);
            // This is a simplified example
            Ok(json!({"result": 42}))
        })
    };
    
    // Create MCP config
    let mcp_config = McpConfig::new(api_key, Models::CLAUDE_3_HAIKU)
        .with_system_prompt("You are a helpful assistant with access to weather and calculator tools. Use them when appropriate.");
    
    // Create MCP node with tools
    let mcp_node = mcp_node_with_tools("Claude", mcp_config, tools);
    
    // Create input and output nodes
    let input_node = create_node!("UserInput", 
        exec: |_| {
            Ok(json!("What's the weather like in Boston, and what is 5 + 7?"))
        },
        post: |shared, _, exec_res| {
            shared.insert("mcp_input".to_string(), exec_res.clone());
            Ok(json!("default"))
        }
    );
    
    let output_node = create_node!("ResponseOutput", 
        exec: |_| Ok(json!(null)),
        post: |shared, _, _| {
            if let Some(output) = shared.get("mcp_output") {
                println!("Claude's response:\n{}", output);
            }
            
            if let Some(tool_results) = shared.get("mcp_tool_results") {
                println!("Tool results: {}", tool_results);
            }
            
            Ok(json!("terminate"))
        }
    );
    
    // Build the flow
    let flow = sequential_flow!("ClaudeToolFlow", input_node, mcp_node, output_node);
    
    // Run the flow
    let mut shared = HashMap::new();
    match flow.orchestrate(&mut shared, None) {
        Ok(_) => println!("Flow completed successfully"),
        Err(e) => println!("Flow failed: {}", e),
    }
    
    Ok(())
}
```

## Enhanced Macros for Simplified Workflow Creation

RPocketFlow offers a set of enhanced macros that make it even easier to build workflows with minimal boilerplate while maintaining full backward compatibility with existing code.

### Node Creation with `create_node!`

The `create_node!` macro provides a more concise syntax for creating nodes:

```rust
use rpocketflow::*;
use serde_json::json;

// Simple node with just an execution function
let simple_node = create_node!("SimpleNode", |_| {
    println!("Node executed!");
    Ok(json!("continue"))
});

// Full node with prep, exec, and post handlers
let full_node = create_node!("FullNode", 
    prep: |shared| {
        shared.insert("prepared".to_string(), json!(true));
        Ok(json!({"data": 42}))
    },
    exec: |prep_res| {
        let data = prep_res["data"].as_i64().unwrap_or(0);
        Ok(json!(data * 2))
    },
    post: |shared, prep_res, exec_res| {
        shared.insert("result".to_string(), exec_res.clone());
        Ok(json!("next_step"))
    }
);

// Node with retry configuration
let resilient_node = create_node!("ResilientNode", 
    exec: |_| {
        // Some potentially failing operation
        Ok(json!("success"))
    },
    max_retries: 3,
    wait_duration: std::time::Duration::from_millis(100)
);
```

### Simplified Flow Creation

#### Sequential Flows with `sequential_flow!`

```rust
use rpocketflow::*;
use serde_json::json;

// Create some nodes
let node1 = create_node!("Step1", |_| Ok(json!("continue")));
let node2 = create_node!("Step2", |_| Ok(json!("continue")));
let node3 = create_node!("Step3", |_| Ok(json!("done")));

// Create a sequential flow (much simpler than the original syntax)
let flow = sequential_flow!("MyFlow", node1, node2, node3);
```

#### Branching Flows with `branching_flow!`

```rust
use rpocketflow::*;
use serde_json::json;

// Create some nodes
let start = create_node!("Start", |_| Ok(json!("path_a")));
let path_a = create_node!("PathA", |_| Ok(json!("done")));
let path_b = create_node!("PathB", |_| Ok(json!("done")));
let end = create_node!("End", |_| Ok(json!("terminate")));

// Create a branching flow with a clearer syntax
let flow = branching_flow!("BranchingFlow", start => {
    "path_a" => path_a => "default" => end,
    "path_b" => path_b => "default" => end
});
```

### Decision Nodes with `decide!`

The `decide!` macro simplifies creating decision nodes:

```rust
use rpocketflow::*;
use serde_json::json;

// Create a decision node that routes based on the "score" in shared state
let router = decide!("Router", |_, shared| {
    if let Some(score) = shared.get("score") {
        if score.as_f64().unwrap_or(0.0) > 0.7 {
            "high_path"
        } else if score.as_f64().unwrap_or(0.0) > 0.3 {
            "medium_path"
        } else {
            "low_path"
        }
    } else {
        "error_path"
    }
});

// Notice you can return &str instead of String, making the code cleaner
```

### Processing Pipelines with `pipeline!`

The `pipeline!` macro creates data processing chains:

```rust
use rpocketflow::*;
use serde_json::json;

// Create a data processing pipeline
let processor = pipeline!("DataProcessor", 
    // Step 1: Extract value
    |data| {
        let value = data["input"].as_i64().unwrap_or(0);
        Ok(json!(value))
    },
    // Step 2: Double the value
    |value| {
        let doubled = value.as_i64().unwrap_or(0) * 2;
        Ok(json!(doubled))
    },
    // Step 3: Format as object
    |doubled| {
        Ok(json!({
            "original": doubled / 2,
            "result": doubled
        }))
    }
);
```

## MCP-Specific Macros

RPocketFlow includes macros specifically designed for MCP integration.

### Quick MCP Node Creation with `mcp_node!`

```rust
use rpocketflow::*;
use std::env;

// Get API key from environment
let api_key = env::var("ANTHROPIC_API_KEY").expect("API key missing");

// Create an MCP node with minimal configuration
let claude = mcp_node!("ClaudeAssistant", api_key, Models::CLAUDE_3_HAIKU);

// With system prompt
let claude_with_prompt = mcp_node!(
    "ClaudeAssistant", 
    api_key, 
    Models::CLAUDE_3_SONNET,
    "You are a helpful assistant specialized in Rust programming."
);

// With full configuration
let claude_full = mcp_node!(
    "ClaudeAssistant", 
    api_key, 
    Models::CLAUDE_3_OPUS,
    "You are a helpful assistant specialized in Rust programming.",
    2000,  // max_tokens
    0.7    // temperature
);
```

### Easy Tool Creation with `mcp_tool!`

```rust
use rpocketflow::*;
use serde_json::json;

// Create a weather tool
let weather_tool = mcp_tool!(
    "get_weather", 
    "Get the weather for a location", 
    [
        ("location", "The city and country to get weather for")
    ], 
    |args| {
        let location = args["location"].as_str().unwrap_or("unknown");
        Ok(json!({
            "temperature": 72,
            "condition": "sunny",
            "location": location
        }))
    }
);

// Create a calculator tool
let calculator_tool = mcp_tool!(
    "calculate", 
    "Perform a mathematical calculation", 
    [
        ("expression", "The mathematical expression to evaluate")
    ], 
    |args| {
        let expression = args["expression"].as_str().unwrap_or("0");
        // In a real implementation, you would evaluate the expression
        Ok(json!({
            "result": 42,
            "expression": expression
        }))
    }
);
```

### Registering Multiple Tools with `mcp_tools!`

```rust
use rpocketflow::*;
use serde_json::json;

// Create a tool registry with multiple tools
let registry = mcp_tools! {
    mcp_tool!("get_weather", "Get weather information", [
        ("location", "City name")
    ], |args| {
        let location = args["location"].as_str().unwrap_or("unknown");
        Ok(json!({"temp": 72, "condition": "sunny"}))
    }),
    
    mcp_tool!("calculate", "Perform basic math", [
        ("expression", "Mathematical expression to evaluate")
    ], |args| {
        let expr = args["expression"].as_str().unwrap_or("0");
        Ok(json!({"result": 42}))
    })
};
```

### Complete MCP Flow with `mcp_flow!`

```rust
use rpocketflow::*;
use std::env;

// Get API key from environment
let api_key = env::var("ANTHROPIC_API_KEY").expect("API key missing");

// Create a complete MCP flow with input and output handling
let flow = mcp_flow!("ConversationFlow", 
    api_key, 
    Models::CLAUDE_3_SONNET,
    system: "You are a helpful assistant that provides concise answers."
);

// With additional parameters
let flow_with_params = mcp_flow!("ConversationFlow", 
    api_key, 
    Models::CLAUDE_3_OPUS,
    system: "You are a helpful assistant that provides concise answers.",
    max_tokens: 2000,
    temperature: 0.7
);
```

## Comparing Original vs. Enhanced Syntax

### Original Syntax

```rust
// Creating a node
let node = node_impl! {
    name: "MyNode",
    prep: |shared| {
        shared.insert("prepared".to_string(), Value::Bool(true));
        Ok(Value::Null)
    },
    exec: |prep_res| {
        // Execute node logic
        Ok(Value::Null)
    }
};

// Creating a flow
let flow = flow! {
    name: "MyFlow",
    nodes: [node1, node2, node3]
};

// Decision node
let decision = decision_node! {
    name: "MyDecision",
    condition: |params, shared| {
        // Complex decision logic
        "default".to_string()
    }
};

// Creating an MCP node
let mcp_config = McpConfig::new(api_key, Models::CLAUDE_3_HAIKU)
    .with_system_prompt("You are a helpful assistant")
    .with_max_tokens(1000);
let mcp_node = mcp_node("ClaudeNode", mcp_config);
```

### Enhanced Syntax

```rust
// Creating a node
let node = create_node!("MyNode", 
    prep: |shared| {
        shared.insert("prepared".to_string(), json!(true));
        Ok(json!(null))
    },
    exec: |prep_res| {
        // Execute node logic
        Ok(json!(null))
    }
);

// Simple exec-only node
let simple_node = create_node!("SimpleNode", |_| {
    Ok(json!("done"))
});

// Creating a flow 
let flow = sequential_flow!("MyFlow", node1, node2, node3);

// Decision node
let decision = decide!("MyDecision", |params, shared| {
    // Simpler decision logic returning &str instead of String
    "default"
});

// Creating an MCP node
let mcp_node = mcp_node!("ClaudeNode", api_key, Models::CLAUDE_3_HAIKU, 
    "You are a helpful assistant", 1000);
```

## Core Concepts

At its core, RPocketFlow is built around the following ideas:

* **Nodes:** The basic building blocks that execute specific tasks. Each node may have preparation, execution, and post-processing phases.
* **Flows:** Structured networks of nodes that control the order of execution, error handling, and shared state management.
* **Actions:** Outcomes from node execution that determine which successor node will execute next (default, named, or terminate).
* **Shared State:** A common context (typically a HashMap of JSON values) that enables nodes to exchange data.
* **SyncNode vs AsyncNode:** Two execution models for different use cases, with corresponding flow orchestrators.

## Advanced Usage

### Conditional Branching

```rust
// Create a decision node that returns different actions based on conditions
let decision = node(DecisionNode::new("Decision"));

// Connect branches based on returned actions
when(&decision, "option1").then(option1_node.clone());
when(&decision, "option2").then(option2_node.clone());
```

### Retries and Error Handling

```rust
// Create a node with a retry policy
let node = create_node!("ApiCall",
    exec: |_| {
        // Potentially failing operation
        Ok(json!("success"))
    },
    max_retries: 3,
    wait_duration: std::time::Duration::from_secs(1)
);
```

### Custom Node Implementation

```rust
pub struct CustomNode {
    base: BaseNode,
    // Additional custom fields
}

impl Node for CustomNode {
    // Implement required methods for the Node trait
}

impl SyncNode for CustomNode {
    fn prep(&mut self, shared: &mut Shared) -> NodeResult<Value> {
        // Custom preparation logic
        Ok(Value::Null)
    }
    
    fn exec(&mut self, prep_res: &Value) -> NodeResult<Value> {
        // Custom execution logic
        Ok(Value::Null)
    }
    
    fn post(&mut self, shared: &mut Shared, prep_res: &Value, exec_res: &Value) -> NodeResult<Value> {
        // Custom post-processing logic
        Ok(Value::Null)
    }
}
```

## Best Practices

### Environment Setup

- **Environment Variables**: Always use a `.env` file with the `dotenv` crate for configuration
- **API Key Management**: Load and validate API keys early in the application lifecycle
- **Tracing**: Configure proper tracing to help diagnose flow execution issues

### Error Handling

- Add appropriate timeouts when waiting for external services
- Include fallback behavior when external services (like MCP servers) are unavailable
- When using tool integrations, implement proper error handling for API calls

### Working with JSON Values

When processing numeric values with the JSON library, be aware that numbers can be represented as either integers or floats. For robust comparisons:

```rust
// Instead of direct equality which may fail due to different number representations:
assert_eq!(result, json!(20));  // May fail if result is 20.0

// Use a more flexible approach:
assert!(
    (result.as_i64() == Some(20)) || 
    (result.as_f64() == Some(20.0))
);
```

### Type Annotations in Closures

Always include type annotations in closures to avoid compilation errors:

```rust
let node = node_impl! {
    name: "ProcessingNode",
    exec: |data: &Value| {  // Type annotation is important
        // Processing logic
        Ok(json!(result))
    }
};
```

### Cloning Nodes in Flow Connections

When connecting nodes in a flow, especially with the branching pattern, remember to clone the node references when using them multiple times:

```rust
let flow = flow! {
    name: "BranchingFlow",
    start: decision_node.clone(),
    connections: [
        (decision_node.clone(), "path1", path1_node.clone()),
        (decision_node.clone(), "path2", path2_node.clone()),
        (path1_node.clone(), "default", end_node.clone())
    ]
};
```

This is necessary because `Arc<Mutex<>>` values are moved when used, so cloning ensures you can reuse the references.

## MCP Integration Troubleshooting

If you encounter issues with the MCP integration, here are some common problems and solutions:

### Connection Issues

1. **Verify MCP Server Availability**:
   - For local MCP servers, check that the server is running and accessible
   - When using custom MCP servers, verify they're running on the expected port

2. **MCP Server vs. MCP Tool Differences**:
   - Note that tools like `tavily-mcp` may run over stdio, not a network port
   - Ensure your application configures the correct connection method (stdio vs. HTTP)

3. **Environment Configuration**:
   - Use the `.env` file and the `dotenv` crate to manage API keys and server URLs
   - Set `MCP_SERVER_URL` for HTTP connections or configure stdio appropriately

### Common Errors

1. **No MCP Output**:
   - If `shared.get("mcp_output")` returns none, check if the MCP server is responding
   - Add a wait time after MCP node execution to give servers time to respond

2. **Node Environment Capture Issues**:
   - When using closures in node definitions that capture environment variables, use hardcoded values or `move |...| {}` closures
   - The `node_impl!` and macros have limitations with captured environment variables

3. **API Key Configuration**:
   - Always check that API keys are properly loaded before initiating MCP connections
   - Use `dotenv().ok();` at the start of your application to load environment variables

## License

RPocketFlow is licensed under the MIT License.

## Contributing

Contributions are welcome! To contribute:

1. Fork the repository.  
2. Create your feature branch (e.g., `git checkout -b feature/amazing-feature`).  
3. Commit your changes (e.g., `git commit -m 'Add some amazing feature'`).  
4. Push to your branch (e.g., `git push origin feature/amazing-feature`).  
5. Open a Pull Request.

Your contributions and feedback are greatly appreciated!
