#![allow(unused)]
use serde_json::Value;
use std::collections::HashMap;
use std::sync::MutexGuard;
use std::time::Duration;

use super::action::Action;
use super::base_node::BaseNode;
use super::node::Node;
use super::sync_node::SyncNode;
use super::types::{NodeRef, NodeResult, Params, Shared};

/// A flow that orchestrates a sequence of nodes
pub struct Flow {
    pub base: BaseNode,
    pub start: NodeRef,
}

impl Flow {
    /// Create a new flow with a starting node
    pub fn new(name: impl Into<String>, start: NodeRef) -> Self {
        Flow {
            base: BaseNode::new(name),
            start,
        }
    }

    /// Get the next node based on the action from the current node
    pub fn get_next_node(node_guard: &mut dyn SyncNode, action: &Action) -> Option<NodeRef> {
        // Convert Action to string
        let action_str = action.to_string();

        // First try the specific action
        if let Some(next) = node_guard.get_successors().get(&action_str) {
            return Some(next.clone());
        }

        // Then try the default action if not already trying default
        if action != &Action::Default {
            if let Some(next) = node_guard.get_successors().get("default") {
                return Some(next.clone());
            }
        }

        // No successor found
        if !node_guard.get_successors().is_empty() {
            println!(
                "Flow ends: action '{}' not found in node '{}'. Available actions: {:?}",
                action,
                node_guard.get_name(),
                node_guard.get_successors().keys().collect::<Vec<_>>()
            );
        }

        None
    }

    /// Orchestrate the flow execution
    pub fn orchestrate(
        &self,
        shared: &mut Shared,
        params_override: Option<Params>,
    ) -> NodeResult<()> {
        let p = params_override.unwrap_or_else(|| self.base.params.clone());
        let mut curr = self.start.clone();

        loop {
            let action = {
                let mut node = match curr.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => {
                        println!("Mutex was poisoned. Recovering and continuing.");
                        poisoned.into_inner()
                    }
                };

                node.set_params(p.clone());

                match node._run(shared) {
                    Ok(action) => {
                        if action == Action::Terminate {
                            println!(
                                "Flow '{}' terminated by node '{}'",
                                self.base.name,
                                node.get_name()
                            );
                            return Ok(());
                        }

                        // Get the next node before dropping the lock
                        let next_node_option = Self::get_next_node(&mut *node, &action);

                        // Release the lock before moving to the next node
                        drop(node);

                        if let Some(next_node) = next_node_option {
                            curr = next_node;
                            continue;
                        } else {
                            // No successor found, end the flow
                            println!("Flow '{}' ended naturally", self.base.name);
                            return Ok(());
                        }
                    }
                    Err(e) => {
                        println!("Error in Flow '{}' execution: {}", self.base.name, e);
                        return Err(format!("Flow error: {}", e));
                    }
                }
            };
        }
    }
}

impl Node for Flow {
    fn set_params(&mut self, params: Params) {
        self.base.set_params(params);
    }

    fn get_params(&self) -> &Params {
        self.base.get_params()
    }

    fn add_successor(&mut self, action: String, successor: NodeRef) {
        self.base.add_successor(action, successor);
    }

    fn get_successors(&self) -> &HashMap<String, NodeRef> {
        self.base.get_successors()
    }

    fn get_successors_mut(&mut self) -> &mut HashMap<String, NodeRef> {
        self.base.get_successors_mut()
    }

    fn get_name(&self) -> &str {
        self.base.get_name()
    }

    fn get_max_retries(&self) -> usize {
        self.base.get_max_retries()
    }

    fn get_wait_duration(&self) -> Duration {
        self.base.get_wait_duration()
    }
}

impl SyncNode for Flow {
    fn prep(&mut self, shared: &mut Shared) -> NodeResult<Value> {
        self.base.prep(shared)
    }

    fn exec(&mut self, _prep_res: &Value) -> NodeResult<Value> {
        Err("Flow doesn't support direct execution".to_string())
    }

    fn post(
        &mut self,
        shared: &mut Shared,
        prep_res: &Value,
        _exec_res: &Value,
    ) -> NodeResult<Value> {
        self.base.post(shared, prep_res, &Value::Null)
    }

    fn _run(&mut self, shared: &mut Shared) -> NodeResult<Action> {
        let prep_res = self.prep(shared)?;
        self.orchestrate(shared, None)?;
        let post_result = self.post(shared, &prep_res, &Value::Null)?;
        Ok(Action::from(&post_result))
    }
}
