use serde::Serialize;
use thiserror::Error;

/// The main error type for the rpocketflow library
#[derive(Error, Debug)]
pub enum FlowError {
    /// Error that occurs when a node execution fails
    #[error("Node execution failed: {0}")]
    NodeExecution(String),

    /// Error that occurs when a flow orchestration fails
    #[error("Flow orchestration failed: {0}")]
    FlowOrchestration(String),

    /// Error that occurs when a node is not found
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    
    /// Error that occurs when a mutex is poisoned
    #[error("Mutex poisoned: {0}")]
    MutexPoisoned(String),
    
    /// Error that occurs with MCP client operations
    #[error("MCP client error: {0}")]
    MCPClient(String),
    
    /// Error that occurs with MCP server operations
    #[error("MCP server error: {0}")]
    MCPServer(String),
    
    /// Error that occurs during protocol communication
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Error from the Anthropic API
    #[error("Anthropic API error: {0}")]
    Anthropic(String),
    
    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    /// Other/unknown error
    #[error("{0}")]
    Other(String),
}

/// Type alias for results returned by nodes in the rpocketflow library
pub type FlowResult<T> = Result<T, FlowError>;

impl From<String> for FlowError {
    fn from(s: String) -> Self {
        FlowError::Other(s)
    }
}

impl From<&str> for FlowError {
    fn from(s: &str) -> Self {
        FlowError::Other(s.to_string())
    }
}

/// Manual implementation of Serialize for FlowError
impl Serialize for FlowError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Convert the error to a string representation
        serializer.serialize_str(&self.to_string())
    }
}