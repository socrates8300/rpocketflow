//! Example showcasing the improved macros in RPocketFlow
//!
//! This example demonstrates how to use the new macro syntax to create
//! workflows with less boilerplate code.

use rpocketflow::*;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    info!("Starting improved macros example");
    
    // Get API key from environment (for MCP examples)
    let api_key = env::var("ANTHROPIC_API_KEY").unwrap_or_else(|_| "demo_key".to_string());
    
    // Example 1: Simple node with improved syntax
    info!("Example 1: Creating a simple node");
    let simple_node = create_node!("SimpleNode", |_| {
        info!("Node executed!");
        Ok(json!("done"))
    });
    
    // Example 2: Sequential flow with simpler syntax
    info!("Example 2: Creating a sequential flow");
    let node1 = create_node!("Step1", |_| {
        info!("Step 1 executed");
        Ok(json!("continue"))
    });
    
    let node2 = create_node!("Step2", |_| {
        info!("Step 2 executed");
        Ok(json!("continue"))
    });
    
    let node3 = create_node!("Step3", |_| {
        info!("Step 3 executed");
        Ok(json!("done"))
    });
    
    let flow = sequential_flow!("SimpleFlow", node1, node2, node3);
    
    let mut shared = HashMap::new();
    flow.orchestrate(&mut shared, None)?;
    
    // Example 3: Decision node with simplified syntax
    info!("Example 3: Using a decision node");
    let decision = decide!("RouteDecision", |_, shared| {
        if let Some(score) = shared.get("score") {
            if score.as_f64().unwrap_or(0.0) > 0.5 {
                "high_path"
            } else {
                "low_path"
            }
        } else {
            "default"
        }
    });
    
    let high_path = create_node!("HighPath", 
        exec: |_| {
            info!("Taking high path");
            Ok(json!("done"))
        }
    );
    
    let low_path = create_node!("LowPath", 
        exec: |_| {
            info!("Taking low path");
            Ok(json!("done"))
        }
    );
    
    // Connect decision node to paths
    when(&decision, "high_path").then(high_path.clone());
    when(&decision, "low_path").then(low_path.clone());
    when(&decision, "default").then(low_path.clone()); // Default fallback
    
    // Execute with different inputs
    let mut shared1 = HashMap::new();
    shared1.insert("score".to_string(), json!(0.7));
    decision.lock().unwrap().run(&mut shared1)?;
    
    let mut shared2 = HashMap::new();
    shared2.insert("score".to_string(), json!(0.3));
    decision.lock().unwrap().run(&mut shared2)?;
    
    // Example 4: Processing pipeline
    info!("Example 4: Creating a data processing pipeline");
    let process = pipeline!("DataProcessor", 
        // Step 1: Parse input
        |data| {
            info!("Pipeline step 1: Parsing input");
            let input = data.as_object()
                .ok_or_else(|| "Expected object input".to_string())?;
            Ok(json!(input))
        },
        // Step 2: Transform data
        |data| {
            info!("Pipeline step 2: Transforming data");
            let input_value = data.get("value")
                .ok_or_else(|| "Missing 'value' field".to_string())?
                .as_i64().unwrap_or(0);
            Ok(json!(input_value * 2))
        },
        // Step 3: Format output
        |data| {
            info!("Pipeline step 3: Formatting output");
            let value = data.as_i64().unwrap_or(0);
            Ok(json!({
                "original": value / 2,
                "doubled": value,
                "status": "success"
            }))
        }
    );
    
    let mut shared = HashMap::new();
    shared.insert("input".to_string(), json!({"value": 21}));
    let mut node = process.lock().unwrap();
    let prep_res = node.prep(&mut shared)?;
    let result = node.exec(&prep_res)?;
    info!("Pipeline result: {}", result);
    
    // Example 5: MCP integration
    info!("Example 5: MCP integration");
    
    // Create tools for function calling
    let tools = mcp_tools! {
        mcp_tool!("get_weather", "Get weather for a location", [
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
            // Just a dummy result for the example
            Ok(json!({
                "result": 42,
                "expression": expr
            }))
        })
    };
    
    // Create a simple MCP node with tools
    let system_prompt = "You are a helpful assistant that provides concise answers.";
    let mcp_config = McpConfig::new(api_key, Models::CLAUDE_3_HAIKU)
        .with_system_prompt(system_prompt)
        .with_max_tokens(1000);
    
    let mcp = mcp_node_with_tools("Claude", mcp_config, tools);
    
    info!("Created MCP node with tools");
    
    // Example 6: Branching flow with simplified syntax
    info!("Example 6: Creating a branching flow");
    let start = create_node!("Start", |_| {
        info!("Start node executed");
        Ok(json!("path_a"))
    });
    
    let path_a = create_node!("PathA", |_| {
        info!("Path A executed");
        Ok(json!("done"))
    });
    
    let path_b = create_node!("PathB", |_| {
        info!("Path B executed");
        Ok(json!("done"))
    });
    
    let end = create_node!("End", |_| {
        info!("End node executed");
        Ok(json!("terminate"))
    });
    
    // This uses our new branching_flow! macro
    let bf = branching_flow!("BranchingFlow", start => {
        "path_a" => path_a => "default" => end,
        "path_b" => path_b => "default" => end
    });
    
    let mut shared = HashMap::new();
    bf.orchestrate(&mut shared, None)?;
    
    info!("All examples completed successfully!");
    Ok(())
}
