#![allow(unused)]
use log::{error, info};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    // We'll be using stdio directly
    let stdin = std::io::stdin();
    let mut stdin = BufReader::new(stdin);
    let mut stdout = std::io::stdout();

    // Server info
    let server_info = json!({
        "jsonrpc": "2.0",
        "result": {
            "serverInfo": {
                "name": "RPocketFlow MCP Server",
                "version": "1.0.0"
            },
            "protocolVersion": "0.1",
            "capabilities": {
                "tools": ["echo", "add"]
            }
        },
        "id": 1
    });

    // Register available tools
    let tools = json!({
        "echo": {
            "name": "echo",
            "description": "Echo back the input",
            "parameters": {
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "The message to echo"
                    }
                },
                "required": ["message"]
            }
        },
        "add": {
            "name": "add",
            "description": "Add two numbers",
            "parameters": {
                "type": "object",
                "properties": {
                    "a": {
                        "type": "number",
                        "description": "First number"
                    },
                    "b": {
                        "type": "number",
                        "description": "Second number"
                    }
                },
                "required": ["a", "b"]
            }
        }
    });

    info!("Starting MCP server...");

    // Receive messages and respond
    let mut buffer = String::new();
    loop {
        // Read a line from stdin
        buffer.clear();
        if stdin.read_line(&mut buffer).unwrap() == 0 {
            // EOF reached
            break;
        }

        // Parse the message
        let message: Value = match serde_json::from_str(&buffer) {
            Ok(msg) => msg,
            Err(e) => {
                error!("Failed to parse message: {}", e);
                continue;
            }
        };

        info!("Received message: {}", message);

        // Process message based on method
        let method = message.get("method").and_then(|m| m.as_str());
        let id = message.get("id").and_then(|i| i.as_u64()).unwrap_or(0);

        let response = match method {
            Some("initialize") => {
                info!("Processing initialize request");
                server_info.clone()
            }
            Some("tools/list") => {
                info!("Processing tools/list request");
                json!({
                    "jsonrpc": "2.0",
                    "result": {
                        "tools": [
                            {
                                "name": "echo",
                                "description": "Echo back the input"
                            },
                            {
                                "name": "add",
                                "description": "Add two numbers"
                            }
                        ]
                    },
                    "id": id
                })
            }
            Some("echo") => {
                let message = message
                    .get("params")
                    .and_then(|p| p.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("No message provided");

                info!("Echo handler called with message: {}", message);

                json!({
                    "jsonrpc": "2.0",
                    "result": {
                        "echo": message
                    },
                    "id": id
                })
            }
            Some("add") => {
                let a = message
                    .get("params")
                    .and_then(|p| p.get("a"))
                    .and_then(|a| a.as_f64())
                    .unwrap_or(0.0);

                let b = message
                    .get("params")
                    .and_then(|p| p.get("b"))
                    .and_then(|b| b.as_f64())
                    .unwrap_or(0.0);

                info!("Add handler called with a={}, b={}", a, b);

                let sum = a + b;
                json!({
                    "jsonrpc": "2.0",
                    "result": {
                        "sum": sum
                    },
                    "id": id
                })
            }
            Some("shutdown") => {
                info!("Processing shutdown request");
                json!({
                    "jsonrpc": "2.0",
                    "result": null,
                    "id": id
                })
            }
            _ => {
                error!("Unknown method: {:?}", method);
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

    info!("Shutting down MCP server");
    Ok(())
}

