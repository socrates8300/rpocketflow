//! Conversation management for MCP nodes
//!
//! This module provides structures for managing conversations with MCP models,
//! including message history, tool calls, and serialization/deserialization.

use anthropic::types::Role;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::VecDeque;

/// Represents a message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// The role of the message sender (as string to avoid serde issues)
    pub role: Role,
    /// The content of the message
    pub content: String,
    /// The timestamp when the message was created (as Unix timestamp)
    pub timestamp: u64,
    /// Optional tool call information
    pub tool_call: Option<ToolCallInfo>,
}

/// Information about a tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallInfo {
    /// The name of the tool
    pub tool_name: String,
    /// The arguments passed to the tool
    pub arguments: Value,
    /// The result of the tool call
    pub result: Option<Value>,
}

/// Manages a conversation with an MCP model
#[derive(Debug, Clone)]
pub struct ConversationManager {
    /// The messages in the conversation
    messages: VecDeque<ConversationMessage>,
    /// The maximum number of messages to keep
    max_messages: Option<usize>,
}

impl Default for ConversationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversationManager {
    /// Create a new conversation manager
    pub fn new() -> Self {
        ConversationManager {
            messages: VecDeque::new(),
            max_messages: None,
        }
    }
    
    /// Set the maximum number of messages to keep
    pub fn with_max_messages(mut self, max: usize) -> Self {
        self.max_messages = Some(max);
        self
    }
    
    /// Add a user message to the conversation
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.add_message(Role::User, content.into(), None);
    }
    
    /// Add an assistant message to the conversation
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.add_message(Role::Assistant, content.into(), None);
    }
    
    /// Add a message with a tool call
    pub fn add_tool_call(&mut self, content: impl Into<String>, tool_info: ToolCallInfo) {
        self.add_message(Role::Assistant, content.into(), Some(tool_info));
    }
    
    /// Add a message to the conversation
    fn add_message(&mut self, role: Role, content: String, tool_call: Option<ToolCallInfo>) {
        // Get current timestamp in seconds since UNIX epoch
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        let message = ConversationMessage {
            role,
            content,
            timestamp,
            tool_call,
        };
        
        self.messages.push_back(message);
        
        // Enforce maximum messages if set
        if let Some(max) = self.max_messages {
            while self.messages.len() > max {
                self.messages.pop_front();
            }
        }
    }
    
    /// Get all messages in the conversation
    pub fn get_messages(&self) -> Vec<&ConversationMessage> {
        self.messages.iter().collect()
    }
    
    /// Get messages as (Role, String) tuples for the MCP API
    pub fn get_messages_as_tuples(&self) -> Vec<(Role, String)> {
        self.messages.iter()
            .map(|msg| (msg.role, msg.content.clone()))
            .collect()
    }
    
    /// Clear all messages from the conversation
    pub fn clear(&mut self) {
        self.messages.clear();
    }
    
    /// Get the last n messages
    pub fn get_last_messages(&self, n: usize) -> Vec<&ConversationMessage> {
        self.messages.iter().rev().take(n).collect::<Vec<_>>().into_iter().rev().collect()
    }
    
    /// Trim the conversation to the last n messages
    pub fn trim_to_last(&mut self, n: usize) {
        while self.messages.len() > n {
            self.messages.pop_front();
        }
    }
    
    /// Convert the conversation to a JSON value
    pub fn to_json(&self) -> Value {
        serde_json::to_value(&self.messages).unwrap_or(Value::Null)
    }
    
    /// Create a conversation from a JSON value
    pub fn from_json(json: &Value) -> Result<Self, String> {
        if let Some(messages) = json.as_array() {
            let mut manager = ConversationManager::new();
            
            for msg in messages {
                if let (Some(role_str), Some(content)) = (
                    msg.get("role").and_then(|r| r.as_str()),
                    msg.get("content").and_then(|c| c.as_str())
                ) {
                    let role = match role_str {
                        "user" => Role::User,
                        "assistant" => Role::Assistant,
                        _ => continue,
                    };
                    
                    let tool_call = msg.get("tool_call").and_then(|tc| {
                        if tc.is_null() {
                            None
                        } else {
                            let tool_name = tc.get("tool_name")?.as_str()?;
                            let arguments = tc.get("arguments")?;
                            
                            Some(ToolCallInfo {
                                tool_name: tool_name.to_string(),
                                arguments: arguments.clone(),
                                result: tc.get("result").cloned(),
                            })
                        }
                    });
                    
                    // Parse timestamp or use current time
                    let timestamp = msg.get("timestamp")
                        .and_then(|t| t.as_u64())
                        .unwrap_or_else(|| {
                            SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs()
                        });
                    
                    // Manually create the message to avoid timestamp issues
                    let message = ConversationMessage {
                        role,
                        content: content.to_string(),
                        timestamp,
                        tool_call,
                    };
                    
                    manager.messages.push_back(message);
                }
            }
            
            Ok(manager)
        } else {
            Err("Expected an array of messages".to_string())
        }
    }
    
    /// Get the number of messages in the conversation
    pub fn len(&self) -> usize {
        self.messages.len()
    }
    
    /// Check if the conversation is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}

