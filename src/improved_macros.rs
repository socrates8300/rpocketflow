//! Enhanced macros for simplified workflow creation
//!
//! This module provides improved macros that make it easier to create nodes,
//! flows, and other workflow components with less boilerplate while maintaining
//! backward compatibility with the original macros.

/// Creates a node with a more flexible syntax that allows specifying different handlers
/// with less verbosity than the original node_impl! macro.
///
/// # Examples
///
/// ```rust
/// use rpocketflow::create_node;
/// use serde_json::json;
///
/// // Basic node with just an exec handler
/// let simple_node = create_node!("SimpleNode", |_| {
///     println!("Node executed!");
///     Ok(json!("done"))
/// });
///
/// // Node with prep, exec, and post handlers
/// let full_node = create_node!("FullNode", 
///     prep: |shared: &mut std::collections::HashMap<String, serde_json::Value>| {
///         shared.insert("prepared".to_string(), json!(true));
///         Ok(json!({"data": 42}))
///     },
///     exec: |prep_res: &serde_json::Value| {
///         let data = prep_res["data"].as_i64().unwrap_or(0);
///         Ok(json!(data * 2))
///     },
///     post: |shared: &mut std::collections::HashMap<String, serde_json::Value>, _prep_res: &serde_json::Value, exec_res: &serde_json::Value| {
///         shared.insert("result".to_string(), exec_res.clone());
///         Ok(json!("next_step"))
///     }
/// );
/// ```
#[macro_export]
macro_rules! create_node {
    // Simple version with just name and exec function
    ($name:expr, $exec_fn:expr) => {
        $crate::node_impl! {
            name: $name,
            exec: $exec_fn
        }
    };
    
    // Full version with named arguments
    ($name:expr, 
        prep: $prep_fn:expr,
        exec: $exec_fn:expr
        $(, post: $post_fn:expr)?
        $(, max_retries: $retries:expr)?
        $(, wait_duration: $wait:expr)?
    ) => {
        $crate::node_impl! {
            name: $name,
            prep: $prep_fn,
            exec: $exec_fn
            $(, post: $post_fn)?
            $(, max_retries: $retries)?
            $(, wait_duration: $wait)?
        }
    };
    
    // Version with just exec and options
    ($name:expr, 
        exec: $exec_fn:expr
        $(, max_retries: $retries:expr)?
        $(, wait_duration: $wait:expr)?
    ) => {
        $crate::node_impl! {
            name: $name,
            exec: $exec_fn
            $(, max_retries: $retries)?
            $(, wait_duration: $wait)?
        }
    };
}

/// Creates a simple sequential flow with less verbose syntax than the original flow! macro.
///
/// # Examples
///
/// ```rust
/// use rpocketflow::{create_node, sequential_flow};
/// use serde_json::json;
///
/// // Create some nodes
/// let node1 = create_node!("Step1", |_| Ok(json!("done")));
/// let node2 = create_node!("Step2", |_| Ok(json!("done")));
/// let node3 = create_node!("Step3", |_| Ok(json!("done")));
///
/// // Create a sequential flow
/// let flow = sequential_flow!("MyFlow", node1, node2, node3);
/// ```
#[macro_export]
macro_rules! sequential_flow {
    ($name:expr, $first:expr, $($rest:expr),+) => {
        $crate::flow! {
            name: $name,
            nodes: [$first, $($rest),+]
        }
    };
}

/// Creates a branching flow with less verbose syntax than the original flow! macro.
///
/// # Examples
///
/// ```rust
/// use rpocketflow::{create_node, branching_flow};
/// use serde_json::json;
///
/// // Create some nodes
/// let start = create_node!("Start", |_| Ok(json!("path_a")));
/// let path_a = create_node!("PathA", |_| Ok(json!("done")));
/// let path_b = create_node!("PathB", |_| Ok(json!("done")));
/// let end = create_node!("End", |_| Ok(json!("terminate")));
///
/// // Create a branching flow
/// let flow = branching_flow!("MyFlow", start => {
///     "path_a" => path_a => "default" => end,
///     "path_b" => path_b => "default" => end
/// });
/// ```
#[macro_export]
macro_rules! branching_flow {
    ($name:expr, $start:expr => {
        $($action:expr => $node:expr => $next_action:expr => $next_node:expr),+
        $(,)?
    }) => {
        $crate::flow! {
            name: $name,
            start: $start.clone(),
            connections: [
                $( ($start.clone(), $action, $node.clone()) ),+,
                $( ($node.clone(), $next_action, $next_node.clone()) ),+
            ]
        }
    };
}

/// Creates an async flow with a sequence of nodes.
///
/// # Examples
///
/// ```rust
/// use rpocketflow::{create_node, async_flow};
/// use serde_json::json;
///
/// // Create some nodes
/// let node1 = create_node!("Step1", |_| Ok(json!("done")));
/// let node2 = create_node!("Step2", |_| Ok(json!("done")));
///
/// // Create an async flow
/// let flow = async_flow!("MyAsyncFlow", node1, node2);
/// ```
#[macro_export]
macro_rules! async_flow {
    ($name:expr, $($node:expr),+) => {{
        let first_node = $crate::sync::node($crate::sync::BaseNode::new("__start__"));
        let mut flow = $crate::async_node::AsyncFlow::new($name, first_node.clone());
        
        let mut prev = first_node.clone();
        $(
            $crate::sync::then(&prev, $node.clone());
            prev = $node.clone();
        )+
        
        flow
    }};
}

/// Creates a simple decision node with improved syntax over the original decision_node! macro.
///
/// # Examples
///
/// ```rust
/// use rpocketflow::decide;
/// use serde_json::json;
///
/// // Create a decision node
/// let router = decide!("Router", |_params, shared| {
///     if let Some(value) = shared.get("direction") {
///         if value.as_str() == Some("left") {
///             "left_path".to_string()
///         } else if value.as_str() == Some("right") {
///             "right_path".to_string()
///         } else {
///             "default".to_string()
///         }
///     } else {
///         "error_path".to_string()
///     }
/// });
/// ```
#[macro_export]
macro_rules! decide {
    ($name:expr, $condition:expr) => {
        $crate::decision_node! {
            name: $name,
            condition: $condition
        }
    };
    
    ($name:expr, $condition:expr, max_retries: $retries:expr) => {
        $crate::decision_node! {
            name: $name,
            condition: $condition,
            max_retries: $retries
        }
    };
    
    ($name:expr, $condition:expr, wait_duration: $wait:expr) => {
        $crate::decision_node! {
            name: $name,
            condition: $condition,
            wait_duration: $wait
        }
    };
    
    ($name:expr, $condition:expr, max_retries: $retries:expr, wait_duration: $wait:expr) => {
        $crate::decision_node! {
            name: $name,
            condition: $condition,
            max_retries: $retries,
            wait_duration: $wait
        }
    };
}

/// Creates a processing pipeline that transforms data through multiple steps.
///
/// # Examples
///
/// ```rust
/// use rpocketflow::pipeline;
/// use serde_json::json;
///
/// // Create a data processing pipeline
/// let process = pipeline!("DataProcessor", 
///     // Step 1: Extract value
///     |data| {
///         let value = data["input"].as_i64().unwrap_or(0);
///         Ok(json!(value))
///     },
///     // Step 2: Double the value
///     |value| {
///         let doubled = value.as_i64().unwrap_or(0) * 2;
///         Ok(json!(doubled))
///     },
///     // Step 3: Format as object
///     |doubled: &serde_json::Value| {
///         let d = doubled.as_i64().unwrap_or(0);
///         Ok(json!({
///             "original": d / 2,
///             "result": d
///         }))
///     }
/// );
/// ```
#[macro_export]
macro_rules! pipeline {
    ($name:expr, $($step:expr),+) => {
        $crate::processing_chain! {
            name: $name,
            steps: [$($step),+]
        }
    };
    
    ($name:expr, $($step:expr),+ ; max_retries: $retries:expr) => {
        $crate::processing_chain! {
            name: $name,
            steps: [$($step),+],
            max_retries: $retries
        }
    };
    
    ($name:expr, $($step:expr),+ ; wait_duration: $wait:expr) => {
        $crate::processing_chain! {
            name: $name,
            steps: [$($step),+],
            wait_duration: $wait
        }
    };
}
