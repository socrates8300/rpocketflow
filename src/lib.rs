// Core modules
pub mod sync;
pub mod marker_traits;
pub mod errors;

// Conditional modules
#[cfg(feature = "async")]
pub mod async_node;

// Macros
pub mod macros;
pub mod improved_macros;
pub mod macros_tests;

// MCP integration
#[cfg(feature = "mcp")]
pub mod mcp;

// Re-export common types for convenience
pub use sync::{
    node, then, when, Action, BaseNode, ConditionalTransition, Flow, Node, NodeRef, NodeResult,
    Params, Shared, SyncNode,
};

pub use marker_traits::{SyncContext, AsyncContext};
pub use errors::{FlowError, FlowResult};

#[cfg(feature = "async")]
pub use async_node::{AsyncFlow, AsyncNode, AsyncNodeImpl};

// Note: Macros defined with #[macro_export] are automatically available at crate root
// The following macros are available: node_impl, flow, decision_node, processing_chain
// And the improved versions: create_node, sequential_flow, branching_flow, decide, pipeline

// Re-export MCP types if enabled
#[cfg(feature = "mcp")]
pub use mcp::{McpConfig, McpNode, mcp_node, mcp_node_with_tools};

#[cfg(feature = "mcp")]
pub use mcp::models::Models;

#[cfg(feature = "mcp")]
pub use mcp::tools::{Tool, ToolRegistry};

#[cfg(feature = "mcp")]
pub use mcp::conversation::{ConversationManager, ConversationMessage, ToolCallInfo};

