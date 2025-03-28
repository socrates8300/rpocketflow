pub mod async_node;
pub mod errors;
pub mod macros;
pub mod macros_tests;
pub mod sync;
pub mod mcp;

// Re-export common types for convenience
pub use async_node::{
    AsyncFlow, AsyncNode, AsyncNodeImpl,
    async_node, async_then, async_when, AsyncConditionalTransition // New async helpers
};
pub use sync::types::AsyncNodeRef; // Export from its actual location
pub use errors::{FlowError, FlowResult};
pub use sync::{
    node, then, when, Action, BaseNode, ConditionalTransition, Flow, Node, NodeRef, NodeResult,
    Params, Shared, SyncNode,
};

// Note: Macros defined with #[macro_export] are automatically available at crate root
// The following macros are available: node_impl, flow, decision_node, processing_chain

// Re-export MCP types (factory functions now return AsyncNodeRef)
pub use mcp::{McpConfig, McpNode, mcp_node}; // mcp_node returns AsyncNodeRef
pub use mcp::models::Models;
pub use mcp::tools::{Tool, ToolRegistry};

// Re-export MCP Protocol types (factory function now returns AsyncNodeRef)
pub use mcp::protocol::{MCPProtocolNode, MCPClientConfig, mcp_protocol_node}; // mcp_protocol_node returns AsyncNodeRef
pub use mcp::protocol::mcp_stdio_config; // Helper export
