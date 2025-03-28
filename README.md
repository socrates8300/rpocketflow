### RPocketFlow

A lightweight, flexible workflow orchestration library for Rust that simplifies the creation and execution of complex agent-based workflows with minimal overhead.

#### Overview

RPocketFlow supports both synchronous and asynchronous execution models, enabling you to build modular workflows with interconnected nodes. Whether you're implementing linear pipelines, branching paths, or complex state machines, RPocketFlow provides a robust framework that emphasizes clarity, minimal boilerplate, and efficient error handling.

#### Features

RPocketFlow offers a modular node-based architecture, making it easy to build and reuse workflow components. It supports both synchronous and asynchronous operations (using Tokio), intuitive flow control with branching and retry mechanisms, and shared state management for data passing between nodes. The builder-pattern API further minimizes boilerplate and helps you focus on your business logic, all while keeping dependencies to a minimum.

Additionally, RPocketFlow includes two types of MCP integrations:

1. **Anthropic Claude Integration**: Direct integration with Anthropic's API for Claude models, supporting:
   - Text-based conversations with Claude
   - Tool usage and function calling
   - Conversation history management
   - Seamless integration with RPocketFlow's node-based architecture

2. **Model Context Protocol (MCP) Client**: Full MCP protocol client implementation that can connect to any MCP-compatible server, supporting:
   - Connection to standard MCP protocol servers
   - Tool discovery and execution 
   - Standardized communication between AI models and external tools
   - Integration with the full MCP ecosystem

#### Installation

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

For MCP integrations, add these dependencies:

```toml
[dependencies]
# For Claude API integration
anthropic = "0.0.8"  # Official Anthropic API client
serde = { version = "1.0", features = ["derive"] }

# For MCP protocol implementation
mcpr = "0.1.0"       # Model Context Protocol for Rust
env_logger = "0.10"  # Optional but recommended for logging
```

#### Quick Start

##### Basic Synchronous Flow

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

##### Asynchronous Flow

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

#### MCP Integration: Using Claude AI in Workflows

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

##### Using Tool Calling with Claude

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

#### MCP Protocol Client: Connecting to MCP Servers

RPocketFlow provides a full implementation of the Model Context Protocol (MCP), allowing you to connect to any MCP-compatible server. This enables you to leverage a wide variety of AI models and tools in your workflows.

Here's how to use the MCP Protocol client:

```rust
use rpocketflow::*;
use serde_json::json;
use std::collections::HashMap;
use log::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    
    // Create MCP client configuration
    let mcp_config = MCPClientConfig::new(
        "MyApplication", 
        "1.0.0"
    );
    
    // Create MCP protocol node
    let protocol_node = mcp_protocol_node("MCPClient", mcp_config);
    
    // Create input node
    let input_node = node_impl! {
        name: "UserInputNode",
        exec: |_: &serde_json::Value| -> NodeResult<serde_json::Value> {
            // In this example, we'll just call a specific tool
            Ok(json!({
                "tool_name": "echo",
                "params": {
                    "message": "Hello, MCP!"
                }
            }))
        },
        post: |shared: &mut Shared, _: &serde_json::Value, exec_res: &serde_json::Value| {
            shared.insert("mcp_tool_call".to_string(), exec_res.clone());
            Ok(json!("default"))
        }
    };
    
    // Create output node
    let output_node = node_impl! {
        name: "OutputNode",
        exec: |_: &serde_json::Value| {
            Ok(json!(null))
        },
        post: |shared: &mut Shared, _: &serde_json::Value, _: &serde_json::Value| {
            // Display the result from the MCP call
            if let Some(result) = shared.get("mcp_exec_result") {
                println!("\nMCP Result:");
                println!("{}", serde_json::to_string_pretty(result).unwrap());
            }
            
            Ok(json!("terminate"))
        }
    };
    
    // Define the flow
    let flow = flow! {
        name: "MCP Protocol Flow",
        start: input_node.clone(),
        connections: [
            (input_node.clone(), "default", protocol_node.clone()),
            (protocol_node.clone(), "success", output_node.clone()),
            (protocol_node.clone(), "error", output_node.clone())
        ]
    };
    
    // Initialize shared state
    let mut shared = HashMap::new();
    
    // Run the flow
    info!("Starting MCP Protocol flow...");
    match flow.orchestrate(&mut shared, None) {
        Ok(_) => info!("Flow completed successfully"),
        Err(e) => error!("Flow failed: {}", e),
    }
    
    Ok(())
}
```

##### Creating an MCP Server

You can create an MCP server using the `mcpr` crate. Here's a simple example:

```rust
use log::{error, info};
use mcpr::{
    error::MCPError,
    server::{Server, ServerConfig},
    transport::stdio::StdioTransport,
    Tool,
};
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), MCPError> {
    // Initialize logging
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    // Configure the server
    let server_config = ServerConfig::new()
        .with_name("My MCP Server")
        .with_version("1.0.0")
        .with_tool(Tool {
            name: "echo".to_string(),
            description: Some("Echo back the input".to_string()),
            input_schema: mcpr::schema::common::ToolInputSchema {
                r#type: "object".to_string(),
                properties: Some(
                    [
                        (
                            "message".to_string(),
                            json!({
                                "type": "string",
                                "description": "The message to echo"
                            }),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                ),
                required: Some(vec!["message".to_string()]),
            },
        });

    // Create the server
    let mut server = Server::new(server_config);

    // Register tool handlers
    server.register_tool_handler("echo", |params: Value| {
        let message = params
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MCPError::Protocol("Missing message".to_string()))?;

        info!("Echo handler called with message: {}", message);

        let response = json!({
            "echo": message
        });

        Ok(response)
    })?;

    // Create a transport
    let transport = StdioTransport::new();

    // Start the server
    info!("Starting MCP server...");
    server.start(transport)?;

    Ok(())
}
```

#### Using Stdio-based MCP Servers

Many MCP tools communicate over standard input/output (stdio) instead of network sockets. RPocketFlow provides convenient helpers for working with these stdio-based MCP servers:

```rust
use rpocketflow::*;
use serde_json::json;

// Create an MCP client config for a stdio-based server
let config = mcp_stdio_config(
    "My Application",  // Client name
    "1.0.0",           // Client version
    "./path/to/mcp_server_executable",  // Server executable path
    vec!["--option1", "value1"]         // Optional server arguments
);

// Create an MCP protocol node with this config
let mcp_node = mcp_protocol_node("MCP Client", config);

// Now you can use this node in your flow to interact with the stdio-based MCP server
```

When building an application that uses stdio-based MCP servers:

1. **Server Startup Management**: RPocketFlow automatically handles launching the server process
2. **Error Handling**: The protocol handles startup failures and connection issues
3. **Process Cleanup**: When your node is dropped, the server process is automatically terminated
4. **Bidirectional Communication**: Messages are sent to the server's stdin and received from stdout

This approach is especially useful for integrating with standalone MCP tools like database connectors, code analyzers, or specialized APIs that are packaged as separate executables.

#### MCP Integration: Troubleshooting

If you encounter issues with the MCP integration, here are some common problems and solutions:

##### Connection Issues

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

##### Common Errors

1. **No MCP Output**:
   - If `shared.get("mcp_output")` returns none, check if the MCP server is responding
   - Add a wait time after MCP node execution to give servers time to respond

2. **Node Environment Capture Issues**:
   - When using closures in node definitions that capture environment variables, use hardcoded values or `move |...| {}` closures
   - The `node_impl!` and `async_node_impl!` macros have limitations with captured environment variables

3. **API Key Configuration**:
   - Always check that API keys are properly loaded before initiating MCP connections
   - Use `dotenv().ok();` at the start of your application to load environment variables

#### Macros for Simplified Usage

RPocketFlow includes several macros that help reduce boilerplate and streamline common tasks when defining nodes and flows.

##### Node Creation Macros

*Using `node_impl!` for Synchronous Nodes:*

```rust
// Create a simple node with just an execution function
let log_node = node_impl! {
    name: "Logger",
    exec: |_prep_res| {
        println!("Log message");
        Ok(Value::Null)
    }
};

// Create a more complex node with all lifecycle hooks
let processor = node_impl! {
    name: "DataProcessor",
    prep: |shared| {
        // Prepare data
        Ok(shared.get("input").unwrap_or(&Value::Null).clone())
    },
    exec: |prep_res| {
        // Process data
        let mut data = prep_res.clone();
        // ... processing logic
        Ok(data)
    },
    post: |shared, _prep_res, exec_res| {
        // Store results
        shared.insert("output".to_string(), exec_res.clone());
        Ok(Value::Null)
    },
    max_retries: 3,
    wait_duration: Duration::from_millis(100)
};
```

*Using `async_node_impl!` for Asynchronous Nodes:*

```rust
// Create an async node with async execution
let async_processor = async_node_impl! {
    name: "AsyncProcessor",
    exec_async: async |_prep_res| {
        // Simulate async work
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(Value::String("processed".to_string()))
    },
    max_retries: 3
};
```

##### Flow Creation Macros

*Defining Synchronous Flows with `flow!`:*

```rust
// Create a simple linear flow
let simple_flow = flow! {
    name: "SimpleFlow",
    nodes: [node1, node2, node3]
};

// Create a branching flow
let branching_flow = flow! {
    name: "BranchingFlow",
    start: decision_node,
    connections: [
        (decision_node, "path1", path1_node),
        (decision_node, "path2", path2_node),
        (path1_node, "default", end_node),
        (path2_node, "default", end_node)
    ]
};
```

*Defining Asynchronous Flows with `async_flow!`:*

```rust
// Create an async flow
let async_flow = async_flow! {
    name: "AsyncProcessingFlow",
    nodes: [fetch_node, process_node, save_node]
};
```

#### Core Concepts

At its core, RPocketFlow is built around the following ideas:

• **Nodes:** The basic building blocks that execute specific tasks and may include preparation, execution, and post-processing phases.  
• **Flows:** Structured networks of nodes that control the order of execution, error handling, and shared state management.  
• **Actions:** Outcomes from node execution that determine which successor node will execute next (default, named, or termination).  
• **Shared State:** A common context (typically a HashMap of JSON values) that enables nodes to exchange data.

#### Advanced Usage

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

## Model Context Protocol (MCP) Integration Guide

RPocketFlow provides robust support for the Model Context Protocol (MCP), with two complementary implementations:

1. **Direct Claude Integration** - Built-in integration with Anthropic's Claude AI models
2. **Full Protocol Implementation** - Complete MCP client implementation that works with any MCP-compatible server

### What is Model Context Protocol?

Model Context Protocol (MCP) is an open standard designed to connect AI assistants with external tools, data sources, and services. It establishes a bidirectional communication channel through which models can:

1. Discover available tools and capabilities
2. Call external tools with structured parameters
3. Receive structured responses from those tools
4. Integrate external data into their context and reasoning

### Implementation Overview

| Feature | Claude Integration | Protocol Implementation |
|---------|-------------------|------------------------|
| **Integration Type** | Direct API | Protocol-level |
| **Compatibility** | Claude models only | Any MCP server |
| **Setup Complexity** | Simple (API key) | Moderate (server required) |
| **Function Calling** | Built-in | Server-dependent |
| **Server Management** | None needed | Automatic or manual |
| **Use Case** | AI text generation | Custom tool integration |

### Setting Up an MCP Client

To use the MCP implementation in RPocketFlow, follow these steps:

#### Option 1: Claude Direct Integration

```rust
use rpocketflow::*;
use serde_json::json;
use std::env;

// Use the Claude macro for the simplest setup
let claude_node = claude_node_macro! {
    name: "Claude",
    api_key: env::var("ANTHROPIC_API_KEY").expect("API key required"),
    model: Models::CLAUDE_3_HAIKU,
    system_prompt: "You are a helpful assistant.",
    max_tokens: 1000
};

// Create a simple flow with input and output nodes
let input_node = node_impl! {
    name: "Input",
    exec: |_| Ok(json!("Tell me about Rust programming.")),
    post: |shared, _, exec_res| {
        shared.insert("mcp_input".to_string(), exec_res.clone());
        Ok(json!("default"))
    }
};

let output_node = node_impl! {
    name: "Output",
    post: |shared, _, _| {
        if let Some(output) = shared.get("mcp_output") {
            println!("Claude says: {}", output);
        }
        Ok(json!("default"))
    }
};

// Create and run the flow
let flow = flow! {
    name: "Claude Flow",
    nodes: [input_node, claude_node, output_node]
};
```

#### Option 2: Protocol-Level Implementation

```rust
use rpocketflow::*;
use serde_json::json;

// Create MCP client configuration
let mcp_config = MCPClientConfig::new(
    "My Application", 
    "1.0.0"
)
.with_server_command(
    "/path/to/mcp/server",  // Path to the MCP server executable
    vec!["--arg1", "--arg2"] // Optional arguments
);

// Create an MCP node for your flow
let mcp_client_node = mcp_protocol_node("MCP Client", mcp_config);
```

```rust
use rpocketflow::*;
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;

// Use the MCP Protocol macro for simple setup
let mcp_node = mcp_protocol_node_macro! {
    name: "MCP Client",
    server_command: "/path/to/mcp_server",
    server_args: ["--option", "value"]
};

// Create input node that specifies which tool to call
let input_node = node_impl! {
    name: "Input",
    exec: |_| Ok(json!({
        "tool_name": "echo",
        "params": {
            "message": "Hello, MCP Protocol!"
        }
    })),
    post: |shared, _, exec_res| {
        shared.insert("mcp_tool_call".to_string(), exec_res.clone());
        Ok(json!("default"))
    }
};

// Create output node to process results
let output_node = node_impl! {
    name: "Output",
    post: |shared, _, _| {
        if let Some(result) = shared.get("mcp_exec_result") {
            // Results are structured with status and tool info
            if let Some(status) = result.get("status").and_then(|s| s.as_str()) {
                match status {
                    "success" => {
                        let tool_result = result.get("result").unwrap();
                        println!("Tool succeeded: {}", tool_result);
                    },
                    "error" => {
                        let error = result.get("error").unwrap();
                        println!("Tool failed: {}", error);
                    },
                    _ => println!("Unknown status")
                }
            }
        }
        Ok(json!("default"))
    }
};

// Create and run the flow
let flow = flow! {
    name: "MCP Protocol Flow",
    start: input_node.clone(),
    connections: [
        (input_node.clone(), "default", mcp_node.clone()),
        (mcp_node.clone(), "success", output_node.clone()),
        (mcp_node.clone(), "error", output_node.clone())
    ]
};

// Run the flow
let mut shared = HashMap::new();
flow.orchestrate(&mut shared, None)?;
```

### Creating Tool Integrations

RPocketFlow makes it easy to create tool integrations for MCP:

```rust
// Create MCP tool handlers with the dedicated macro
let weather_tool = mcp_tool_handler! {
    name: "WeatherTool",
    tool_name: "get_weather",
    handler: |params| {
        // Extract parameters
        let location = params.get("location")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
            
        // Call external API or service (simulated)
        let temp = 72; // Would come from API
        let condition = "sunny";
        
        // Return structured result
        Ok(json!({
            "temperature": temp,
            "condition": condition,
            "location": location
        }))
    }
};

// Use the tool in a flow
let flow = flow! {
    name: "Weather Flow",
    start: input_node.clone(),
    connections: [
        (input_node.clone(), "default", weather_tool.clone()),
        (weather_tool.clone(), "success", output_node.clone()),
        (weather_tool.clone(), "error", error_node.clone())
    ]
};
```

#### Creating an MCP Server

RPocketFlow also supports creating MCP-compatible servers. Here's how to implement a basic MCP server:

```rust
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // We'll use stdio for communication
    let stdin = std::io::stdin();
    let mut stdin = BufReader::new(stdin);
    let mut stdout = std::io::stdout();
    
    // Server information for initialization responses
    let server_info = json!({
        "jsonrpc": "2.0",
        "result": {
            "serverInfo": {
                "name": "My MCP Server",
                "version": "1.0.0"
            },
            "protocolVersion": "0.1",
            "capabilities": {
                "tools": ["my_tool"]
            }
        },
        "id": 1
    });
    
    // Main loop to process incoming messages
    let mut buffer = String::new();
    loop {
        // Read a line from stdin
        buffer.clear();
        if stdin.read_line(&mut buffer).unwrap() == 0 {
            break; // EOF reached
        }
        
        // Parse the message
        let message: Value = match serde_json::from_str(&buffer) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Failed to parse message: {}", e);
                continue;
            }
        };
        
        // Process message based on method
        let method = message.get("method").and_then(|m| m.as_str());
        let id = message.get("id").and_then(|i| i.as_u64()).unwrap_or(0);
        
        let response = match method {
            Some("initialize") => server_info.clone(),
            Some("tools/list") => {
                json!({
                    "jsonrpc": "2.0",
                    "result": {
                        "tools": [
                            {
                                "name": "my_tool",
                                "description": "Description of my tool"
                            }
                        ]
                    },
                    "id": id
                })
            },
            Some("my_tool") => {
                // Example tool implementation
                // Extract parameters and execute tool logic here
                json!({
                    "jsonrpc": "2.0",
                    "result": {
                        "output": "Tool executed successfully"
                    },
                    "id": id
                })
            },
            Some("shutdown") => {
                json!({
                    "jsonrpc": "2.0",
                    "result": null,
                    "id": id
                })
            },
            _ => {
                json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32601,
                        "message": "Method not found"
                    },
                    "id": id
                })
            }
        };
        
        // Send the response
        let response_str = serde_json::to_string(&response).unwrap();
        writeln!(stdout, "{}", response_str).unwrap();
        stdout.flush().unwrap();
        
        // If shutdown was called, exit the loop
        if method == Some("shutdown") {
            break;
        }
    }
    
    Ok(())
}
```

#### MCP Protocol Details

The MCP implementation in RPocketFlow follows these key protocol steps:

1. **Initialization**: The client connects to the server and exchanges capability information.
2. **Tool Discovery**: The client can request a list of available tools from the server.
3. **Tool Invocation**: The client can call specific tools with structured parameters.
4. **Shutdown**: The client can gracefully close the connection when done.

All communication uses JSON-RPC 2.0 format with these standard methods:

- `initialize`: Establishes the connection and exchanges capabilities.
- `tools/list`: Retrieves available tools from the server.
- `<tool_name>`: Calls a specific tool with parameters.
- `shutdown`: Closes the connection gracefully.

#### Advanced MCP Features

##### Custom Transport Layers

While the default implementation uses stdio for communication, you can implement custom transport layers for different communication channels:

```rust
// Example of using a custom command for an MCP server
let mcp_config = MCPClientConfig::new("MyApp", "1.0")
    .with_server_command("/path/to/server", vec!["--port", "8080"]);
```

##### Tool Error Handling

Proper error handling for MCP tool calls:

```rust
// In your flow's output node
match exec_res.get("status").and_then(|s| s.as_str()) {
    Some("success") => {
        // Process successful result
        let result = exec_res.get("result").unwrap();
        println!("Tool succeeded with result: {}", result);
    },
    Some("error") => {
        // Handle error
        let error = exec_res.get("error").unwrap();
        println!("Tool failed with error: {}", error);
    },
    _ => {
        println!("Unknown status in MCP response");
    }
}
```

##### Advanced Tool Parameters

Working with complex MCP tool parameters is straightforward:

```rust
// Complex nested parameters for database search
let search_node = node_impl! {
    name: "DatabaseSearch",
    exec: |_| {
        let complex_params = json!({
            "query": {
                "filters": [
                    {"field": "name", "op": "contains", "value": "test"},
                    {"field": "status", "op": "equals", "value": "active"}
                ],
                "sort": {"field": "created_at", "direction": "desc"},
                "limit": 10
            },
            "options": {
                "include_metadata": true,
                "format": "json"
            }
        });
        
        Ok(json!({
            "tool_name": "search_database",
            "params": complex_params
        }))
    },
    post: |shared, _, exec_res| {
        shared.insert("mcp_tool_call".to_string(), exec_res.clone());
        Ok(json!("default"))
    }
};
```

##### Complete Example: AI-Powered Data Analysis Flow

Here's a complete example combining Claude's AI capabilities with custom MCP tools:

```rust
use rpocketflow::*;
use serde_json::json;
use std::collections::HashMap;
use std::env;

// Create an AI assistant node
let claude = claude_node_macro! {
    name: "Claude",
    api_key: env::var("ANTHROPIC_API_KEY").expect("API key required"),
    model: Models::CLAUDE_3_SONNET,
    system_prompt: "You are a data analysis assistant."
};

// Create data processing tools
let data_tool = mcp_tool_handler! {
    name: "DataProcessor",
    tool_name: "process_data",
    handler: |params| {
        // Process data and return results
        Ok(json!({
            "processed_data": [1, 2, 3, 4, 5],
            "statistics": {
                "mean": 3.0,
                "median": 3,
                "std_dev": 1.58
            }
        }))
    }
};

// Create a flow with branching logic
let flow = flow! {
    name: "AI Data Analysis",
    start: input_node.clone(),
    connections: [
        // Route user query to Claude
        (input_node.clone(), "query", claude.clone()),
        
        // Route data processing requests to the data tool
        (input_node.clone(), "process", data_tool.clone()),
        
        // Send Claude's response to the output node
        (claude.clone(), "default", output_node.clone()),
        
        // Send data processing results to Claude for explanation
        (data_tool.clone(), "success", claude.clone()),
        
        // Error handling path
        (data_tool.clone(), "error", error_node.clone())
    ]
};
```

#### Comparing MCP Implementations in RPocketFlow

RPocketFlow offers two complementary MCP implementations for different use cases:

##### 1. Anthropic Claude Integration (`McpNode`)

This implementation provides direct integration with Anthropic's Claude AI models using their API:

- **Direct access** to Claude models via Anthropic's API
- Built-in support for Claude's function calling capabilities
- Simple setup using API keys
- No need for external servers or processes
- Limited to Anthropic's models and features

**Example usage:**

```rust
// Create config with API key
let mcp_config = McpConfig::new(
    "YOUR_ANTHROPIC_API_KEY", 
    Models::CLAUDE_3_SONNET
);

// Create a Claude MCP node
let claude_node = mcp_node("ClaudeNode", mcp_config);
```

##### 2. Protocol-Level Implementation (`MCPProtocolNode`)

This implementation provides a full protocol-level client that can connect to any MCP-compatible server:

- Works with **any MCP server** implementation
- Supports custom tools and services
- Can launch and manage server processes
- More flexible but requires an external server
- Compatible with the broader MCP ecosystem

**Example usage:**

```rust
// Create config with server command
let mcp_config = MCPClientConfig::new(
    "MyClient", 
    "1.0.0"
)
.with_server_command("/path/to/server", vec![]);

// Create a protocol-level MCP node
let mcp_node = mcp_protocol_node("MCPNode", mcp_config);
```

##### When to Use Each Implementation:

- Use `McpNode` (Claude integration) when:
  - You need direct access to Claude AI models
  - You want simple setup without external services
  - You're focusing on natural language tasks

- Use `MCPProtocolNode` (protocol-level) when:
  - You need to use custom MCP-compatible tools/servers
  - You want to integrate with non-Anthropic MCP servers
  - You're building a system with multiple specialized tools
  - You need more control over the MCP communication process

#### MCP Convenience Macros

RPocketFlow provides several macros to simplify working with MCP:

##### 1. Creating an MCP Protocol Client

```rust
// Using the macro (concise)
let mcp_node = mcp_protocol_node_macro! {
    name: "MCP Client",
    server_command: "/path/to/mcp_server",
    server_args: ["--port", "8080"],
    client_name: "My Client",
    client_version: "1.0.0"
};

// Equivalent without macro (verbose)
let config = MCPClientConfig::new("My Client", "1.0.0")
    .with_server_command("/path/to/mcp_server", vec!["--port".to_string(), "8080".to_string()]);
let mcp_node = mcp_protocol_node("MCP Client", config);
```

##### 2. Creating a Claude Node

```rust
// Using the macro (concise)
let claude_node = claude_node_macro! {
    name: "Claude Assistant",
    api_key: api_key,
    model: Models::CLAUDE_3_SONNET,
    system_prompt: "You are a helpful coding assistant.",
    max_tokens: 2000,
    temperature: 0.7
};

// Equivalent without macro (verbose)
let config = McpConfig::new(api_key, Models::CLAUDE_3_SONNET)
    .with_system_prompt("You are a helpful coding assistant.")
    .with_max_tokens(2000)
    .with_temperature(0.7);
let claude_node = mcp_node("Claude Assistant", config);
```

##### 3. Creating Tool Handler Nodes

```rust
// Using the macro (concise)
let weather_handler = mcp_tool_handler! {
    name: "WeatherHandler",
    tool_name: "get_weather",
    handler: |params| {
        let location = params.get("location")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
            
        // In real code, you would call a weather API here
        Ok(json!({
            "temperature": 72,
            "condition": "sunny",
            "location": location
        }))
    }
};

// The macro handles all the boilerplate of creating a node that:
// 1. Takes parameters from prep_res
// 2. Calls your handler function
// 3. Formats success/error responses correctly
// 4. Stores results in shared state using a consistent naming pattern
// 5. Returns appropriate actions based on execution status
```

### Best Practices for RPocketFlow Applications

#### Environment Setup

- **Environment Variables**: Always use a `.env` file with the `dotenv` crate for configuration
- **API Key Management**: Load and validate API keys early in the application lifecycle
- **Logging**: Configure proper logging to help diagnose flow execution issues

#### Error Handling

- Add appropriate timeouts when waiting for external services
- Include fallback behavior when external services (like MCP servers) are unavailable
- When using tool integrations, implement proper error handling for API calls

### Conclusion

RPocketFlow's MCP implementation provides a powerful way to integrate AI capabilities and external tools into your Rust applications. With both direct Claude integration and protocol-level MCP support, you have the flexibility to choose the approach that best fits your needs.

The convenience macros make it easy to set up MCP nodes, create tool handlers, and build complex flows with minimal boilerplate. Whether you're building a simple AI-powered application or a complex system with multiple specialized tools, RPocketFlow's MCP support has you covered.

For more examples and detailed API documentation, check out the [examples](./examples/) directory and the [API documentation](https://docs.rs/rpocketflow).

##### Testing MCP Integrations

Before building complex workflows with MCP:

1. Create a simple test application to verify MCP server connectivity
2. Test individual tools separately to ensure they work as expected
3. Start with minimal flows and add complexity incrementally

#### Alternative Implementation: Direct API Integration

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

#### Architecture

RPocketFlow is structured around key components:

• **Core Types:** Definitions such as `Params`, `Shared`, `NodeRef`, and `NodeResult` underpin the system.  
• **Flow Control:** The `Action` enum (with options like Default, Named, and Terminate) determines how workflows progress.  
• **Node Traits:** The `Node` trait is extended by `SyncNode` for synchronous operations and `AsyncNode` for asynchronous tasks, with `BaseNode` providing a default implementation.  
• **Workflow Orchestration:** `Flow` and `AsyncFlow` manage the execution order, shared state, and error recovery across nodes.

### Important Usage Notes

#### Cloning Nodes in Flow Connections

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

#### Working with JSON Numbers

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

#### Type Annotations in Closures

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

#### License

RPocketFlow is licensed under the MIT License.

#### Contributing

Contributions are welcome! To contribute:

1. Fork the repository.  
2. Create your feature branch (e.g., `git checkout -b feature/amazing-feature`).  
3. Commit your changes (e.g., `git commit -m 'Add some amazing feature'`).  
4. Push to your branch (e.g., `git push origin feature/amazing-feature`).  
5. Open a Pull Request.

Your contributions and feedback are greatly appreciated!
