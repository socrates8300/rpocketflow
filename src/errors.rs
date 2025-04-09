//! Error handling for RPocketFlow
//!
//! This module provides error types and result aliases for the library.

use thiserror::Error;
use std::io;

/// Custom error type for RPocketFlow
#[derive(Debug, Error)]
pub enum FlowError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Node execution error: {0}")]
    NodeExecution(String),

    #[error("Flow orchestration error: {0}")]
    FlowOrchestration(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("MCP error: {0}")]
    Mcp(String),
}

/// Result type alias for RPocketFlow operations
pub type FlowResult<T> = Result<T, FlowError>;

