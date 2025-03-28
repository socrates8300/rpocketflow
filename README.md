### RPocketFlow

A lightweight, flexible workflow orchestration library for Rust that simplifies the creation and execution of complex agent-based workflows with minimal overhead.

#### Overview

RPocketFlow supports both synchronous and asynchronous execution models, enabling you to build modular workflows with interconnected nodes. Whether you’re implementing linear pipelines, branching paths, or complex state machines, RPocketFlow provides a robust framework that emphasizes clarity, minimal boilerplate, and efficient error handling.

#### Features

RPocketFlow offers a modular node-based architecture, making it easy to build and reuse workflow components. It supports both synchronous and asynchronous operations (using Tokio), intuitive flow control with branching and retry mechanisms, and shared state management for data passing between nodes. The builder-pattern API further minimizes boilerplate and helps you focus on your business logic, all while keeping dependencies to a minimum.

#### Installation

Add RPocketFlow to your `Cargo.toml`:

```toml
[dependencies]
rpocketflow = "0.1.0"
```

Also ensure you have the required dependencies for extended features:

```toml
[dependencies]
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }  # Required for async features
async-trait = "0.1"  # Required for asynchronous trait support
```

#### Quick Start

##### Basic Synchronous Flow

Below is an example of a simple synchronous flow using custom nodes:

```rust
use rpocketflow::*;
use serde_json::Value;
use std::collections::HashMap;

// Define a custom node that prints a message
struct PrintNode {
    base: BaseNode,
    message: String,
}

impl PrintNode {
    fn new(name: impl Into<String>, message: impl Into<String>) -> Self {
        PrintNode {
            base: BaseNode::new(name),
            message: message.into(),
        }
    }
}

impl Node for PrintNode {
    fn get_params(&self) -> &Params { &self.base.params }
    fn set_params(&mut self, params: Params) { self.base.params = params; }
    fn add_successor(&mut self, action: String, successor: NodeRef) { self.base.add_successor(action, successor); }
    fn get_successors(&self) -> &std::collections::HashMap<String, NodeRef> { &self.base.successors }
    fn get_successors_mut(&mut self) -> &mut std::collections::HashMap<String, NodeRef> { &mut self.base.successors }
    fn get_name(&self) -> &str { &self.base.name }
}

impl SyncNode for PrintNode {
    fn exec(&mut self, _prep_res: &Value) -> NodeResult<Value> {
        println!("{}: {}", self.get_name(), self.message);
        Ok(Value::Null)
    }
}

fn main() {
    // Create nodes
    let node1 = node(PrintNode::new("First", "Hello from node 1"));
    let node2 = node(PrintNode::new("Second", "Hello from node 2"));
    let node3 = node(PrintNode::new("Third", "Hello from node 3"));
    
    // Connect nodes sequentially
    then(&node1, node2.clone());
    then(&node2, node3.clone());
    
    // Create and run the flow
    let flow = Flow::new("HelloFlow", node1);
    let mut shared = std::collections::HashMap::new();
    
    match flow.orchestrate(&mut shared, None) {
        Ok(_) => println!("Flow completed successfully"),
        Err(e) => println!("Flow failed: {}", e),
    }
}
```

##### Asynchronous Flow

For workflows requiring asynchronous execution, consider this example:

```rust
use rpocketflow::*;
use serde_json::Value;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};
use async_trait::async_trait;

struct AsyncPrintNode {
    base: BaseNode,
    message: String,
    delay_ms: u64,
}

impl AsyncPrintNode {
    fn new(name: impl Into<String>, message: impl Into<String>, delay_ms: u64) -> Self {
        AsyncPrintNode {
            base: BaseNode::new(name),
            message: message.into(),
            delay_ms,
        }
    }
}

impl Node for AsyncPrintNode {
    fn get_params(&self) -> &Params { &self.base.params }
    fn set_params(&mut self, params: Params) { self.base.params = params; }
    fn add_successor(&mut self, action: String, successor: NodeRef) { self.base.add_successor(action, successor); }
    fn get_successors(&self) -> &std::collections::HashMap<String, NodeRef> { &self.base.successors }
    fn get_successors_mut(&mut self) -> &mut std::collections::HashMap<String, NodeRef> { &mut self.base.successors }
    fn get_name(&self) -> &str { &self.base.name }
}

impl SyncNode for AsyncPrintNode {}

#[async_trait]
impl AsyncNode for AsyncPrintNode {
    async fn exec_async(&mut self, _prep_res: &Value) -> NodeResult<Value> {
        sleep(Duration::from_millis(self.delay_ms)).await;
        println!("{}: {}", self.get_name(), self.message);
        Ok(Value::Null)
    }
}

#[tokio::main]
async fn main() {
    // Create async nodes
    let node1 = node(AsyncPrintNode::new("First", "Hello from async node 1", 100));
    let node2 = node(AsyncPrintNode::new("Second", "Hello from async node 2", 200));
    let node3 = node(AsyncPrintNode::new("Third", "Hello from async node 3", 150));
    
    // Connect nodes sequentially
    then(&node1, node2.clone());
    then(&node2, node3.clone());
    
    // Create and run the async flow
    let flow = AsyncFlow::new("HelloAsyncFlow", node1);
    let mut shared = HashMap::new();
    
    match flow.orchestrate(&mut shared, None).await {
        Ok(_) => println!("Async flow completed successfully"),
        Err(e) => println!("Async flow failed: {}", e),
    }
}
```

#### Macros for Simplified Usage

RPocketFlow includes several macros that help reduce boilerplate and streamline common tasks when defining nodes and flows.

##### Node Creation Macros

*Using `node_impl!` for Synchronous Nodes:*

```rust
// Create a simple node with just an execution function
let log_node = node_impl! {
    name: "Logger",
    exec: |_prep_res| {
        println!("Log message");
        Ok(Value::Null)
    }
};

// Create a more complex node with all lifecycle hooks
let processor = node_impl! {
    name: "DataProcessor",
    prep: |shared| {
        // Prepare data
        Ok(shared.get("input").unwrap_or(&Value::Null).clone())
    },
    exec: |prep_res| {
        // Process data
        let mut data = prep_res.clone();
        // ... processing logic
        Ok(data)
    },
    post: |shared, _prep_res, exec_res| {
        // Store results
        shared.insert("output".to_string(), exec_res.clone());
        Ok(Value::Null)
    },
    max_retries: 3,
    wait_duration: Duration::from_millis(100)
};
```

*Using `async_node_impl!` for Asynchronous Nodes:*

```rust
// Create an async node with async execution
let async_processor = async_node_impl! {
    name: "AsyncProcessor",
    exec_async: async |_prep_res| {
        // Simulate async work
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(Value::String("processed".to_string()))
    },
    max_retries: 3
};
```

##### Flow Creation Macros

*Defining Synchronous Flows with `flow!`:*

```rust
// Create a simple linear flow
let simple_flow = flow! {
    name: "SimpleFlow",
    nodes: [node1, node2, node3]
};

// Create a branching flow
let branching_flow = flow! {
    name: "BranchingFlow",
    start: decision_node,
    connections: [
        (decision_node, "path1", path1_node),
        (decision_node, "path2", path2_node),
        (path1_node, "default", end_node),
        (path2_node, "default", end_node)
    ]
};
```

*Defining Asynchronous Flows with `async_flow!`:*

```rust
// Create an async flow
let async_flow = async_flow! {
    name: "AsyncProcessingFlow",
    nodes: [fetch_node, process_node, save_node]
};
```

##### Specialized Node Macros

*Creating Decision Nodes with `decision_node!`:*

```rust
// Create a decision node based on shared state values
let router = decision_node! {
    name: "Router",
    condition: |params, shared| {
        if let Some(Value::Number(score)) = shared.get("score") {
            if score.as_f64().unwrap_or(0.0) > 0.8 {
                "high_priority"
            } else if score.as_f64().unwrap_or(0.0) > 0.5 {
                "medium_priority"
            } else {
                "low_priority"
            }
        } else {
            "unknown"
        }
    }
};
```

*Creating Processing Pipelines with `processing_chain!`:*

```rust
// Create a multi-step processing pipeline
let pipeline = processing_chain! {
    name: "Pipeline",
    steps: [
        |data| {
            // Step 1: Validate the data format
            if data.is_object() {
                Ok(data.clone())
            } else {
                Err("Invalid data format".to_string())
            }
        },
        |data| {
            // Step 2: Transform the data
            let mut result = data.clone();
            // ... transformation logic
            Ok(result)
        },
        |data| {
            // Step 3: Normalize the data
            let mut result = data.clone();
            // ... normalization logic
            Ok(result)
        }
    ],
    max_retries: 2
};
```

#### Core Concepts

At its core, RPocketFlow is built around the following ideas:

• **Nodes:** The basic building blocks that execute specific tasks and may include preparation, execution, and post-processing phases.  
• **Flows:** Structured networks of nodes that control the order of execution, error handling, and shared state management.  
• **Actions:** Outcomes from node execution that determine which successor node will execute next (default, named, or termination).  
• **Shared State:** A common context (typically a HashMap of JSON values) that enables nodes to exchange data.

#### Advanced Usage

For more complex scenarios, RPocketFlow supports advanced techniques:

*Conditional Branching:*

```rust
// Create a decision node that returns different actions based on conditions
let decision = node(DecisionNode::new("Decision"));

// Connect branches based on returned actions
when(&decision, "option1").then(option1_node.clone());
when(&decision, "option2").then(option2_node.clone());
```

*Retries and Error Handling:*

```rust
// Create a node with a retry policy
let node = node(ApiCallNode::new("ApiCall")
    .with_max_retries(3)
    .with_wait_duration(Duration::from_secs(1)));
```

*Custom Node Implementation:*

```rust
pub struct CustomNode {
    base: BaseNode,
    // Additional custom fields
}

impl Node for CustomNode {
    // Implement required methods for the Node trait
}

impl SyncNode for CustomNode {
    fn prep(&mut self, shared: &mut Shared) -> NodeResult<Value> {
        // Custom preparation logic
    }
    
    fn exec(&mut self, prep_res: &Value) -> NodeResult<Value> {
        // Custom execution logic
    }
    
    fn post(&mut self, shared: &mut Shared, prep_res: &Value, exec_res: &Value) -> NodeResult<Value> {
        // Custom post-processing logic
    }
}
```

#### Architecture

RPocketFlow is structured around key components:

• **Core Types:** Definitions such as `Params`, `Shared`, `NodeRef`, and `NodeResult` underpin the system.  
• **Flow Control:** The `Action` enum (with options like Default, Named, and Terminate) determines how workflows progress.  
• **Node Traits:** The `Node` trait is extended by `SyncNode` for synchronous operations and `AsyncNode` for asynchronous tasks, with `BaseNode` providing a default implementation.  
• **Workflow Orchestration:** `Flow` and `AsyncFlow` manage the execution order, shared state, and error recovery across nodes.

#### License

RPocketFlow is licensed under the MIT License.

#### Contributing

Contributions are welcome! To contribute:

1. Fork the repository.  
2. Create your feature branch (e.g., `git checkout -b feature/amazing-feature`).  
3. Commit your changes (e.g., `git commit -m 'Add some amazing feature'`).  
4. Push to your branch (e.g., `git push origin feature/amazing-feature`).  
5. Open a Pull Request.

Your contributions and feedback are greatly appreciated!

