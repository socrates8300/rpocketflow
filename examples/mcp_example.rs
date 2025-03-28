use rpocketflow::*;
use serde_json::json;
use std::collections::HashMap;
use std::env;

use rpocketflow::mcp::tools::{string_param, Tool, ToolRegistry};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    simple_logger::init_with_level(log::Level::Info)
        .map_err(|e| format!("Failed to initialize logger: {}", e))?;
    
    // Get API key from environment
    let api_key = env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");
    
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
        })
    ).with_handler(|args| {
        let location = args.get("location")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
            
        let unit = args.get("unit")
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
    
    // Create MCP nodes
    let input_node = node_impl! {
        name: "UserInputNode",
        exec: |_: &Value| -> NodeResult<Value> {
            println!("Ask about the weather in a specific location:");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)
                .map_err(|e| format!("Failed to read user input: {}", e))?;
            Ok(json!(input.trim()))
        },
        post: |shared: &mut Shared, _: &Value, exec_res: &Value| {
            shared.insert("mcp_input".to_string(), exec_res.clone());
            Ok(json!("default"))
        }
    };
    
    let mcp_node = mcp_node("ClaudeNode", mcp_config);
    
    let output_node = node_impl! {
        name: "OutputNode",
        exec: |_: &Value| {
            Ok(json!(null))
        },
        post: |shared: &mut Shared, _: &Value, _: &Value| -> NodeResult<Value> {
            if let Some(output) = shared.get("mcp_output") {
                if let Some(text) = output.as_str() {
                    println!("\nClaude's response:");
                    println!("{}\n", text);
                }
            }
            
            println!("Do you want to continue? (yes/no)");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)
                .map_err(|e| format!("Failed to read user input: {}", e))?;
            
            if input.trim().to_lowercase() == "yes" {
                Ok(json!("continue"))
            } else {
                Ok(json!("terminate"))
            }
        }
    };
    
    // Create the flow
    let flow = flow! {
        name: "MCP Conversation Flow",
        start: input_node.clone(),
        connections: [
            (input_node.clone(), "default", mcp_node.clone()),
            (mcp_node.clone(), "default", output_node.clone()),
            (output_node.clone(), "continue", input_node.clone())
        ]
    };
    
    // Initialize shared state
    let mut shared = HashMap::new();
    
    // Run the flow (synchronously, not async)
    match flow.orchestrate(&mut shared, None) {
        Ok(_) => println!("Flow completed successfully"),
        Err(e) => eprintln!("Flow failed: {}", e),
    }
    
    Ok(())
}