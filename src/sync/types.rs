use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex}; // Keep StdMutex for NodeRef
use tokio::sync::Mutex as TokioMutex;    // Use TokioMutex for AsyncNodeRef

// Keep existing types
use super::sync_node::SyncNode;
use crate::errors::FlowResult;

// Import AsyncNode trait
use crate::async_node::AsyncNode;

/// Type alias for node parameters
pub type Params = HashMap<String, Value>;

/// Type alias for shared state between nodes
pub type Shared = HashMap<String, Value>;

/// Type alias for a synchronous node reference (uses standard Mutex)
pub type NodeRef = Arc<StdMutex<dyn SyncNode + Send>>;

/// Type alias for an asynchronous node reference (uses Tokio Mutex)
pub type AsyncNodeRef = Arc<TokioMutex<dyn AsyncNode + Send>>; // UPDATED TYPE

/// Type alias for results from node operations
pub type NodeResult<T> = FlowResult<T>;
