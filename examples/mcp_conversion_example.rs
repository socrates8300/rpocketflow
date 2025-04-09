//! Example showcasing MCP conversation management
//!
//! This example demonstrates how to use the enhanced MCP conversation features
//! to manage multi-turn interactions with Claude models.

use rpocketflow::*;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    info!("Starting MCP conversation example");
    
    // Get API key from environment
    let api_key = env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");
    
    // Create tool registry for weather and math functions
    let tools = mcp_tools! {
        mcp_tool!("get_weather", "Get weather information for a location", [
            ("location", "The city and state/country")
        ], |args| {
            let location = args["location"].as_str().unwrap_or("unknown");
            info!("Tool called: get_weather for {}", location);
            Ok(json!({
                "temperature": 72,
                "condition": "sunny",
                "humidity": 45,
                "location": location
            }))
        }),
        
        mcp_tool!("calculate", "Calculate a math expression", [
            ("expression", "The mathematical expression to evaluate")
        ], |args| {
            let expr = args["expression"].as_str().unwrap_or("0");
            info!("Tool called: calculate with expression {}", expr);
            
            // For simplicity, let's just handle basic operations
            let result = match expr {
                "1+1" => 2,
                "2*3" => 6,
                "10/2" => 5,
                _ => 42, // Default answer for this example
            };
            
            Ok(json!({
                "result": result,
                "expression": expr
            }))
        })
    };
    
    // Create MCP config
    let mcp_config = McpConfig::new(api_key, Models::CLAUDE_3_HAIKU)
        .with_system_prompt("You are a helpful assistant with access to weather and calculator tools. Use them when appropriate.")
        .with_max_tokens(1000)
        .with_temperature(0.7);
    
    // Create MCP node with tools
    let mut mcp_node = McpNode::new("Claude", mcp_config)
        .with_tool_registry(tools)
        .with_max_conversation_length(10); // Limit conversation length
    
    // Interactive conversation loop
    println!("Starting conversation with Claude. Type 'exit' to quit.");
    
    loop {
        print!("> ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let input = input.trim();
        if input.to_lowercase() == "exit" {
            break;
        }
        
        // Add user message to conversation
        mcp_node.add_user_message(input);
        
        // Execute the MCP node
        let mut shared = HashMap::new();
        
        // Store the result of exec_async
        match mcp_node.exec_async(&json!(null)).await {
            Ok(result) => {
                // Extract text response
                if let Some(text) = result.get("text") {
                    if let Some(text_str) = text.as_str() {
                        println!("Claude: {}", text_str);
                    }
                }
                
                // Check for tool results
                if let Some(tool_results) = result.get("tool_results") {
                    if let Some(tool_array) = tool_results.as_array() {
                        for tool_result in tool_array {
                            if let (Some(id), Some(name), Some(_args), Some(result)) = (
                                tool_result.get(0),
                                tool_result.get(1),
                                tool_result.get(2),
                                tool_result.get(3)
                            ) {
                                println!("Tool '{}' (ID: {}) returned: {}", 
                                    name.as_str().unwrap_or("unknown"),
                                    id.as_str().unwrap_or("unknown"),
                                    result);
                            }
                        }
                    }
                }
            },
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }
    
    info!("Conversation ended");
    Ok(())
}
