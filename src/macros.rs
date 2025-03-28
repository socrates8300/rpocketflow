/// Macro to simplify creating a new node with minimal boilerplate
///
/// # Examples
///
/// ```rust
/// // Create a simple node
/// let my_node = node_impl! {
///     name: "MyNode",
///     exec: |_prep_res| {
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

        struct GeneratedNode {
            base: BaseNode,
        }

        impl GeneratedNode {
            fn new(name: impl Into<String>) -> Self {
                let mut base = BaseNode::new(name);
                $(base = base.with_max_retries($retries);)?
                $(base = base.with_wait_duration($wait);)?
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
                $(return ($prep_fn)(shared);)?
                Ok(Value::Null)
            }

            fn exec(&mut self, prep_res: &Value) -> NodeResult<Value> {
                ($exec_fn)(prep_res)
            }

            #[allow(unused_variables)]
            fn post(&mut self, shared: &mut Shared, prep_res: &Value, exec_res: &Value) -> NodeResult<Value> {
                $(return ($post_fn)(shared, prep_res, exec_res);)?
                Ok(Value::Null)
            }
        }

        $crate::sync::node(GeneratedNode::new($name))
    }}
}

/// Macro to simplify creating an async node with minimal boilerplate
///
/// # Examples
///
/// ```rust
/// // Create a simple async node
/// let my_async_node = async_node_impl! {
///     name: "MyAsyncNode",
///     exec_async: async |_prep_res| {
///         tokio::time::sleep(Duration::from_millis(100)).await;
///         println!("Async node is executing!");
///         Ok(Value::Null)
///     }
/// };
/// ```
#[macro_export]
macro_rules! async_node_impl {
    (
        name: $name:expr,
        $(prep_async: $prep_fn:expr,)?
        exec_async: $exec_fn:expr
        $(, post_async: $post_fn:expr)?
        $(, max_retries: $retries:expr)?
        $(, wait_duration: $wait:expr)?
        $(,)?
    ) => {{
        use $crate::sync::{BaseNode, Node, NodeResult, Params, Shared, SyncNode};
        use $crate::async_node::AsyncNode;
        use serde_json::Value;
        use std::collections::HashMap;
        use std::time::Duration;
        use async_trait::async_trait;

        struct GeneratedAsyncNode {
            base: BaseNode,
        }

        impl GeneratedAsyncNode {
            fn new(name: impl Into<String>) -> Self {
                let mut base = BaseNode::new(name);
                $(base = base.with_max_retries($retries);)?
                $(base = base.with_wait_duration($wait);)?
                Self { base }
            }
        }

        impl Node for GeneratedAsyncNode {
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

        impl SyncNode for GeneratedAsyncNode {}

        #[async_trait]
        impl AsyncNode for GeneratedAsyncNode {
            #[allow(unused_variables)]
            async fn prep_async(&mut self, shared: &mut Shared) -> NodeResult<Value> {
                $(return ($prep_fn)(shared).await;)?
                Ok(Value::Null)
            }

            async fn exec_async(&mut self, prep_res: &Value) -> NodeResult<Value> {
                ($exec_fn)(prep_res).await
            }

            #[allow(unused_variables)]
            async fn post_async(&mut self, shared: &mut Shared, prep_res: &Value, exec_res: &Value) -> NodeResult<Value> {
                $(return ($post_fn)(shared, prep_res, exec_res).await;)?
                Ok(Value::Null)
            }
        }

        $crate::sync::node(GeneratedAsyncNode::new($name))
    }}
}

/// Macro to create a simple flow with connected nodes
///
/// # Examples
///
/// ```rust
/// // Create a linear flow
/// let flow = flow! {
///     name: "SimpleFlow",
///     nodes: [node1, node2, node3]
/// };
///
/// // Create a more complex flow with branches
/// let flow = flow! {
///     name: "BranchingFlow",
///     start: start_node,
///     connections: [
///         (start_node, "path1", path1_node),
///         (start_node, "path2", path2_node),
///         (path1_node, "default", end_node),
///         (path2_node, "default", end_node)
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

/// Macro to create an async flow with connected nodes
///
/// # Examples
///
/// ```rust
/// // Create a linear async flow
/// let flow = async_flow! {
///     name: "SimpleAsyncFlow",
///     nodes: [node1, node2, node3]
/// };
/// ```
#[macro_export]
macro_rules! async_flow {
    // Simple linear flow
    (
        name: $name:expr,
        nodes: [$first:expr $(, $rest:expr)+]
    ) => {{
        use $crate::sync::then;
        use $crate::async_node::AsyncFlow;

        let first_node = $first;
        $(
            let _ = then(&first_node, $rest);
        )*

        AsyncFlow::new($name, first_node)
    }};

    // Flow with explicit connections
    (
        name: $name:expr,
        start: $start:expr,
        connections: [$(($from:expr, $action:expr, $to:expr)),+ $(,)?]
    ) => {{
        use $crate::sync::when;
        use $crate::async_node::AsyncFlow;

        let start_node = $start;
        $(
            let _ = when(&$from, $action).then($to);
        )+

        AsyncFlow::new($name, start_node)
    }};
}

/// Macro to create a simple decision node
///
/// # Examples
///
/// ```rust
/// // Create a decision node based on a condition
/// let decision_node = decision_node! {
///     name: "RouteDecision",
///     condition: |params, shared| {
///         if let Some(Value::Number(age)) = shared.get("age") {
///             if age.as_u64().unwrap_or(0) >= 18 {
///                 "adult"
///             } else {
///                 "minor"
///             }
///         } else {
///             "unknown"
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
                let mut base = BaseNode::new(name);
                $(base = base.with_max_retries($retries);)?
                $(base = base.with_wait_duration($wait);)?
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
/// // Create a chain of processing steps
/// let processor = processing_chain! {
///     name: "DataProcessor",
///     steps: [
///         |data| { /* transformation 1 */ Ok(data) },
///         |data| { /* transformation 2 */ Ok(data) },
///         |data| { /* transformation 3 */ Ok(data) }
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
                let mut base = BaseNode::new(name);
                $(base = base.with_max_retries($retries);)?
                $(base = base.with_wait_duration($wait);)?
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
