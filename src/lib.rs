pub mod async_node;
pub mod macros;
pub mod macros_tests;
pub mod sync;
pub mod mcp;

// Re-export common types for convenience
pub use async_node::{AsyncFlow, AsyncNode, AsyncNodeImpl};
pub use sync::{
    node, then, when, Action, BaseNode, ConditionalTransition, Flow, Node, NodeRef, NodeResult,
    Params, Shared, SyncNode,
};

// Note: Macros defined with #[macro_export] are automatically available at crate root
// The following macros are available: node_impl, flow, decision_node, processing_chain

// Re-export MCP types
pub use mcp::{McpConfig, McpNode, mcp_node};
pub use mcp::models::Models;
pub use mcp::tools::{Tool, ToolRegistry};
