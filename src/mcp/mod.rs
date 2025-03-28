use anthropic::client::{Client as AnthropicClient, ClientBuilder};
use anthropic::types::{Message, MessagesRequest, Role, ContentBlock};
use async_trait::async_trait;
use serde_json::{Value, json};
use std::collections::HashMap;
// All imports are needed
use log::{debug, error};

use crate::async_node::AsyncNode;
use crate::async_node::EMPTY_ASYNC_SUCCESSORS; // Import static empty map
use crate::sync::{Node, NodeRef, NodeResult, Params, Shared, BaseNode, SyncNode};
use crate::sync::types::AsyncNodeRef; // Import AsyncNodeRef type

pub mod models;
pub mod tools;
pub mod protocol;
#[cfg(test)]
pub mod tests;

// Re-export the stdio config helper for easier access
pub use protocol::mcp_stdio_config;

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
    client: AnthropicClient,
    messages: Vec<(Role, String)>,
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
            messages: Vec::new(),
        }
    }

    /// Add a message to the conversation
    pub fn add_message(&mut self, role: impl Into<String>, content: impl Into<String>) {
        let role_str = role.into();
        let role = match role_str.as_str() {
            "user" => Role::User,
            "assistant" => Role::Assistant,
            _ => Role::User, // Default to user for unknown roles
        };
        
        self.messages.push((role, content.into()));
    }

    /// Get the current conversation
    pub fn get_messages(&self) -> &Vec<(Role, String)> {
        &self.messages
    }

    /// Clear the conversation
    pub fn clear_messages(&mut self) {
        self.messages.clear();
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
        // Create message objects from stored messages
        let mut api_messages = Vec::new();
        for (role, content) in &self.messages {
            api_messages.push(Message {
                role: *role,
                content: vec![ContentBlock::Text { 
                    text: content.clone(),
                }],
            });
        }
        
        // Get the system prompt (use empty string if none provided)
        let system = self.config.system_prompt.clone().unwrap_or_default();
        
        // Create the request with all required fields
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
        
        // Execute the request
        match self.client.messages(request).await {
            Ok(response) => {
                debug!("MCP response received");
                let text = Self::extract_text_from_response(&response);
                
                // Store the assistant message in the conversation history
                self.messages.push((Role::Assistant, text.clone()));
                
                Ok(json!({
                    "text": text,
                    "raw_message": json!(response),
                }))
            },
            Err(e) => {
                error!("Error from MCP API: {}", e);
                Err(crate::errors::FlowError::Anthropic(format!("MCP API error: {}", e)))
            }
        }
    }
    
    async fn prep_async(&mut self, shared: &mut Shared) -> NodeResult<Value> {
        // Check if there are any input messages in the shared state
        if let Some(input) = shared.get("mcp_input") {
            if let Some(input_str) = input.as_str() {
                // Add user message to the conversation
                self.add_message("user", input_str);
            }
        }
        
        // Check if there's conversation history to load
        if let Some(history) = shared.get("mcp_history") {
            if let Some(history_array) = history.as_array() {
                for msg in history_array {
                    let role = msg.get("role")
                        .ok_or_else(|| "Missing 'role' in history message".to_string())?
                        .as_str()
                        .ok_or_else(|| "Role must be a string".to_string())?;
                    
                    let content = msg.get("content")
                        .ok_or_else(|| "Missing 'content' in history message".to_string())?
                        .as_str()
                        .ok_or_else(|| "Content must be a string".to_string())?;
                    
                    self.add_message(role, content);
                }
            }
        }
        
        Ok(json!({"prepared": true}))
    }
    
    async fn post_async(&mut self, shared: &mut Shared, _prep_res: &Value, exec_res: &Value) -> NodeResult<Value> {
        // Store the response in the shared state
        if let Some(text) = exec_res.get("text") {
            shared.insert("mcp_output".to_string(), text.clone());
        }
        
        // Update the conversation history in the shared state
        let messages = self.get_messages().iter().map(|(role, content)| {
            let role_str = match role {
                Role::User => "user",
                Role::Assistant => "assistant",
            };
            
            json!({
                "role": role_str,
                "content": content,
            })
        }).collect::<Vec<_>>();
        
        shared.insert("mcp_history".to_string(), json!(messages));
        
        Ok(json!("default"))
    }
    
    // Add required async successor methods
    fn add_async_successor(&mut self, _action: String, _successor: AsyncNodeRef) {
        // Since McpNode doesn't store async successors directly, log a warning
        log::warn!("McpNode '{}' does not support direct async successors; use wrapping structure like AsyncFlow.", self.get_name());
    }

    fn get_async_successors(&self) -> &HashMap<String, AsyncNodeRef> {
        // Use the static empty map from once_cell
        &EMPTY_ASYNC_SUCCESSORS
    }

    fn get_async_successors_mut(&mut self) -> &mut HashMap<String, AsyncNodeRef> {
        // This should never be called since we don't store async successors
        log::warn!("McpNode does not support direct mutable access to async successors.");
        unimplemented!("McpNode does not directly manage mutable async successors");
    }
}


/// Create a new MCP node
pub fn mcp_node(name: impl Into<String>, config: McpConfig) -> AsyncNodeRef { // Return type is AsyncNodeRef
    let node = McpNode::new(name, config);
    // Use the async_node helper which now uses TokioMutex
    crate::async_node::async_node(node)
}