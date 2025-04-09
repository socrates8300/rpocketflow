//! Tests for MCP integration
//!
//! This module contains tests for MCP nodes, tools, and conversation management.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::tools::{Tool, ToolRegistry, string_param};
    use crate::mcp::conversation::{ConversationManager, ToolCallInfo};
    use crate::sync::Shared;
    use anthropic::types::Role;
    use serde_json::json;
    
    #[test]
    fn test_conversation_manager() {
        let mut convo = ConversationManager::new();
        
        // Add messages
        convo.add_user_message("Hello");
        convo.add_assistant_message("Hi there!");
        convo.add_user_message("How are you?");
        
        // Check conversation state
        assert_eq!(convo.len(), 3);
        let messages = convo.get_messages();
        assert_eq!(messages[0].content, "Hello");
        assert_eq!(messages[1].content, "Hi there!");
        assert_eq!(messages[2].content, "How are you?");
        
        // Test tuple conversion
        let tuples = convo.get_messages_as_tuples();
        assert_eq!(tuples.len(), 3);
        assert_eq!(tuples[0].0, Role::User);
        assert_eq!(tuples[0].1, "Hello");
        
        // Test trimming
        convo.trim_to_last(2);
        assert_eq!(convo.len(), 2);
        let messages = convo.get_messages();
        assert_eq!(messages[0].content, "Hi there!");
        assert_eq!(messages[1].content, "How are you?");
        
        // Test JSON conversion
        let json_val = convo.to_json();
        assert!(json_val.is_array());
        assert_eq!(json_val.as_array().unwrap().len(), 2);
        
        // Test clearing
        convo.clear();
        assert_eq!(convo.len(), 0);
        assert!(convo.is_empty());
    }
    
    #[test]
    fn test_tool_handler_state() {
        // Create a tool with a handler
        let tool = Tool::new("test_tool", "A test tool", json!({
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "The query to process"}
            },
            "required": ["query"]
        })).with_handler(|args| {
            let query = args.get("query")
                .and_then(|v| v.as_str())
                .unwrap_or("");
                
            Ok(json!({
                "result": format!("Processed: {}", query)
            }))
        });
        
        // Check handler state
        assert!(tool.handler_available());
        assert_eq!(tool.handler_state_description(), "available");
        
        // Test execution
        let args = json!({"query": "test query"});
        let result = tool.execute(args);
        assert!(result.is_ok());
        let result_value = result.unwrap();
        assert_eq!(result_value["result"].as_str().unwrap(), "Processed: test query");
        
        // Clone the tool
        let cloned_tool = tool.clone();
        
        // Check handler state of cloned tool
        assert!(!cloned_tool.handler_available());
        assert_eq!(cloned_tool.handler_state_description(), "removed during clone");
        
        // Try to execute the cloned tool (should fail)
        let args = json!({"query": "test query"});
        let result = cloned_tool.execute(args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("removed during cloning"));
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
        
        // Test non-existent tool
        let result = registry.process_tool_call("nonexistent_tool", json!({}));
        assert!(result.get("error").is_some());
        assert!(result["error"].as_str().unwrap().contains("Tool not found"));
        
        // Test tool declarations
        let declarations = registry.to_tool_declarations();
        assert_eq!(declarations.len(), 1);
        assert_eq!(declarations[0]["name"], "test_tool");
        assert_eq!(declarations[0]["description"], "A test tool");
    }
    
    // For tests that require mocking the Anthropic API, you would use a
    // crate like mockito. Those tests would be feature-gated and implemented
    // in separate test modules with the appropriate setup.
}
