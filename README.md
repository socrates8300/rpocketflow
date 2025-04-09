# RPocketFlow

A lightweight, flexible workflow orchestration library for Rust that simplifies the creation and execution of complex agent-based workflows with minimal overhead.

## Overview

RPocketFlow supports both synchronous and asynchronous execution models, enabling you to build modular workflows with interconnected nodes. Whether you're implementing linear pipelines, branching paths, or complex state machines, RPocketFlow provides a robust framework that emphasizes clarity, minimal boilerplate, and efficient error handling.

## Features

RPocketFlow offers a modular node-based architecture, making it easy to build and reuse workflow components. It supports both synchronous and asynchronous operations (using Tokio), intuitive flow control with branching and retry mechanisms, and shared state management for data passing between nodes. The builder-pattern API further minimizes boilerplate and helps you focus on your business logic, all while keeping dependencies to a minimum.

Additionally, RPocketFlow includes integration with Anthropic's Model Context Protocol (MCP), making it easy to build AI-powered workflows using Claude models. The MCP integration supports:

- Text-based conversations with Claude
- Tool usage and function calling
- Conversation history management
- Seamless integration with RPocketFlow's node-based architecture

## Installation

Add RPocketFlow to your `Cargo.toml`:

```toml
[dependencies]
rpocketflow = "0.1.0"
```

Also ensure you have the required dependencies for extended features:

```toml
[dependencies]
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }  # Required for async features
async-trait = "0.1"  # Required for asynchronous trait support
```

For MCP integration with Claude, add these dependencies:

```toml
[dependencies]
anthropic = "0.0.8"  # Official Anthropic API client
serde = { version = "1.0", features = ["derive"] }
```

## Quick Start

### Basic Synchronous Flow

Below is an example of a simple synchronous flow using custom nodes:

```rust
use rpocketflow::*;
use serde_json::Value;
use std::collections::HashMap;

// Define a custom node that prints a message
struct PrintNode {
    base: BaseNode,
    message: String,
}

impl PrintNode {
    fn new(name: impl Into<String>, message: impl Into<String>) -> Self {
        PrintNode {
            base: BaseNode::new(name),
            message: message.into(),
        }
    }
}

impl Node for PrintNode {
    fn get_params(&self) -> &Params { &self.base.params }
    fn set_params(&mut self, params: Params) { self.base.params = params; }
    fn add_successor(&mut self, action: String, successor: NodeRef) { self.base.add_successor(action, successor); }
    fn get_successors(&self) -> &std::collections::HashMap<String, NodeRef> { &self.base.successors }
    fn get_successors_mut(&mut self) -> &mut std::collections::HashMap<String, NodeRef> { &mut self.base.successors }
    fn get_name(&self) -> &str { &self.base.name }
}

impl SyncNode for PrintNode {
    fn exec(&mut self, _prep_res: &Value) -> NodeResult<Value> {
        println!("{}: {}", self.get_name(), self.message);
        Ok(Value::Null)
    }
}

fn main() {
    // Create nodes
    let node1 = node(PrintNode::new("First", "Hello from node 1"));
    let node2 = node(PrintNode::new("Second", "Hello from node 2"));
    let node3 = node(PrintNode::new("Third", "Hello from node 3"));
    
    // Connect nodes sequentially
    then(&node1, node2.clone());
    then(&node2, node3.clone());
    
    // Create and run the flow
    let flow = Flow::new("HelloFlow", node1);
    let mut shared = std::collections::HashMap::new();
    
    match flow.orchestrate(&mut shared, None) {
        Ok(_) => println!("Flow completed successfully"),
        Err(e) => println!("Flow failed: {}", e),
    }
}
```

### Asynchronous Flow

For workflows requiring asynchronous execution, consider this example:

```rust
use rpocketflow::*;
use serde_json::Value;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use async_trait::async_trait;

struct AsyncPrintNode {
    base: BaseNode,
    message: String,
    delay_ms: u64,
}

impl AsyncPrintNode {
    fn new(name: impl Into<String>, message: impl Into<String>, delay_ms: u64) -> Self {
        AsyncPrintNode {
            base: BaseNode::new(name),
            message: message.into(),
            delay_ms,
        }
    }
}

impl Node for AsyncPrintNode {
    fn get_params(&self) -> &Params { &self.base.params }
    fn set_params(&mut self, params: Params) { self.base.params = params; }
    fn add_successor(&mut self, action: String, successor: NodeRef) { self.base.add_successor(action, successor); }
    fn get_successors(&self) -> &std::collections::HashMap<String, NodeRef> { &self.base.successors }
    fn get_successors_mut(&mut self) -> &mut std::collections::HashMap<String, NodeRef> { &mut self.base.successors }
    fn get_name(&self) -> &str { &self.base.name }
}

impl SyncNode for AsyncPrintNode {}

#[async_trait]
impl AsyncNode for AsyncPrintNode {
    async fn exec_async(&mut self, _prep_res: &Value) -> NodeResult<Value> {
        sleep(Duration::from_millis(self.delay_ms)).await;
        println!("{}: {}", self.get_name(), self.message);
        Ok(Value::Null)
    }
}

#[tokio::main]
async fn main() {
    // Create async nodes
    let node1 = node(AsyncPrintNode::new("First", "Hello from async node 1", 100));
    let node2 = node(AsyncPrintNode::new("Second", "Hello from async node 2", 200));
    let node3 = node(AsyncPrintNode::new("Third", "Hello from async node 3", 150));
    
    // Connect nodes sequentially
    then(&node1, node2.clone());
    then(&node2, node3.clone());
    
    // Create and run the async flow
    let flow = AsyncFlow::new("HelloAsyncFlow", node1);
    let mut shared = HashMap::new();
    
    match flow.orchestrate(&mut shared, None).await {
        Ok(_) => println!("Async flow completed successfully"),
        Err(e) => println!("Async flow failed: {}", e),
    }
}
```

## MCP Integration: Using Claude AI in Workflows

RPocketFlow makes it easy to incorporate Claude AI models into your workflows using the MCP (Model Context Protocol) integration. Here's how to create a simple conversational agent:

```rust
use rpocketflow::*;
use serde_json::json;
use std::collections::HashMap;
use std::env;

fn main() {
    // Get Anthropic API key from environment
    let api_key = env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");
    
    // Create a config for Claude
    let mcp_config = McpConfig::new(api_key, Models::CLAUDE_3_HAIKU)
        .with_system_prompt("You are a helpful assistant specialized in Rust programming.")
        .with_max_tokens(1000)
        .with_temperature(0.7);
    
    // Create an MCP node to interact with Claude
    let mcp_node = mcp_node("ClaudeNode", mcp_config);
    
    // Create a simple input and output node
    let input_node = node_impl! {
        name: "UserInput",
        exec: |_| {
            Ok(json!("What are the advantages of using Rust over C++?"))
        },
        post: |shared, _, exec_res| {
            shared.insert("mcp_input".to_string(), exec_res.clone());
            Ok(json!("default"))
        }
    };
    
    let output_node = node_impl! {
        name: "ResponseOutput",
        exec: |_| Ok(json!(null)),
        post: |shared, _, _| {
            if let Some(output) = shared.get("mcp_output") {
                println!("Claude's response:\n{}", output);
            }
            Ok(json!("terminate"))
        }
    };
    
    // Connect the nodes in a flow
    let flow = flow! {
        name: "ClaudeQAFlow",
        start: input_node.clone(),
        connections: [
            (input_node.clone(), "default", mcp_node.clone()),
            (mcp_node.clone(), "default", output_node.clone())
        ]
    };
    
    // Run the flow
    let mut shared = HashMap::new();
    match flow.orchestrate(&mut shared, None) {
        Ok(_) => println!("Flow completed successfully"),
        Err(e) => println!("Flow failed: {}", e),
    }
}
```

### Using Tool Calling with Claude

You can enhance Claude's capabilities by providing tools that it can call:

```rust
use rpocketflow::*;
use rpocketflow::mcp::tools::{Tool, ToolRegistry, string_param};
use serde_json::json;
use std::collections::HashMap;

// Create a tool registry
let mut registry = ToolRegistry::new();

// Create a weather tool
let weather_tool = Tool::new(
    "get_weather",
    "Get the current weather for a location",
    json!({
        "type": "object",
        "properties": {
            "location": string_param("The city and state, e.g. San Francisco, CA")
        },
        "required": ["location"]
    })
).with_handler(|args| {
    let location = args.get("location")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    
    // In a real implementation, you would call a weather API here
    Ok(json!({
        "temperature": 72,
        "condition": "sunny",
        "location": location
    }))
});

// Register the tool
registry.register(weather_tool);

// Now you can provide this tool registry to Claude
// (See the full example in examples/mcp_example.rs)
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

### Quick MCP Node Creation with `mcp_simple!`

```rust
use rpocketflow::*;
use std::env;

// Get API key from environment
let api_key = env::var("ANTHROPIC_API_KEY").expect("API key missing");

// Create an MCP node with minimal configuration
let claude = mcp_simple!("ClaudeAssistant", api_key, Models::CLAUDE_3_HAIKU);

// With system prompt
let claude_with_prompt = mcp_simple!(
    "ClaudeAssistant", 
    api_key, 
    Models::CLAUDE_3_SONNET,
    "You are a helpful assistant specialized in Rust programming."
);

// With full configuration
let claude_full = mcp_simple!(
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

### Registering Multiple Tools with `register_tools!`

```rust
use rpocketflow::*;
use serde_json::json;

// Create a tool registry with multiple tools
let registry = register_tools! {
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
let mcp_node = mcp_simple!("ClaudeNode", api_key, Models::CLAUDE_3_HAIKU, 
    "You are a helpful assistant", 1000);
```

## MCP Integration: Troubleshooting

If you encounter issues with the MCP integration, here are some common problems and solutions:

### Connection Issues

If you're having trouble connecting to MCP servers:

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
   - The `node_impl!` and `async_node_impl!` macros have limitations with captured environment variables

3. **API Key Configuration**:
   - Always check that API keys are properly loaded before initiating MCP connections
   - Use `dotenv().ok();` at the start of your application to load environment variables

## Core Concepts

At its core, RPocketFlow is built around the following ideas:

• **Nodes:** The basic building blocks that execute specific tasks and may include preparation, execution, and post-processing phases.  
• **Flows:** Structured networks of nodes that control the order of execution, error handling, and shared state management.  
• **Actions:** Outcomes from node execution that determine which successor node will execute next (default, named, or termination).  
• **Shared State:** A common context (typically a HashMap of JSON values) that enables nodes to exchange data.

## Advanced Usage

For more complex scenarios, RPocketFlow supports advanced techniques:

*Conditional Branching:*

```rust
// Create a decision node that returns different actions based on conditions
let decision = node(DecisionNode::new("Decision"));

// Connect branches based on returned actions
when(&decision, "option1").then(option1_node.clone());
when(&decision, "option2").then(option2_node.clone());
```

*Retries and Error Handling:*

```rust
// Create a node with a retry policy
let node = node(ApiCallNode::new("ApiCall")
    .with_max_retries(3)
    .with_wait_duration(Duration::from_secs(1)));
```

*Custom Node Implementation:*

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

## Best Practices for RPocketFlow Applications

### Environment Setup

- **Environment Variables**: Always use a `.env` file with the `dotenv` crate for configuration
- **API Key Management**: Load and validate API keys early in the application lifecycle
- **Logging**: Configure proper logging to help diagnose flow execution issues

### Error Handling

- Add appropriate timeouts when waiting for external services
- Include fallback behavior when external services (like MCP servers) are unavailable
- When using tool integrations, implement proper error handling for API calls

### Testing MCP Integrations

Before building complex workflows with MCP:

1. Create a simple test application to verify MCP server connectivity
2. Test individual tools separately to ensure they work as expected
3. Start with minimal flows and add complexity incrementally

## Alternative Implementation: Direct API Integration

If you're experiencing issues with MCP tool integration or prefer direct API control, you can implement your own API clients within RPocketFlow:

```rust
use rpocketflow::*;
use serde_json::json;
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::env;
use dotenv::dotenv;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv().ok();

    // Get API key
    let api_key = env::var("WEATHER_API_KEY")
        .expect("API key must be set in .env file");

    // Create a client wrapper closure
    let weather_client = move |location: &str| -> Result<serde_json::Value, String> {
        let url = format!(
            "https://api.example.com/weather?location={}&appid={}",
            location, api_key
        );

        match Client::new().get(&url).send() {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>() {
                        Ok(data) => Ok(data),
                        Err(e) => Err(format!("Failed to parse response: {}", e))
                    }
                } else {
                    Err(format!("API error: {}", response.status()))
                }
            },
            Err(e) => Err(format!("Request failed: {}", e))
        }
    };

    // Create RPocketFlow nodes
    let input_node = node_impl! {
        name: "InputNode",
        exec: |_| {
            // In a real app, get this from user input
            Ok(json!("New York"))
        }
    };

    // Create API node with the client
    let api_key_for_node = api_key.clone(); // Clone for capture
    let weather_node = node_impl! {
        name: "WeatherNode",
        exec: |input| {
            let location = input.as_str().unwrap_or("unknown");

            // Direct API call (simplified example)
            let url = format!(
                "https://api.example.com/weather?location={}&appid={}",
                location, api_key_for_node
            );

            match reqwest::blocking::get(&url) {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<serde_json::Value>() {
                            Ok(data) => Ok(data),
                            Err(e) => Err(format!("Failed to parse response: {}", e))
                        }
                    } else {
                        Err(format!("API error: {}", response.status()))
                    }
                },
                Err(e) => Err(format!("Request failed: {}", e))
            }
        }
    };

    let output_node = node_impl! {
        name: "OutputNode",
        exec: |weather_data| {
            println!("Weather data: {}", weather_data);
            Ok(json!(null))
        }
    };

    // Connect nodes
    let flow = flow! {
        name: "WeatherFlow",
        start: input_node.clone(),
        connections: [
            (input_node.clone(), "default", weather_node.clone()),
            (weather_node.clone(), "default", output_node.clone())
        ]
    };

    // Run the flow
    let mut shared = HashMap::new();
    match flow.orchestrate(&mut shared, None) {
        Ok(_) => println!("Flow completed successfully"),
        Err(e) => eprintln!("Flow failed: {}", e),
    }

    Ok(())
}
```

## Architecture

RPocketFlow is structured around key components:

• **Core Types:** Definitions such as `Params`, `Shared`, `NodeRef`, and `NodeResult` underpin the system.  
• **Flow Control:** The `Action` enum (with options like Default, Named, and Terminate) determines how workflows progress.  
• **Node Traits:** The `Node` trait is extended by `SyncNode` for synchronous operations and `AsyncNode` for asynchronous tasks, with `BaseNode` providing a default implementation.  
• **Workflow Orchestration:** `Flow` and `AsyncFlow` manage the execution order, shared state, and error recovery across nodes.

## Important Usage Notes

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

### Working with JSON Numbers

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
