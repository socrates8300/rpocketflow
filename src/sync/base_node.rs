use std::collections::HashMap;
use std::time::Duration;

use super::node::Node;
use super::sync_node::SyncNode;
use super::types::{NodeRef, Params};

/// Base implementation of a node
pub struct BaseNode {
    /// Node name for debugging and logging
    pub name: String,
    /// Node parameters
    pub params: Params,
    /// Node successors mapped by action
    pub successors: HashMap<String, NodeRef>,
    /// Maximum number of retry attempts
    pub max_retries: usize,
    /// Wait duration between retry attempts
    pub wait_duration: Duration,
}

impl BaseNode {
    /// Create a new BaseNode with default settings
    pub fn new(name: impl Into<String>) -> Self {
        BaseNode {
            name: name.into(),
            params: HashMap::new(),
            successors: HashMap::new(),
            max_retries: 1,
            wait_duration: Duration::from_millis(0),
        }
    }

    /// Set the maximum number of retry attempts
    pub fn with_max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Set the wait duration between retry attempts
    pub fn with_wait_duration(mut self, wait_duration: Duration) -> Self {
        self.wait_duration = wait_duration;
        self
    }

    /// Set initial parameters
    pub fn with_params(mut self, params: Params) -> Self {
        self.params = params;
        self
    }
}

impl Node for BaseNode {
    fn get_params(&self) -> &Params {
        &self.params
    }

    fn set_params(&mut self, params: Params) {
        self.params = params;
    }

    fn add_successor(&mut self, action: String, successor: NodeRef) {
        if self.successors.contains_key(&action) {
            println!(
                "Warning: Node '{}': Overwriting successor for action '{}'",
                self.name, action
            );
        }
        self.successors.insert(action, successor);
    }

    fn get_successors(&self) -> &HashMap<String, NodeRef> {
        &self.successors
    }

    fn get_successors_mut(&mut self) -> &mut HashMap<String, NodeRef> {
        &mut self.successors
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_max_retries(&self) -> usize {
        self.max_retries
    }

    fn get_wait_duration(&self) -> Duration {
        self.wait_duration
    }
}

impl SyncNode for BaseNode {}
