use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub type Params = HashMap<String, Value>;
pub type Shared = HashMap<String, Value>;
pub type NodeRef = Arc<Mutex<dyn SyncNode + Send>>;

pub trait SyncNode: Send {
    fn set_params(&mut self, params: Params);
    fn get_params(&self) -> &Params;
    fn add_successor(&mut self, action: String, successor: NodeRef);
    fn get_successors(&self) -> &HashMap<String, NodeRef>;
    fn get_successors_mut(&mut self) -> &mut HashMap<String, NodeRef>;

    // Default lifecycle methods.
    fn prep(&mut self, _shared: &mut Shared) -> Result<Value, String> {
        Ok(Value::Null)
    }
    fn exec(&mut self, _prep_res: &Value) -> Result<Value, String> {
        Ok(Value::Null)
    }
    fn post(
        &mut self,
        _shared: &mut Shared,
        _prep_res: &Value,
        _exec_res: &Value,
    ) -> Result<Value, String> {
        Ok(Value::Null)
    }
    fn _run(&mut self, shared: &mut Shared) -> Result<Value, String> {
        let prep_res = self.prep(shared)?;
        let exec_res = self.exec(&prep_res)?;
        self.post(shared, &prep_res, &exec_res)
    }
}

#[derive(Default)]
pub struct BaseNode {
    pub params: Params,
    pub successors: HashMap<String, NodeRef>,
}

impl BaseNode {
    pub fn new() -> Self {
        BaseNode {
            params: HashMap::new(),
            successors: HashMap::new(),
        }
    }
}

impl SyncNode for BaseNode {
    fn set_params(&mut self, params: Params) {
        self.params = params;
    }
    fn get_params(&self) -> &Params {
        &self.params
    }
    fn add_successor(&mut self, action: String, successor: NodeRef) {
        if self.successors.contains_key(&action) {
            println!("Warning: Overwriting successor for action '{}'", action);
        }
        self.successors.insert(action, successor);
    }
    fn get_successors(&self) -> &HashMap<String, NodeRef> {
        &self.successors
    }
    fn get_successors_mut(&mut self) -> &mut HashMap<String, NodeRef> {
        &mut self.successors
    }
}

// Helper function to chain nodes using the default ("default") action.
pub fn then(node: &NodeRef, successor: NodeRef) -> NodeRef {
    node.lock()
        .unwrap()
        .add_successor("default".to_string(), successor.clone());
    successor
}

// Struct and helper for adding a conditional transition.
pub struct ConditionalTransition {
    pub src: NodeRef,
    pub action: String,
}

impl ConditionalTransition {
    pub fn then(self, tgt: NodeRef) -> NodeRef {
        self.src
            .lock()
            .unwrap()
            .add_successor(self.action, tgt.clone());
        tgt
    }
}

pub fn sub(node: &NodeRef, action: &str) -> ConditionalTransition {
    ConditionalTransition {
        src: node.clone(),
        action: action.to_string(),
    }
}

pub struct Flow {
    pub base: BaseNode,
    pub start: NodeRef,
}

impl Flow {
    pub fn new(start: NodeRef) -> Self {
        Flow {
            base: BaseNode::new(),
            start,
        }
    }

    pub fn get_next_node(curr: &dyn SyncNode, action: &Value) -> Option<NodeRef> {
        let act = if action.is_null() {
            "default".to_string()
        } else if let Some(s) = action.as_str() {
            s.to_string()
        } else {
            "default".to_string()
        };
        let succ = curr.get_successors().get(&act);
        if succ.is_none() && !curr.get_successors().is_empty() {
            println!(
                "Warning: Flow ends: '{}' not found in {:?}",
                act,
                curr.get_successors().keys().collect::<Vec<_>>()
            );
        }
        succ.cloned()
    }

    pub fn orch(&self, shared: &mut Shared, params_override: Option<Params>) {
        let p = params_override.unwrap_or_else(|| self.base.params.clone());
        let mut curr = self.start.clone();
        loop {
            {
                let curr_clone = curr.clone();
                let mut node = curr_clone.lock().unwrap();
                node.set_params(p.clone());
                match node._run(shared) {
                    Ok(action) => {
                        if let Some(next_node) = Flow::get_next_node(&*node, &action) {
                            curr = next_node;
                        } else {
                            break;
                        }
                    }
                    Err(e) => {
                        println!("Error in Flow execution: {}", e);
                        break;
                    }
                }
            }
        }
    }
}

impl SyncNode for Flow {
    fn set_params(&mut self, params: Params) {
        self.base.params = params;
    }
    fn get_params(&self) -> &Params {
        &self.base.params
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
    fn prep(&mut self, shared: &mut Shared) -> Result<Value, String> {
        self.base.prep(shared)
    }
    fn exec(&mut self, _prep_res: &Value) -> Result<Value, String> {
        Err("Flow can't exec.".to_string())
    }
    fn post(
        &mut self,
        shared: &mut Shared,
        prep_res: &Value,
        _exec_res: &Value,
    ) -> Result<Value, String> {
        self.base.post(shared, prep_res, &Value::Null)
    }
    fn _run(&mut self, shared: &mut Shared) -> Result<Value, String> {
        let prep_res = self.prep(shared)?;
        self.orch(shared, None);
        self.post(shared, &prep_res, &Value::Null)
    }
}
