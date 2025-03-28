#![allow(unused)]
/// Macro to simplify creating a new node with minimal boilerplate
///
/// # Examples
///
/// ```rust
/// use rpocketflow::node_impl;
/// use serde_json::Value;
/// use log::info;
///
/// // Create a simple node
/// let my_node = node_impl! {
///     name: "MyNode",
///     exec: |_prep_res: &Value| {
///         info!("Node is executing!");
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

/// Macro to create a simple flow with connected nodes
///
/// # Examples
///
/// ```rust
/// use rpocketflow::flow;
/// use rpocketflow::node_impl;
/// use serde_json::Value;
///
/// // First, create some nodes to use in our flow
/// let node1 = node_impl! { name: "Node1", exec: |_: &Value| Ok(Value::Null) };
/// let node2 = node_impl! { name: "Node2", exec: |_: &Value| Ok(Value::Null) };
/// let node3 = node_impl! { name: "Node3", exec: |_: &Value| Ok(Value::Null) };
///
/// // Create a linear flow
/// let flow = flow! {
///     name: "SimpleFlow",
///     nodes: [node1, node2, node3]
/// };
/// ```
///
/// ```rust
/// use rpocketflow::flow;
/// use rpocketflow::node_impl;
/// use serde_json::Value;
///
/// // First, create some nodes
/// let start_node = node_impl! { name: "Start", exec: |_: &Value| Ok(Value::Null) };
/// let path1_node = node_impl! { name: "Path1", exec: |_: &Value| Ok(Value::Null) };
/// let path2_node = node_impl! { name: "Path2", exec: |_: &Value| Ok(Value::Null) };
/// let end_node = node_impl! { name: "End", exec: |_: &Value| Ok(Value::Null) };
///
/// // Create a more complex flow with branches
/// let flow = flow! {
///     name: "BranchingFlow",
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

/// Macro to create an MCP client node with protocol-level implementation
///
/// # Examples
///
/// ```
/// // This is a pseudo-code example. In real code, the macro would be used like this:
/// //
/// // use rpocketflow::mcp_protocol_node_macro;
/// // use serde_json::json;
/// //
/// // // Create an MCP client that launches a specific server command
/// // let mcp_node = mcp_protocol_node_macro! {
/// //     name: "MCP Client",
/// //     server_command: "/path/to/mcp_server",
/// //     server_args: ["--config", "config.json"],
/// //     client_name: "My Client",
/// //     client_version: "1.0.0"
/// // };
/// ```
#[macro_export]
macro_rules! mcp_protocol_node_macro {
    (
        name: $name:expr,
        server_command: $server_cmd:expr
        $(, server_args: [$($arg:expr),*])?
        $(, client_name: $client_name:expr)?
        $(, client_version: $client_version:expr)?
        $(,)?
    ) => {{
        // First, create the configuration
        let mut config = $crate::MCPClientConfig::new(
            $($client_name)?.unwrap_or_else(|| "RPocketFlow Client"),
            $($client_version)?.unwrap_or_else(|| env!("CARGO_PKG_VERSION"))
        );

        // Add server command and args
        let args = vec![$($($arg.to_string()),*)?];
        config = config.with_server_command($server_cmd, args);

        // Create and return the node
        $crate::mcp_protocol_node($name, config)
    }};

    // Variation without server command (for future use with non-process servers)
    (
        name: $name:expr
        $(, client_name: $client_name:expr)?
        $(, client_version: $client_version:expr)?
        $(,)?
    ) => {{
        // Create the configuration
        let config = $crate::MCPClientConfig::new(
            $($client_name)? .unwrap_or_else(|| "RPocketFlow Client"),
            $($client_version)? .unwrap_or_else(|| env!("CARGO_PKG_VERSION"))
        );

        // Create and return the node
        $crate::mcp_protocol_node($name, config)
    }};
}

/// Macro to create a Claude MCP node
///
/// # Examples
///
/// ```rust
/// use rpocketflow::claude_node_macro;
///
/// // Create a Claude node with API key and model
/// let claude_node = claude_node_macro! {
///     name: "Claude",
///     api_key: "YOUR_API_KEY",
///     model: "claude-3-sonnet-20240229",
///     system_prompt: "You are a helpful assistant."
/// };
/// ```
#[macro_export]
macro_rules! claude_node_macro {
    (
        name: $name:expr,
        api_key: $api_key:expr,
        model: $model:expr
        $(, system_prompt: $system_prompt:expr)?
        $(, max_tokens: $max_tokens:expr)?
        $(, temperature: $temperature:expr)?
        $(,)?
    ) => {{
        // First, create the configuration
        let mut config = $crate::McpConfig::new($api_key, $model);

        // Add optional parameters
        $(config = config.with_system_prompt($system_prompt);)?
        $(config = config.with_max_tokens($max_tokens);)?
        $(config = config.with_temperature($temperature);)?

        // Create and return the node
        $crate::mcp_node($name, config)
    }};
}

/// Macro to easily create a tool handler node for MCP tools
///
/// # Examples
///
/// ```rust
/// use rpocketflow::mcp_tool_handler;
/// use serde_json::json;
///
/// // Create a tool handler for a weather tool
/// let weather_handler = mcp_tool_handler! {
///     name: "WeatherHandler",
///     tool_name: "get_weather",
///     handler: |params| {
///         let location = params.get("location")
///             .and_then(|v| v.as_str())
///             .unwrap_or("unknown");
///             
///         // In real code, you would call a weather API here
///         Ok(json!({
///             "temperature": 72,
///             "condition": "sunny",
///             "location": location
///         }))
///     }
/// };
/// ```
#[macro_export]
macro_rules! mcp_tool_handler {
    (
        name: $name:expr,
        tool_name: $tool_name:expr,
        handler: $handler:expr
        $(,)?
    ) => {{
        use $crate::sync::{BaseNode, Node, NodeResult, Params, Shared, SyncNode};
        use serde_json::{Value, json};
        use std::collections::HashMap;

        struct ToolHandlerNode {
            base: BaseNode,
            tool_name: String,
            handler: Box<dyn Fn(&Value) -> Result<Value, String> + Send>,
        }

        impl ToolHandlerNode {
            fn new(
                name: impl Into<String>,
                tool_name: impl Into<String>,
                handler: impl Fn(&Value) -> Result<Value, String> + Send + 'static
            ) -> Self {
                Self {
                    base: BaseNode::new(name),
                    tool_name: tool_name.into(),
                    handler: Box::new(handler),
                }
            }
        }

        impl Node for ToolHandlerNode {
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
        }

        impl SyncNode for ToolHandlerNode {
            fn exec(&mut self, prep_res: &Value) -> NodeResult<Value> {
                match (self.handler)(prep_res) {
                    Ok(result) => Ok(json!({
                        "tool_name": self.tool_name,
                        "result": result,
                        "status": "success"
                    })),
                    Err(err) => Ok(json!({
                        "tool_name": self.tool_name,
                        "error": err,
                        "status": "error"
                    }))
                }
            }

            fn post(&mut self, shared: &mut Shared, _prep_res: &Value, exec_res: &Value) -> NodeResult<Value> {
                // Store the result in shared state with a standard key pattern
                shared.insert(format!("mcp_tool_result_{}", self.tool_name), exec_res.clone());

                // Return the appropriate action based on status
                let status = exec_res.get("status").and_then(|s| s.as_str()).unwrap_or("unknown");
                Ok(json!(status))
            }
        }

        $crate::sync::node(ToolHandlerNode::new($name, $tool_name, $handler))
    }};
}
