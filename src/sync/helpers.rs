use std::sync::Arc;
use std::sync::Mutex;

use super::sync_node::SyncNode;
use super::types::NodeRef;

/// Create a NodeRef from a SyncNode implementation
pub fn node<T: SyncNode + 'static>(node: T) -> NodeRef {
    Arc::new(Mutex::new(node))
}

/// Helper function to chain nodes using the default action
pub fn then(from: &NodeRef, to: NodeRef) -> NodeRef {
    from.lock()
        .unwrap()
        .add_successor("default".to_string(), to.clone());
    to
}

/// Struct and helper for adding a conditional transition
pub struct ConditionalTransition {
    pub src: NodeRef,
    pub action: String,
}

impl ConditionalTransition {
    /// Chain the transition to a target node
    pub fn then(self, tgt: NodeRef) -> NodeRef {
        self.src
            .lock()
            .unwrap()
            .add_successor(self.action, tgt.clone());
        tgt
    }
}

/// Helper function to create a conditional transition
pub fn when(node: &NodeRef, action: impl Into<String>) -> ConditionalTransition {
    ConditionalTransition {
        src: node.clone(),
        action: action.into(),
    }
}
