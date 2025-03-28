#![allow(unused)]
#[cfg(test)]
mod mcp_node_tests {
    use crate::mcp::models::Models;
    use crate::mcp::{mcp_node, McpConfig};
    use crate::sync::{Node, Shared, SyncNode};
    use serde_json::json;
    use std::collections::HashMap;
    use std::env;

    #[test]
    fn test_mcp_node_creation() {
        let config = McpConfig::new("test_key", Models::CLAUDE_3_HAIKU)
            .with_system_prompt("Test prompt")
            .with_max_tokens(100)
            .with_temperature(0.5);

        let node = mcp_node("TestNode", config);

        // Verify the node was created with the correct name
        let node_ref = node.lock().unwrap();
        assert_eq!(node_ref.get_name(), "TestNode");
    }
}

#[cfg(test)]
mod protocol_tests {
    use crate::mcp::protocol::{mcp_protocol_node, MCPClientConfig};
    use crate::sync::{Node, Shared, SyncNode};
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_mcp_protocol_node_creation() {
        let config = MCPClientConfig::new("TestClient", "1.0.0");
        let node = mcp_protocol_node("TestProtocolNode", config);

        // Verify the node was created with the correct name
        let node_ref = node.lock().unwrap();
        assert_eq!(node_ref.get_name(), "TestProtocolNode");
    }
}

#[cfg(test)]
mod tool_registry_tests {
    use crate::mcp::tools::{string_param, Tool, ToolRegistry};
    use serde_json::json;

    #[test]
    fn test_tool_registry() {
        // Create a tool registry
        let mut registry = ToolRegistry::new();

        // Create a test tool
        let test_tool = Tool::new(
            "test_tool",
            "A test tool",
            json!({
                "type": "object",
                "properties": {
                    "input": string_param("Test input parameter")
                },
                "required": ["input"]
            }),
        )
        .with_handler(|args| {
            let input = args
                .get("input")
                .and_then(|v| v.as_str())
                .unwrap_or("default");

            Ok(json!({
                "output": format!("Processed: {}", input)
            }))
        });

        // Register the tool
        registry.register(test_tool);

        // Verify tool retrieval
        let retrieved_tool = registry.get("test_tool").expect("Tool should exist");
        assert_eq!(retrieved_tool.name, "test_tool");
        assert_eq!(retrieved_tool.description, "A test tool");

        // Test tool execution
        let result = registry.process_tool_call("test_tool", json!({"input": "test_value"}));
        assert_eq!(
            result.get("output").and_then(|v| v.as_str()),
            Some("Processed: test_value")
        );

        // Test non-existent tool
        let error_result = registry.process_tool_call("nonexistent_tool", json!({}));
        assert!(error_result.get("error").is_some());
    }
}

