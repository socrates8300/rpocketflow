# RPocketFlow

A lightweight, flexible workflow orchestration library for Rust that enables defining and running complex agent-based workflows with minimal overhead.

## Overview

RPocketFlow provides both synchronous and asynchronous execution models for creating workflow graphs with nodes that can be connected and orchestrated. It's designed for building agent systems, task pipelines, and state machines with a focus on reliability and clarity.

## Features

- **Modular Node-Based Architecture**: Build complex workflows by connecting simple nodes
- **Both Synchronous and Async Support**: Run workflows synchronously or with Tokio-based async
- **Flow Control**: Branch and merge execution paths based on node outputs
- **Retry Mechanism**: Configurable retry policies with fallback handling
- **Shared State**: Pass data between nodes with a shared context
- **Builder Pattern**: Intuitive API for constructing workflows
- **Minimal Dependencies**: Core functionality requires only a few dependencies

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rpocketflow = "0.1.0"
```

Additionally, ensure you have the required dependencies:

```toml
[dependencies]
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }  # Only if using async features
async-trait = "0.1"  # Only if using async features
```

## Quick Start

### Basic Synchronous Flow

```rust
use rpocketflow::*;
use serde_json::Value;
use std::collections::HashMap;

// Define custom nodes
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
    fn get_successors(&self) -> &HashMap<String, NodeRef> { &self.base.successors }
    fn get_successors_mut(&mut self) -> &mut HashMap<String, NodeRef> { &mut self.base.successors }
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
    
    // Connect nodes
    then(&node1, node2.clone());
    then(&node2, node3.clone());
    
    // Create and run flow
    let flow = Flow::new("HelloFlow", node1);
    let mut shared = HashMap::new();
    
    match flow.orchestrate(&mut shared, None) {
        Ok(_) => println!("Flow completed successfully"),
        Err(e) => println!("Flow failed: {}", e),
    }
}
```

### Async Flow

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
    fn get_successors(&self) -> &HashMap<String, NodeRef> { &self.base.successors }
    fn get_successors_mut(&mut self) -> &mut HashMap<String, NodeRef> { &mut self.base.successors }
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
    // Create nodes
    let node1 = node(AsyncPrintNode::new("First", "Hello from async node 1", 100));
    let node2 = node(AsyncPrintNode::new("Second", "Hello from async node 2", 200));
    let node3 = node(AsyncPrintNode::new("Third", "Hello from async node 3", 150));
    
    // Connect nodes
    then(&node1, node2.clone());
    then(&node2, node3.clone());
    
    // Create async flow
    let flow = AsyncFlow::new("HelloAsyncFlow", node1);
    let mut shared = HashMap::new();
    
    match flow.orchestrate(&mut shared, None).await {
        Ok(_) => println!("Async flow completed successfully"),
        Err(e) => println!("Async flow failed: {}", e),
    }
}
```

## Core Concepts

### Nodes

Nodes are the basic building blocks of a workflow. Each node can:
- Prepare data (prep phase)
- Execute business logic (exec phase)
- Post-process results (post phase)

### Flows

Flows orchestrate the execution of nodes. They manage:
- The execution order based on node outputs
- Shared state between nodes
- Error handling and recovery

### Actions

Nodes can return actions to control flow direction:
- Default: Continue to the default successor
- Named: Follow a specific named path
- Terminate: End the flow immediately

### Shared State

The `Shared` type (a HashMap of JSON Values) allows nodes to pass data between each other.

## Advanced Usage

### Conditional Branching

```rust
// Create a decision node that returns different actions
let decision = node(DecisionNode::new("Decision"));

// Connect branches
when(&decision, "option1").then(option1_node.clone());
when(&decision, "option2").then(option2_node.clone());
```

### Retries and Error Handling

```rust
// Create a node with retry policy
let node = node(ApiCallNode::new("ApiCall")
    .with_max_retries(3)
    .with_wait_duration(Duration::from_secs(1)));
```

### Custom Node Implementation

```rust
pub struct CustomNode {
    base: BaseNode,
    // Custom fields
}

impl Node for CustomNode {
    // Implement Node trait methods
}

impl SyncNode for CustomNode {
    fn prep(&mut self, shared: &mut Shared) -> NodeResult<Value> {
        // Preparation logic
    }
    
    fn exec(&mut self, prep_res: &Value) -> NodeResult<Value> {
        // Execution logic
    }
    
    fn post(&mut self, shared: &mut Shared, prep_res: &Value, exec_res: &Value) -> NodeResult<Value> {
        // Post-processing logic
    }
}
```

## Architecture

RPocketFlow is designed around these key components:

- **Types** (`Params`, `Shared`, `NodeRef`, `NodeResult`): Core type definitions
- **Action**: Flow control enum (Default, Named, Terminate)
- **Node**: Base trait for all nodes
- **SyncNode**: Trait for synchronous execution
- **AsyncNode**: Trait for asynchronous execution
- **BaseNode**: Default implementation of Node
- **Flow/AsyncFlow**: Orchestration of node execution

## License

Licensed under the MIT License.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request
