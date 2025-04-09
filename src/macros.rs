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
