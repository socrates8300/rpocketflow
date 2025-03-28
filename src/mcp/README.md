# MCP Integration for RPocketFlow

This module provides integration with Anthropic's Model Context Protocol (MCP) for the RPocketFlow library. It allows you to create workflows that interact with Claude models, supporting both simple text interactions and complex tool-based exchanges.

## Features

- Communication with Claude models via Anthropic API
- Support for conversation history management
- Tool integration with function calling capabilities
- Seamless integration with RPocketFlow's node-based architecture

## Setup

First, add the necessary dependencies to your `Cargo.toml`:

```toml
[dependencies]
rpocketflow = "0.1.0"
anthropic = "0.0.8"
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

Make sure you have an Anthropic API key available.

## Usage

### Basic MCP Node

```rust
use rpocketflow::*;
use serde_json::json;
use std::collections::HashMap;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment
    let api_key = env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");
    
    // Initialize MCP config
    let mcp_config = McpConfig::new(api_key, Models::CLAUDE_3_SONNET)
        .with_system_prompt("You are a helpful assistant.")
        .with_max_tokens(1000);
    
    // Create MCP node
    let mcp_node = mcp_node("ClaudeNode", mcp_config);
    
    // Create and run a flow
    let flow = async_flow! {
        name: "SimpleClaudeFlow",
        nodes: [mcp_node]
    };
    
    // Initialize shared state with user input
    let mut shared = HashMap::new();
    shared.insert("mcp_input".to_string(), json!("What is the capital of France?"));
    
    // Run the flow
    let result = flow.orchestrate(&mut shared, None).await?;
    
    // Print the response
    if let Some(output) = shared.get("mcp_output") {
        println!("Claude's response: {}", output);
    }
    
    Ok(())
}
```

### Adding Tools

```rust
use rpocketflow::*;
use rpocketflow::mcp::tools::{string_param, Tool, ToolRegistry};
use serde_json::json;

// Create a tool registry
let mut registry = ToolRegistry::new();

// Create and register a simple calculator tool
let calculator_tool = Tool::new(
    "calculate",
    "Perform a mathematical calculation",
    json!({
        "type": "object",
        "properties": {
            "expression": string_param("The mathematical expression to evaluate")
        },
        "required": ["expression"]
    })
).with_handler(|args| {
    let expression = args.get("expression")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    // This is a very simplified example - in a real app you would use a proper expression evaluator
    let result = 42; // Placeholder for actual calculation
    Ok(json!({ "result": result }))
});

registry.register(calculator_tool);

// Now you can use these tools with your MCP node
// ...
```

## Advanced Usage

For more complex scenarios, refer to the `examples/mcp_example.rs` file which demonstrates:

- A multi-node workflow with user input and output nodes
- Tool integration with a weather API example
- Conversation state management
- Flow control based on user responses

## MCP Node States

The MCP node manages several states in the shared state object:

| Key | Description |
|-----|-------------|
| `mcp_input` | Input message to send to the model |
| `mcp_output` | Text response from the model |
| `mcp_history` | Conversation history |
| `mcp_tools` | (Optional) Tool registry for function calling |

## Error Handling

MCP node errors are returned through the standard RPocketFlow error handling mechanisms. Common errors include API authentication issues, rate limiting, and malformed requests.