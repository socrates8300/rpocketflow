// examples/simple_example.rs
use rpocketflow::*;
use serde_json::json;
use std::collections::HashMap;

fn main() {
    // Create a simple node with our new macro
    let node = create_node!("MyNode", |_| {
        println!("Node executed!");
        Ok(json!("done"))
    });
    
    // Create a sequential flow with our new macro
    let node1 = create_node!("Step1", |_| Ok(json!("next")));
    let node2 = create_node!("Step2", |_| Ok(json!("next")));
    let node3 = create_node!("Step3", |_| Ok(json!("done")));
    
    let flow = sequential_flow!("MyFlow", node1, node2, node3);
    
    // Run the flow
    let mut shared = HashMap::new();
    match flow.orchestrate(&mut shared, None) {
        Ok(_) => println!("Flow completed successfully!"),
        Err(e) => println!("Flow error: {}", e),
    }
    
    // Create a simple tool with our new macro
    let weather_tool = mcp_tool!("get_weather", "Get weather for a location", [
        ("location", "The city name")
    ], |args| {
        let location = args["location"].as_str().unwrap_or("unknown");
        println!("Getting weather for {}", location);
        Ok(json!({"temp": 72, "condition": "sunny"}))
    });
    
    println!("Tool created: {}", weather_tool.name);
    
    println!("Example completed successfully!");
}

