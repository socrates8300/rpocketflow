use std::collections::HashMap;
use std::time::Duration;

// Removing unused import
use super::types::{NodeRef, Params};

/// Base trait for all nodes in the system
pub trait Node: Send {
    /// Get the node's parameters
    fn get_params(&self) -> &Params;

    /// Set the node's parameters
    fn set_params(&mut self, params: Params);

    /// Get the node's successors
    fn get_successors(&self) -> &HashMap<String, NodeRef>;

    /// Get mutable access to the node's successors
    fn get_successors_mut(&mut self) -> &mut HashMap<String, NodeRef>;

    /// Add a successor for a specific action
    fn add_successor(&mut self, action: String, successor: NodeRef);

    /// Get the node's name (for debugging and logging)
    fn get_name(&self) -> &str;

    /// Get the maximum number of retry attempts
    fn get_max_retries(&self) -> usize {
        1 // Default is 1 attempt (no retries)
    }

    /// Get the wait duration between retry attempts
    fn get_wait_duration(&self) -> Duration {
        Duration::from_millis(0) // Default is no wait
    }
}
