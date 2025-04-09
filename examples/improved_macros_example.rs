use rpocketflow::*;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment (for MCP examples)
    let api_key = env::var("ANTHROPIC_API_KEY").unwrap_or_else(|_| "demo_key".to_string());
    
    // Example 1: Simple node with improved syntax
    println!("Example 1: Creating a simple node");
    let simple_node = create_node!("SimpleNode", |_| {
        println!("Node executed!");
        Ok(json!("done"))
    });
    
    // Example 2: Sequential flow with simpler syntax
    println!("Example 2: Creating a sequential flow");
    let node1 = create_node!("Step1", |_| {
        println!("Step 1 executed");
        Ok(json!("continue"))
    });
    
    let node2 = create_node!("Step2", |_| {
        println!("Step 2 executed");
        Ok(json!("continue"))
    });
    
    let node3 = create_node!("Step3", |_| {
        println!("Step 3 executed");
        Ok(json!("done"))
    });
    
    let flow = sequential_flow!("SimpleFlow", node1, node2, node3);
    
    let mut shared = HashMap::new();
    flow.orchestrate(&mut shared, None)?;
    
    // Example 3: Decision node with simplified syntax
    println!("Example 3: Using a decision node");
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
            println!("Taking high path");
            Ok(json!("done"))
        }
    );
    
    let low_path = create_node!("LowPath", 
        exec: |_| {
            println!("Taking low path");
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
    println!("Example 4: Creating a data processing pipeline");
    let process = pipeline!("DataProcessor", 
        // Step 1: Parse input
        |data| {
            println!("Pipeline step 1: Parsing input");
            let input = data.as_object()
                .ok_or_else(|| "Expected object input".to_string())?;
            Ok(json!(input))
        },
        // Step 2: Transform data
        |data| {
            println!("Pipeline step 2: Transforming data");
            let input_value = data.get("value")
                .ok_or_else(|| "Missing 'value' field".to_string())?
                .as_i64().unwrap_or(0);
            Ok(json!(input_value * 2))
        },
        // Step 3: Format output
        |data| {
            println!("Pipeline step 3: Formatting output");
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
    println!("Pipeline result: {}", result);
    
    // Example 5: MCP integration (no actual API call)
    println!("Example 5: MCP integration");
    
    // Create tools for function calling
    let tools = register_tools! {
        mcp_tool!("get_weather", "Get weather for a location", {
            "location": "The city and state/country"
        }, |args| {
            let location = args["location"].as_str().unwrap_or("unknown");
            println!("Tool called: get_weather for {}", location);
            Ok(json!({
                "temperature": 72,
                "condition": "sunny",
                "humidity": 45,
                "location": location
            }))
        }),
        
        mcp_tool!("calculate", "Calculate a math expression", {
            "expression": "The mathematical expression to evaluate"
        }, |args| {
            let expr = args["expression"].as_str().unwrap_or("0");
            println!("Tool called: calculate with expression {}", expr);
            // Just a dummy result for the example
            Ok(json!({
                "result": 42,
                "expression": expr
            }))
        })
    };
    
    // In a real implementation, you would add these tools to your MCP node
    // and make actual API calls
    
    // Create a simple MCP flow
    let system_prompt = "You are a helpful assistant that provides concise answers.";
    let flow = mcp_flow!("SimpleConversation", api_key, Models::CLAUDE_3_HAIKU,
        system: system_prompt,
        max_tokens: 1000
    );
    
    println!("Created MCP flow: {}", flow.get_name());
    
    // Example 6: Branching flow with simplified syntax
    println!("Example 6: Creating a branching flow");
    let start = create_node!("Start", |_| {
        println!("Start node executed");
        Ok(json!("path_a"))
    });
    
    let path_a = create_node!("PathA", |_| {
        println!("Path A executed");
        Ok(json!("done"))
    });
    
    let path_b = create_node!("PathB", |_| {
        println!("Path B executed");
        Ok(json!("done"))
    });
    
    let end = create_node!("End", |_| {
        println!("End node executed");
        Ok(json!("terminate"))
    });
    
    // This uses our new branching_flow! macro
    let bf = branching_flow!("BranchingFlow", start => {
        "path_a" => path_a => "default" => end,
        "path_b" => path_b => "default" => end
    });
    
    let mut shared = HashMap::new();
    bf.orchestrate(&mut shared, None)?;
    
    println!("All examples completed successfully!");
    Ok(())
}

