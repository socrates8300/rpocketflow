#[cfg(test)]
mod tests {
    use crate::*;
    use serde_json::json;
    use anthropic::types::Role;

    // Helper to create a mock MCP config
    fn mock_mcp_config() -> McpConfig {
        McpConfig::new("test-api-key", Models::CLAUDE_3_HAIKU)
            .with_system_prompt("You are a test assistant.")
            .with_max_tokens(100)
            .with_temperature(0.0)
    }

    #[test]
    fn test_message_creation() {
        let mut node = McpNode::new("TestNode", mock_mcp_config());
        
        // Add a user message
        node.add_message("user", "Hello world");
        
        // Check that the message was added correctly
        assert_eq!(node.messages.len(), 1);
        assert_eq!(node.messages[0].1, "Hello world");
        assert!(matches!(node.messages[0].0, Role::User));
    }

    #[test]
    fn test_tool_registry() {
        // Create a tool registry
        let mut registry = ToolRegistry::new();
        
        // Add a test tool
        let test_tool = Tool::new(
            "test_tool",
            "A test tool",
            json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "The query to process"}
                },
                "required": ["query"]
            })
        ).with_handler(|args| {
            let query = args.get("query")
                .and_then(|v| v.as_str())
                .unwrap_or("");
                
            Ok(json!({
                "result": format!("Processed: {}", query)
            }))
        });
        
        registry.register(test_tool);
        
        // Test tool execution
        let args = json!({"query": "test query"});
        let result = registry.process_tool_call("test_tool", args);
        
        assert_eq!(result["result"], "Processed: test query");
    }
}