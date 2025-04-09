#![allow(unused)]
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

use super::action::Action;
use super::base_node::BaseNode;
use super::flow::Flow;
use super::helpers::{node, then, when};
use super::node::Node;
use super::sync_node::SyncNode;
use super::types::{NodeRef, NodeResult, Params, Shared};

// A test node that adds a value to shared state
struct AddToShared {
    base: BaseNode,
    key: String,
    value: Value,
}

impl AddToShared {
    fn new(name: impl Into<String>, key: impl Into<String>, value: Value) -> Self {
        AddToShared {
            base: BaseNode::new(name),
            key: key.into(),
            value,
        }
    }
}

impl Node for AddToShared {
    fn get_params(&self) -> &Params {
        self.base.get_params()
    }

    fn set_params(&mut self, params: Params) {
        self.base.set_params(params);
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

impl SyncNode for AddToShared {
    fn post(
        &mut self,
        shared: &mut Shared,
        _prep_res: &Value,
        _exec_res: &Value,
    ) -> NodeResult<Value> {
        shared.insert(self.key.clone(), self.value.clone());
        Ok(Value::Null)
    }
}

// A test node that returns a specific action
struct ActionNode {
    base: BaseNode,
    return_action: String,
}

impl ActionNode {
    fn new(name: impl Into<String>, return_action: impl Into<String>) -> Self {
        ActionNode {
            base: BaseNode::new(name),
            return_action: return_action.into(),
        }
    }
}

impl Node for ActionNode {
    fn get_params(&self) -> &Params {
        self.base.get_params()
    }

    fn set_params(&mut self, params: Params) {
        self.base.set_params(params);
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

impl SyncNode for ActionNode {
    fn post(
        &mut self,
        _shared: &mut Shared,
        _prep_res: &Value,
        _exec_res: &Value,
    ) -> NodeResult<Value> {
        Ok(Value::String(self.return_action.clone()))
    }
}

#[test]
fn test_basic_flow() {
    let node1 = node(AddToShared::new(
        "Node1",
        "key1",
        Value::String("value1".to_string()),
    ));
    let node2 = node(AddToShared::new(
        "Node2",
        "key2",
        Value::String("value2".to_string()),
    ));
    let node3 = node(AddToShared::new(
        "Node3",
        "key3",
        Value::String("value3".to_string()),
    ));

    then(&node1, node2.clone());
    then(&node2, node3.clone());

    let flow = Flow::new("TestFlow", node1);
    let mut shared = HashMap::new();

    let result = flow.orchestrate(&mut shared, None);
    assert!(result.is_ok());

    assert_eq!(shared.len(), 3);
    assert_eq!(
        shared.get("key1").unwrap(),
        &Value::String("value1".to_string())
    );
    assert_eq!(
        shared.get("key2").unwrap(),
        &Value::String("value2".to_string())
    );
    assert_eq!(
        shared.get("key3").unwrap(),
        &Value::String("value3".to_string())
    );
}

#[test]
fn test_branching_flow() {
    let start = node(ActionNode::new("Start", "branch1"));
    let branch1 = node(AddToShared::new(
        "Branch1",
        "path",
        Value::String("branch1".to_string()),
    ));
    let branch2 = node(AddToShared::new(
        "Branch2",
        "path",
        Value::String("branch2".to_string()),
    ));

    when(&start, "branch1").then(branch1.clone());
    when(&start, "branch2").then(branch2.clone());

    let flow = Flow::new("BranchingFlow", start);
    let mut shared = HashMap::new();

    let result = flow.orchestrate(&mut shared, None);
    assert!(result.is_ok());

    assert_eq!(shared.len(), 1);
    assert_eq!(
        shared.get("path").unwrap(),
        &Value::String("branch1".to_string())
    );
}

#[test]
fn test_termination() {
    let node1 = node(AddToShared::new(
        "Node1",
        "key1",
        Value::String("value1".to_string()),
    ));
    let node2 = node(ActionNode::new("Node2", "terminate"));
    let node3 = node(AddToShared::new(
        "Node3",
        "key3",
        Value::String("value3".to_string()),
    ));

    then(&node1, node2.clone());
    then(&node2, node3.clone());

    let flow = Flow::new("TerminationFlow", node1);
    let mut shared = HashMap::new();

    let result = flow.orchestrate(&mut shared, None);
    assert!(result.is_ok());

    assert_eq!(shared.len(), 1);
    assert_eq!(
        shared.get("key1").unwrap(),
        &Value::String("value1".to_string())
    );
    assert!(!shared.contains_key("key3"));
}
