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

/// Macro to create an async flow with connected nodes
///
/// # Examples
///
/// ```rust,no_run
/// use rpocketflow::*;
/// use rpocketflow::async_node::{AsyncNode, AsyncNodeImpl, async_node};
/// use serde_json::Value;
/// use std::collections::HashMap;
/// use async_trait::async_trait;
///
/// // Define a simple async node
/// struct SimpleAsyncNode {
///     base: AsyncNodeImpl,
/// }
///
/// impl SimpleAsyncNode {
///     fn new(name: impl Into<String>) -> Self {
///         Self { base: AsyncNodeImpl::new(name) }
///     }
/// }
///
/// // Implement Node trait
/// impl Node for SimpleAsyncNode {
///     fn get_params(&self) -> &Params { self.base.get_params() }
///     fn set_params(&mut self, params: Params) { self.base.set_params(params); }
///     fn add_successor(&mut self, action: String, successor: NodeRef) { self.base.add_successor(action, successor); }
///     fn get_successors(&self) -> &HashMap<String, NodeRef> { self.base.get_successors() }
///     fn get_successors_mut(&mut self) -> &mut HashMap<String, NodeRef> { self.base.get_successors_mut() }
///     fn get_name(&self) -> &str { self.base.get_name() }
///     fn get_max_retries(&self) -> usize { self.base.get_max_retries() }
///     fn get_wait_duration(&self) -> std::time::Duration { self.base.get_wait_duration() }
/// }
///
/// // Implement SyncNode for compatibility
/// impl SyncNode for SimpleAsyncNode {}
///
/// // Implement AsyncNode trait
/// #[async_trait]
/// impl AsyncNode for SimpleAsyncNode {
///     // Implement required async successor methods
///     fn add_async_successor(&mut self, action: String, successor: AsyncNodeRef) {
///         self.base.add_async_successor(action, successor);
///     }
///     fn get_async_successors(&self) -> &HashMap<String, AsyncNodeRef> {
///         self.base.get_async_successors()
///     }
///     fn get_async_successors_mut(&mut self) -> &mut HashMap<String, AsyncNodeRef> {
///         self.base.get_async_successors_mut()
///     }
/// }
///
/// // Create a simple async flow with a start node
/// let start_node = async_node(SimpleAsyncNode::new("StartNode"));
/// let flow = async_flow! {
///     name: "SimpleAsyncFlow",
///     start: start_node.clone()
/// };
/// ```
#[macro_export]
macro_rules! async_flow {
    (
        name: $name:expr,
        start: $start:expr
        $(,)?
    ) => {{
        use $crate::async_node::AsyncFlow;
        AsyncFlow::new($name, $start)
    }};
}
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
