use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::sync_node::SyncNode;
use crate::errors::FlowResult;

/// Type alias for node parameters
pub type Params = HashMap<String, Value>;

/// Type alias for shared state between nodes
pub type Shared = HashMap<String, Value>;

/// Type alias for a node reference
pub type NodeRef = Arc<Mutex<dyn SyncNode + Send>>;

/// Type alias for results from node operations
pub type NodeResult<T> = FlowResult<T>;
