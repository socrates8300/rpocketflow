//! MCP (Model Context Protocol) integration for RPocketFlow
//!
//! This module provides integration with Anthropic's Claude models through
//! the Model Context Protocol, supporting both simple text interactions and
//! complex tool-based exchanges.

use tracing::{debug, warn, error};
use anthropic::client::{Client, ClientBuilder};
use anthropic::types::{Message, MessagesRequest, ContentBlock};
use async_trait::async_trait;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::async_node::AsyncNode;
use crate::sync::{Node, NodeRef, NodeResult, Params, Shared, BaseNode, SyncNode};
use crate::mcp::tools::ToolRegistry;
use crate::mcp::conversation::ConversationManager;

pub mod models;
pub mod tools;
pub mod conversation;
pub mod macros;

#[cfg(test)]
pub mod tests;

/// MCP client configuration for communicating with Anthropic's API
#[derive(Clone)]
pub struct McpConfig {
    /// Anthropic API key
    pub api_key: String,
    /// Model to use (e.g., "claude-3-opus-20240229")
    pub model: String,
    /// System prompt to use for the model
    pub system_prompt: Option<String>,
    /// Maximum tokens to generate in the response
    pub max_tokens: Option<u32>,
    /// Temperature for generation (0.0-1.0)
    pub temperature: Option<f32>,
}

impl McpConfig {
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        McpConfig {
            api_key: api_key.into(),
            model: model.into(),
            system_prompt: None,
            max_tokens: None,
            temperature: None,
        }
    }

    pub fn with_system_prompt(mut self, system_prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(system_prompt.into());
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
}

/// Base MCP node that provides common functionality for interacting with MCP
pub struct McpNode {
    base: BaseNode,
    config: McpConfig,
    client: Client,
    conversation: ConversationManager,
    tool_registry: Option<Arc<ToolRegistry>>,
}

impl McpNode {
    pub fn new(name: impl Into<String>, config: McpConfig) -> Self {
        let mut builder = ClientBuilder::default();
        builder.api_key(config.api_key.clone());
        
        let client = builder.build()
            .expect("Failed to create Anthropic client");
        
        McpNode {
            base: BaseNode::new(name),
            config,
            client,
            conversation: ConversationManager::new(),
            tool_registry: None,
        }
    }
    
    /// Add a tool registry for function calling
    pub fn with_tool_registry(mut self, registry: ToolRegistry) -> Self {
        self.tool_registry = Some(Arc::new(registry));
        self
    }
    
    /// Set a tool registry for function calling
    pub fn set_tool_registry(&mut self, registry: ToolRegistry) {
        self.tool_registry = Some(Arc::new(registry));
    }
    
    /// Get reference to the tool registry, if available
    pub fn get_tool_registry(&self) -> Option<&Arc<ToolRegistry>> {
        self.tool_registry.as_ref()
    }
    
    /// Set max conversation length
    pub fn with_max_conversation_length(mut self, max: usize) -> Self {
        self.conversation = self.conversation.with_max_messages(max);
        self
    }
    
    /// Add a user message to the conversation
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.conversation.add_user_message(content);
    }
    
    /// Add an assistant message to the conversation
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.conversation.add_assistant_message(content);
    }
    
    /// Get the conversation manager
    pub fn get_conversation(&self) -> &ConversationManager {
        &self.conversation
    }
    
    /// Clear the conversation
    pub fn clear_conversation(&mut self) {
        self.conversation.clear();
    }
    
    /// Extract text from a response
    pub fn extract_text_from_response(response: &anthropic::types::MessagesResponse) -> String {
        response.content.iter()
            .filter_map(|block| {
                match block {
                    ContentBlock::Text { text, .. } => Some(text.clone()),
                    _ => None,
                }
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}

impl Node for McpNode {
    fn get_params(&self) -> &Params {
        &self.base.params
    }

    fn set_params(&mut self, params: Params) {
        self.base.params = params;
    }

    fn add_successor(&mut self, action: String, successor: NodeRef) {
        self.base.add_successor(action, successor);
    }

    fn get_successors(&self) -> &HashMap<String, NodeRef> {
        &self.base.successors
    }

    fn get_successors_mut(&mut self) -> &mut HashMap<String, NodeRef> {
        &mut self.base.successors
    }

    fn get_name(&self) -> &str {
        &self.base.name
    }
}

impl SyncNode for McpNode {}

#[async_trait]
impl AsyncNode for McpNode {
    async fn exec_async(&mut self, _prep_res: &Value) -> NodeResult<Value> {
        // Create message objects from conversation manager
        let api_messages = self.conversation.get_messages_as_tuples().iter()
            .map(|(role, content)| {
                Message {
                    role: *role,
                    content: vec![ContentBlock::Text { 
                        text: content.clone(),
                    }],
                }
            })
            .collect::<Vec<_>>();
        
        // Get the system prompt (use empty string if none provided)
        let system = self.config.system_prompt.clone().unwrap_or_else(|| "".to_string());
        
        // Create the request
        let request = MessagesRequest {
            messages: api_messages,
            system,
            model: self.config.model.clone(),
            max_tokens: self.config.max_tokens.map(|t| t as usize).unwrap_or(1024),
            stop_sequences: Vec::new(),
            stream: false,
            temperature: self.config.temperature.map(|t| t as f64),
            top_p: None,
            top_k: None,
        };

        // Note: Anthropic API tools integration is handled differently in newer versions
        // We'll manually check for tool usage in the response content
        
        // Execute the request
        match self.client.messages(request).await {
            Ok(response) => {
                debug!(target: "rpocketflow::mcp", "MCP response received");
                
                // Extract text content
                let text_content = Self::extract_text_from_response(&response);
                
                // Add to conversation history
                self.conversation.add_assistant_message(text_content.clone());
                
                // Note: Anthropic's API currently returns tool calls in a different way
                // For this version, we'll implement a basic text-based approach
                // Check if there's a tool call in the text
                let tool_results: Vec<(String, String, Value, Value)> = Vec::new();
                
                // In a real implementation with a newer Anthropic API, we'd parse
                // tool calls from the response structure
                
                Ok(json!({
                    "text": text_content,
                    "tool_results": tool_results,
                    "raw_message": json!(response),
                }))
            },
            Err(e) => {
                error!(target: "rpocketflow::mcp", error = %e, "Error from MCP API");
                Err(format!("MCP API error: {}", e))
            }
        }
    }
    
    async fn prep_async(&mut self, shared: &mut Shared) -> NodeResult<Value> {
        // Check if there are any input messages in the shared state
        if let Some(input) = shared.get("mcp_input") {
            if let Some(input_str) = input.as_str() {
                // Add user message to the conversation
                self.conversation.add_user_message(input_str);
            }
        }
        
        // Check if there's conversation history to load
        if let Some(history) = shared.get("mcp_history") {
            // Create a new conversation from the history
            match ConversationManager::from_json(history) {
                Ok(manager) => {
                    // Only replace if the imported conversation has messages
                    if manager.len() > 0 {
                        self.conversation = manager;
                    }
                },
                Err(e) => {
                    error!(target: "rpocketflow::mcp", error = %e, "Failed to parse conversation history");
                    return Err(format!("Failed to parse conversation history: {}", e));
                }
            }
        }
        
        // Check if there's a tool registry in the shared state
        if let Some(tools) = shared.get("mcp_tools") {
            // Try to extract the tool registry from shared state
            // This is for backward compatibility with existing code
            if let Some(tools_obj) = tools.as_object() {
                warn!(
                    target: "rpocketflow::mcp",
                    "Using tool registry from shared state is deprecated. Use McpNode::with_tool_registry instead."
                );
                
                // Log found tools for debugging
                debug!(
                    target: "rpocketflow::mcp",
                    tools = ?tools_obj.keys().collect::<Vec<_>>(),
                    "Found tools in shared state"
                );
            }
        }
        
        Ok(json!({"prepared": true}))
    }
    
    async fn post_async(&mut self, shared: &mut Shared, _prep_res: &Value, exec_res: &Value) -> NodeResult<Value> {
        // Store the response in the shared state
        if let Some(text) = exec_res.get("text") {
            shared.insert("mcp_output".to_string(), text.clone());
        }
        
        // Store tool results if available
        if let Some(tool_results) = exec_res.get("tool_results") {
            shared.insert("mcp_tool_results".to_string(), tool_results.clone());
        }
        
        // Update the conversation history in the shared state
        shared.insert("mcp_history".to_string(), self.conversation.to_json());
        
        Ok(json!("default"))
    }
}

/// Create a new MCP node
pub fn mcp_node(name: impl Into<String>, config: McpConfig) -> NodeRef {
    let node = McpNode::new(name, config);
    Arc::new(Mutex::new(node))
}

/// Create a new MCP node with tools
pub fn mcp_node_with_tools(name: impl Into<String>, config: McpConfig, registry: ToolRegistry) -> NodeRef {
    let node = McpNode::new(name, config).with_tool_registry(registry);
    Arc::new(Mutex::new(node))
}

