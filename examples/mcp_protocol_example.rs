#![allow(unused)]
use log::{error, info};
use rpocketflow::*;
use serde_json::json;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    // Check if the server executable exists
    let server_path = Path::new("./target/debug/examples/mcp_server_example");
    if !server_path.exists() {
        error!("Server executable not found. Please run 'cargo build --example mcp_server_example' first.");
        return Ok(());
    }

    // Create MCP client configuration
    let mcp_config = MCPClientConfig::new("RPocketFlow Example", "0.1.0")
        .with_server_command(server_path.to_string_lossy().to_string(), vec![]);

    // Create nodes for the flow
    let protocol_node = mcp_protocol_node("MCPProtocolNode", mcp_config);

    let input_node = node_impl! {
        name: "UserInputNode",
        exec: |_: &serde_json::Value| -> NodeResult<serde_json::Value> {
            println!("\nEnter the tool name to call (or 'exit' to quit, 'list' to view tools):");
            print!("> ");
            std::io::stdout().flush()
                .map_err(|e| format!("Failed to flush stdout: {}", e))?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)
                .map_err(|e| format!("Failed to read user input: {}", e))?;

            let input = input.trim();

            if input == "exit" {
                return Ok(json!("exit"));
            }

            if input == "list" {
                return Ok(json!("list"));
            }

            // Ask for parameters if it's a tool call
            println!("Enter the parameters in JSON format (or press Enter for empty params):");
            print!("> ");
            std::io::stdout().flush()
                .map_err(|e| format!("Failed to flush stdout: {}", e))?;

            let mut params_input = String::new();
            std::io::stdin().read_line(&mut params_input)
                .map_err(|e| format!("Failed to read user input: {}", e))?;

            let params = params_input.trim();
            let params_value = if params.is_empty() {
                json!({})
            } else {
                match serde_json::from_str(params) {
                    Ok(v) => v,
                    Err(e) => {
                        error!("Invalid JSON parameters: {}", e);
                        println!("Invalid JSON parameters. Using empty params.");
                        json!({})
                    }
                }
            };

            Ok(json!({
                "tool_name": input,
                "params": params_value
            }))
        },
        post: |shared: &mut Shared, _: &serde_json::Value, exec_res: &serde_json::Value| {
            if exec_res.as_str() == Some("exit") {
                return Ok(json!("terminate"));
            }

            if exec_res.as_str() == Some("list") {
                // Display available tools if we have them
                if let Some(tools) = shared.get("mcp_tools") {
                    if let Some(tools_obj) = tools.as_object() {
                        println!("\nAvailable tools:");
                        for (name, desc) in tools_obj {
                            println!("  - {} - {}", name, desc);
                        }
                    }
                } else {
                    println!("Tool list not available yet. Try again after initialization.");
                }
                return Ok(json!("continue"));
            }

            // Store the tool call info in shared state
            shared.insert("mcp_tool_call".to_string(), exec_res.clone());

            Ok(json!("default"))
        }
    };

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

            Ok(json!("continue"))
        }
    };

    // Define the flow
    let flow = flow! {
        name: "MCP Protocol Example Flow",
        start: input_node.clone(),
        connections: [
            (input_node.clone(), "default", protocol_node.clone()),
            (protocol_node.clone(), "success", output_node.clone()),
            (protocol_node.clone(), "error", output_node.clone()),
            (output_node.clone(), "continue", input_node.clone())
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

