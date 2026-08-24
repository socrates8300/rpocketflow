/// Macro to simplify creating a new node with minimal boilerplate
///
/// # Examples
///
/// ```rust
/// use rpocketflow::node_impl;
/// use serde_json::Value;
///
/// // Create a simple node
/// let my_node = node_impl! {
///     name: "MyNode",
///     exec: |_prep_res: &Value| {
///         println!("Node is executing!");
///         Ok(Value::Null)
///     }
/// };
/// ```
#[macro_export]
macro_rules! node_impl {
    (
        name: $name:expr,
        $(prep: $prep_fn:expr,)?
        exec: $exec_fn:expr
        $(, post: $post_fn:expr)?
        $(, max_retries: $retries:expr)?
        $(, wait_duration: $wait:expr)?
        $(,)?
    ) => {{
        use $crate::sync::{BaseNode, Node, NodeResult, Params, Shared, SyncNode};
        use serde_json::Value;
        use std::collections::HashMap;
        use std::time::Duration;

        #[allow(non_camel_case_types)]
        #[doc(hidden)]
        struct GeneratedNode {
            base: BaseNode,
        }

        impl GeneratedNode {
            fn new(name: impl Into<String>) -> Self {
                let base = BaseNode::new(name);
                let base = base$(.with_max_retries($retries))?$(.with_wait_duration($wait))?;
                Self { base }
            }
        }

        impl Node for GeneratedNode {
            fn get_params(&self) -> &Params { &self.base.get_params() }
            fn set_params(&mut self, params: Params) { self.base.set_params(params); }
            fn add_successor(&mut self, action: String, successor: $crate::sync::NodeRef) {
                self.base.add_successor(action, successor);
            }
            fn get_successors(&self) -> &HashMap<String, $crate::sync::NodeRef> {
                self.base.get_successors()
            }
            fn get_successors_mut(&mut self) -> &mut HashMap<String, $crate::sync::NodeRef> {
                self.base.get_successors_mut()
            }
            fn get_name(&self) -> &str { self.base.get_name() }
            fn get_max_retries(&self) -> usize { self.base.get_max_retries() }
            fn get_wait_duration(&self) -> Duration { self.base.get_wait_duration() }
        }

        impl SyncNode for GeneratedNode {
            #[allow(unused_variables)]
            fn prep(&mut self, shared: &mut Shared) -> NodeResult<Value> {
                $(let result = ($prep_fn)(shared);
                return result;)?
                Ok(Value::Null)
            }

            fn exec(&mut self, prep_res: &Value) -> NodeResult<Value> {
                ($exec_fn)(prep_res)
            }

            #[allow(unused_variables)]
            fn post(&mut self, shared: &mut Shared, prep_res: &Value, exec_res: &Value) -> NodeResult<Value> {
                $(let result = ($post_fn)(shared, prep_res, exec_res);
                return result;)?
                Ok(Value::Null)
            }
        }

        $crate::sync::node(GeneratedNode::new($name))
    }}
}

// Restored 2026-08-24: flow!, decision_node!, processing_chain! were removed by
// "Improved everything." (9b3e218) while improved_macros.rs and the test suite
// still reference them — a half-finished migration. Recovered verbatim from 4aa6af8
// (doc-example header for flow! reconstructed).

/// Creates a flow by connecting nodes with explicit actions.
///
/// # Examples
///
/// ```rust
/// use rpocketflow::{create_node, flow};
/// use serde_json::json;
///
/// let start_node = create_node!("Start", |_| Ok(json!("path1")));
/// let path1_node = create_node!("Path1", |_| Ok(json!("done")));
/// let path2_node = create_node!("Path2", |_| Ok(json!("done")));
/// let end_node = create_node!("End", |_| Ok(json!("done")));
///
/// let flow = flow! {
///     name: "MyFlow",
///     start: start_node.clone(),
///     connections: [
///         (start_node.clone(), "path1", path1_node.clone()),
///         (start_node.clone(), "path2", path2_node.clone()),
///         (path1_node.clone(), "default", end_node.clone()),
///         (path2_node.clone(), "default", end_node.clone())
///     ]
/// };
/// ```
#[macro_export]
macro_rules! flow {

    // Simple linear flow
    (
        name: $name:expr,
        nodes: [$first:expr $(, $rest:expr)+]
    ) => {{
        use $crate::sync::{Flow, then};

        let first_node = $first;
        $(
            let _ = then(&first_node, $rest);
        )*

        Flow::new($name, first_node)
    }};

    // Flow with explicit connections
    (
        name: $name:expr,
        start: $start:expr,
        connections: [$(($from:expr, $action:expr, $to:expr)),+ $(,)?]
    ) => {{
        use $crate::sync::{Flow, when};

        let start_node = $start;
        $(
            let _ = when(&$from, $action).then($to);
        )+

        Flow::new($name, start_node)
    }};
}

/// Macro to create a simple decision node
///
/// # Examples
///
/// ```rust
/// use rpocketflow::decision_node;
/// use rpocketflow::{Params, Shared};
/// use serde_json::Value;
/// use std::collections::HashMap;
///
/// // Create a decision node based on a condition
/// let decision_node = decision_node! {
///     name: "RouteDecision",
///     condition: |params: &Params, shared: &Shared| {
///         if let Some(Value::Number(age)) = shared.get("age") {
///             if age.as_u64().unwrap_or(0) >= 18 {
///                 "adult".to_string()
///             } else {
///                 "minor".to_string()
///             }
///         } else {
///             "unknown".to_string()
///         }
///     }
/// };
/// ```
#[macro_export]
macro_rules! decision_node {
    (
        name: $name:expr,
        condition: $condition:expr
        $(, max_retries: $retries:expr)?
        $(, wait_duration: $wait:expr)?
        $(,)?
    ) => {{
        use $crate::sync::{BaseNode, Node, NodeResult, Params, Shared, SyncNode};
        use serde_json::Value;
        use std::collections::HashMap;
        use std::time::Duration;

        struct DecisionNode {
            base: BaseNode,
            condition: Box<dyn Fn(&Params, &Shared) -> String + Send>,
        }

        impl DecisionNode {
            fn new(
                name: impl Into<String>,
                condition: impl Fn(&Params, &Shared) -> String + Send + 'static
            ) -> Self {
                let base = BaseNode::new(name);
                let base = base$(.with_max_retries($retries))?$(.with_wait_duration($wait))?;
                Self {
                    base,
                    condition: Box::new(condition),
                }
            }
        }

        impl Node for DecisionNode {
            fn get_params(&self) -> &Params { &self.base.get_params() }
            fn set_params(&mut self, params: Params) { self.base.set_params(params); }
            fn add_successor(&mut self, action: String, successor: $crate::sync::NodeRef) {
                self.base.add_successor(action, successor);
            }
            fn get_successors(&self) -> &HashMap<String, $crate::sync::NodeRef> {
                self.base.get_successors()
            }
            fn get_successors_mut(&mut self) -> &mut HashMap<String, $crate::sync::NodeRef> {
                self.base.get_successors_mut()
            }
            fn get_name(&self) -> &str { self.base.get_name() }
            fn get_max_retries(&self) -> usize { self.base.get_max_retries() }
            fn get_wait_duration(&self) -> Duration { self.base.get_wait_duration() }
        }

        impl SyncNode for DecisionNode {
            fn post(&mut self, shared: &mut Shared, _prep_res: &Value, _exec_res: &Value) -> NodeResult<Value> {
                // Use the condition to determine the next action
                let decision = (self.condition)(self.get_params(), shared);
                Ok(Value::String(decision))
            }
        }

        $crate::sync::node(DecisionNode::new($name, $condition))
    }}
}

/// Macro for creating a simple processing chain
///
/// # Examples
///
/// ```rust
/// use rpocketflow::processing_chain;
/// use rpocketflow::NodeResult;
/// use serde_json::Value;
/// 
/// // Create a chain of processing steps
/// let processor = processing_chain! {
///     name: "DataProcessor",
///     steps: [
///         |data: &Value| -> NodeResult<Value> { /* transformation 1 */ Ok(data.clone()) },
///         |data: &Value| -> NodeResult<Value> { /* transformation 2 */ Ok(data.clone()) },
///         |data: &Value| -> NodeResult<Value> { /* transformation 3 */ Ok(data.clone()) }
///     ]
/// };
/// ```
#[macro_export]
macro_rules! processing_chain {
    (
        name: $name:expr,
        steps: [$($step:expr),+ $(,)?]
        $(, max_retries: $retries:expr)?
        $(, wait_duration: $wait:expr)?
        $(,)?
    ) => {{
        use $crate::sync::{BaseNode, Node, NodeResult, Params, Shared, SyncNode};
        use serde_json::Value;
        use std::collections::HashMap;
        use std::time::Duration;

        struct ProcessingChain {
            base: BaseNode,
            steps: Vec<Box<dyn Fn(&Value) -> NodeResult<Value> + Send>>,
        }

        impl ProcessingChain {
            fn new(name: impl Into<String>) -> Self {
                let base = BaseNode::new(name);
                let base = base$(.with_max_retries($retries))?$(.with_wait_duration($wait))?;
                Self {
                    base,
                    steps: Vec::new()
                }
            }

            fn add_step(&mut self, step: impl Fn(&Value) -> NodeResult<Value> + Send + 'static) {
                self.steps.push(Box::new(step));
            }
        }

        impl Node for ProcessingChain {
            fn get_params(&self) -> &Params { &self.base.get_params() }
            fn set_params(&mut self, params: Params) { self.base.set_params(params); }
            fn add_successor(&mut self, action: String, successor: $crate::sync::NodeRef) {
                self.base.add_successor(action, successor);
            }
            fn get_successors(&self) -> &HashMap<String, $crate::sync::NodeRef> {
                self.base.get_successors()
            }
            fn get_successors_mut(&mut self) -> &mut HashMap<String, $crate::sync::NodeRef> {
                self.base.get_successors_mut()
            }
            fn get_name(&self) -> &str { self.base.get_name() }
            fn get_max_retries(&self) -> usize { self.base.get_max_retries() }
            fn get_wait_duration(&self) -> Duration { self.base.get_wait_duration() }
        }

        impl SyncNode for ProcessingChain {
            fn prep(&mut self, shared: &mut Shared) -> NodeResult<Value> {
                // Get input data from shared state
                if let Some(input) = shared.get("input") {
                    Ok(input.clone())
                } else {
                    Ok(Value::Null)
                }
            }

            fn exec(&mut self, prep_res: &Value) -> NodeResult<Value> {
                let mut current = prep_res.clone();

                for step in &self.steps {
                    current = step(&current)?;
                }

                Ok(current)
            }
        }

        let mut chain = ProcessingChain::new($name);
        $(
            chain.add_step($step);
        )+

        $crate::sync::node(chain)
    }}
}
