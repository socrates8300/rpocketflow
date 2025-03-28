// No longer needed at top level if issues are fixed
// #![allow(unused)]
use crate::sync::types::AsyncNodeRef; // Import the new type
use crate::sync::{Action, Node, NodeResult, Shared, SyncNode}; // Keep SyncNode for AsyncNodeImpl's base
use async_trait::async_trait;
use once_cell::sync::Lazy; // Import Lazy for static initialization
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc; // Keep Arc but remove std::sync::Mutex
use std::time::Duration;
use tokio::sync::Mutex as TokioMutex; // Use TokioMutex for AsyncNodeRef
use tokio::time::sleep;

// Static empty map using once_cell for McpNode/MCPProtocolNode defaults
pub static EMPTY_ASYNC_SUCCESSORS: Lazy<HashMap<String, AsyncNodeRef>> = Lazy::new(HashMap::new);

#[async_trait]
pub trait AsyncNode: Node {
    // --- Existing Async Lifecycle Methods ---
    async fn prep_async(&mut self, _shared: &mut Shared) -> NodeResult<Value> {
        Ok(Value::Null)
    }

    async fn exec_async(&mut self, prep_res: &Value) -> NodeResult<Value> {
        log::debug!("AsyncNode exec_async called with prep_res: {}", prep_res);
        Ok(Value::Null)
    }

    async fn post_async(
        &mut self,
        _shared: &mut Shared,
        _prep_res: &Value,
        _exec_res: &Value,
    ) -> NodeResult<Value> {
        Ok(Value::Null)
    }

    async fn exec_fallback_async(
        &mut self,
        _prep_res: &Value,
        err: crate::errors::FlowError,
    ) -> NodeResult<Value> {
        Err(err)
    }

    // --- Internal Async Execution Logic (Retry) ---
    async fn _exec_async(&mut self, prep_res: &Value) -> NodeResult<Value> {
        let max_retries = self.get_max_retries();
        let node_name = self.get_name().to_string(); // Get name outside loop for logging

        for attempt in 0..max_retries {
            match self.exec_async(prep_res).await {
                Ok(val) => return Ok(val),
                Err(e) => {
                    if attempt + 1 >= max_retries {
                        log::error!(
                            "Node '{}' failed after {} attempts. Error: {}",
                            node_name,
                            max_retries,
                            e
                        );
                        // Call async fallback on the last attempt
                        return self.exec_fallback_async(prep_res, e).await;
                    } else {
                        let wait = self.get_wait_duration();
                        if wait > Duration::from_secs(0) {
                            log::warn!(
                                "Node '{}' execution failed, retrying in {:?} (attempt {}/{})",
                                node_name,
                                wait,
                                attempt + 1,
                                max_retries
                            );
                            sleep(wait).await;
                        } else {
                            log::warn!(
                                "Node '{}' execution failed, retrying immediately (attempt {}/{})",
                                node_name,
                                attempt + 1,
                                max_retries
                            );
                        }
                    }
                }
            }
        }

        // This part should ideally be unreachable if max_retries >= 1 due to the logic above
        // If max_retries is 0, the loop doesn't run, and we hit this.
        Err(crate::errors::FlowError::NodeExecution(format!(
            "Node '{}' failed execution (max_retries={})",
            node_name, max_retries
        )))
    }

    // --- Internal Async Run (Lifecycle Chaining) ---
    async fn _run_async(&mut self, shared: &mut Shared) -> NodeResult<Action> {
        let prep_res = self.prep_async(shared).await?;
        // Important: Clone prep_res if it needs to be passed to post_async unmodified
        // as _exec_async might mutate state based on it. Using refs is fine if _exec_async is read-only WRT prep_res.
        let exec_res = self._exec_async(&prep_res).await?;
        let post_res = self.post_async(shared, &prep_res, &exec_res).await?;
        Ok(Action::from(&post_res))
    }

    // --- Public Async Run Method (Optional Direct Use) ---
    async fn run_async(&mut self, shared: &mut Shared) -> NodeResult<Action> {
        if !self.get_async_successors().is_empty() {
            log::warn!("Warning: Node '{}' won't run successors when run_async is called directly. Use AsyncFlow for orchestration.", self.get_name());
        }
        self._run_async(shared).await
    }

    // --- NEW: Methods for Managing Async Successors ---
    fn add_async_successor(&mut self, action: String, successor: AsyncNodeRef);
    fn get_async_successors(&self) -> &HashMap<String, AsyncNodeRef>;
    fn get_async_successors_mut(&mut self) -> &mut HashMap<String, AsyncNodeRef>;
}

// =============================================
//      AsyncNodeImpl: Base Async Node
// =============================================
pub struct AsyncNodeImpl {
    pub base: crate::sync::BaseNode,
    pub async_successors: HashMap<String, AsyncNodeRef>, // Store async successors separately
}

impl AsyncNodeImpl {
    pub fn new(name: impl Into<String>) -> Self {
        AsyncNodeImpl {
            base: crate::sync::BaseNode::new(name),
            async_successors: HashMap::new(),
        }
    }

    // Builder methods chain correctly
    pub fn with_max_retries(mut self, max_retries: usize) -> Self {
        self.base = self.base.with_max_retries(max_retries);
        self
    }

    pub fn with_wait_duration(mut self, wait_duration: Duration) -> Self {
        self.base = self.base.with_wait_duration(wait_duration);
        self
    }
}

// Implement the base `Node` trait, delegating common parts to `base`
impl Node for AsyncNodeImpl {
    fn get_params(&self) -> &HashMap<String, Value> {
        self.base.get_params()
    }
    fn set_params(&mut self, params: HashMap<String, Value>) {
        self.base.set_params(params);
    }
    // These refer to *sync* successors stored in BaseNode, potentially unused in async flows
    fn add_successor(&mut self, action: String, successor: crate::sync::NodeRef) {
        self.base.add_successor(action, successor);
    }
    fn get_successors(&self) -> &HashMap<String, crate::sync::NodeRef> {
        self.base.get_successors()
    }
    fn get_successors_mut(&mut self) -> &mut HashMap<String, crate::sync::NodeRef> {
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

// Implement the `AsyncNode` trait
#[async_trait]
impl AsyncNode for AsyncNodeImpl {
    // Default async lifecycle methods can delegate to sync if appropriate,
    // but exec_async needs a real implementation in concrete types or override.
    async fn prep_async(&mut self, shared: &mut Shared) -> NodeResult<Value> {
        // Delegate to sync prep by default
        self.base.prep(shared)
    }

    async fn exec_async(&mut self, prep_res: &Value) -> NodeResult<Value> {
        // Default async exec does nothing, should be overridden by users
        log::debug!(
            "AsyncNodeImpl exec_async called for '{}' with prep_res: {}",
            self.get_name(),
            prep_res
        );
        Ok(Value::Null)
    }

    async fn post_async(
        &mut self,
        shared: &mut Shared,
        prep_res: &Value,
        exec_res: &Value,
    ) -> NodeResult<Value> {
        // Delegate to sync post by default
        self.base.post(shared, prep_res, exec_res)
    }

    // Implement the new async successor methods
    fn add_async_successor(&mut self, action: String, successor: AsyncNodeRef) {
        if self.async_successors.contains_key(&action) {
            log::warn!(
                "AsyncNode '{}': Overwriting async successor for action '{}'",
                self.get_name(),
                action
            );
        }
        self.async_successors.insert(action, successor);
    }

    fn get_async_successors(&self) -> &HashMap<String, AsyncNodeRef> {
        &self.async_successors
    }

    fn get_async_successors_mut(&mut self) -> &mut HashMap<String, AsyncNodeRef> {
        &mut self.async_successors
    }
}

// =============================================
//          AsyncFlow: Orchestrator
// =============================================
pub struct AsyncFlow {
    // Use AsyncNodeImpl for base properties, allows AsyncFlow itself to have retry config etc.
    pub base: AsyncNodeImpl,
    pub start: AsyncNodeRef, // Start node MUST be an AsyncNodeRef
}

impl AsyncFlow {
    pub fn new(name: impl Into<String>, start: AsyncNodeRef) -> Self {
        AsyncFlow {
            base: AsyncNodeImpl::new(name), // Initialize base
            start,
        }
    }

    // Helper to find the next *async* node based on action
    fn get_next_async_node(
        node_guard: &(impl AsyncNode + ?Sized), // Use generic bound
        action: &Action,
    ) -> Option<AsyncNodeRef> {
        let action_str = action.to_string();
        let successors = node_guard.get_async_successors();

        // Try specific action first
        if let Some(next) = successors.get(&action_str) {
            return Some(next.clone());
        }

        // Fallback to default action
        if action != &Action::Default {
            if let Some(next) = successors.get("default") {
                return Some(next.clone());
            }
        }

        // Log if successors exist but the action wasn't found
        if !successors.is_empty() {
            log::warn!(
                "AsyncFlow '{}': Action '{}' not found in async successors of node '{}'. Available actions: {:?}",
                "?", // Need flow name here, maybe pass self?
                action,
                node_guard.get_name(),
                successors.keys().collect::<Vec<_>>()
            );
        }

        None
    }

    /// Orchestrate the asynchronous flow execution
    pub async fn orchestrate(
        &self, // Keep immutable borrow to self for flow name/params if needed
        shared: &mut Shared,
        params_override: Option<HashMap<String, Value>>,
    ) -> NodeResult<()> {
        // Use flow's own parameters if no override provided
        let flow_params = self.base.get_params();
        let p = params_override.unwrap_or_else(|| flow_params.clone());
        let flow_name = self.base.get_name().to_string(); // Get flow name for logging

        let mut curr = self.start.clone();

        loop {
            let next_node_option: Option<AsyncNodeRef>;
            
            // Clone curr before locking to avoid borrowing issues
            let current_node = curr.clone();

            // Scope for the TokioMutexGuard (which IS Send)
            // Acquire lock asynchronously on the clone
            let mut node_guard = current_node.lock().await; // Use .lock().await with TokioMutex

            let node_name = node_guard.get_name().to_string();
            log::debug!("AsyncFlow '{}': Executing node '{}'", flow_name, node_name);

            node_guard.set_params(p.clone());

            // Execute the node asynchronously - With TokioMutex guard can be held across .await points
            match node_guard._run_async(shared).await {
                Ok(action) => {
                    log::debug!("AsyncFlow '{}': Node '{}' finished with action: {}", flow_name, node_name, action);
                    if action == Action::Terminate {
                        log::info!("AsyncFlow '{}' terminated by node '{}'", flow_name, node_name);
                        // MutexGuard dropped automatically at end of scope
                        return Ok(());
                    }
                    
                    // Get the next node using the same guard
                    next_node_option = Self::get_next_async_node(&*node_guard, &action);
                    // MutexGuard dropped automatically at end of scope
                }
                Err(e) => {
                    log::error!("Error during execution of node '{}' in AsyncFlow '{}': {}", node_name, flow_name, e);
                    // MutexGuard dropped automatically at end of scope
                    return Err(crate::errors::FlowError::FlowOrchestration(format!(
                        "AsyncFlow '{}' failed at node '{}': {}",
                        flow_name, node_name, e
                    )));
                }
            }
            // TokioMutexGuard is dropped here when exiting the scope

            // --- Post-Lock Logic ---
            if let Some(next_node) = next_node_option {
                curr = next_node;
            } else {
                log::info!("AsyncFlow '{}' ended naturally after node '{}'", flow_name, node_name);
                return Ok(());
            }
        } // End loop
    } // End orchestrate
}

// Implement Node for AsyncFlow, delegating to its base AsyncNodeImpl
impl Node for AsyncFlow {
    fn get_params(&self) -> &HashMap<String, Value> {
        self.base.get_params()
    }
    fn set_params(&mut self, params: HashMap<String, Value>) {
        self.base.set_params(params);
    }
    // Delegate sync successors (less relevant for AsyncFlow itself)
    fn add_successor(&mut self, action: String, successor: crate::sync::NodeRef) {
        self.base.add_successor(action, successor);
    }
    fn get_successors(&self) -> &HashMap<String, crate::sync::NodeRef> {
        self.base.get_successors()
    }
    fn get_successors_mut(&mut self) -> &mut HashMap<String, crate::sync::NodeRef> {
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

// Implement AsyncNode for AsyncFlow
#[async_trait]
impl AsyncNode for AsyncFlow {
    // Delegate basic async lifecycle methods to base impl (can be overridden if needed)
    async fn prep_async(&mut self, shared: &mut Shared) -> NodeResult<Value> {
        self.base.prep_async(shared).await
    }

    // An AsyncFlow cannot be executed directly like a simple node
    async fn exec_async(&mut self, _prep_res: &Value) -> NodeResult<Value> {
        Err(crate::errors::FlowError::NodeExecution(format!(
            "AsyncFlow '{}' cannot be executed directly via exec_async. Use orchestrate.",
            self.get_name()
        )))
    }

    async fn post_async(
        &mut self,
        shared: &mut Shared,
        prep_res: &Value,
        _exec_res: &Value, // exec_res is implicitly Null from the _run_async implementation
    ) -> NodeResult<Value> {
        // Pass Null as exec_res as the flow's own execution doesn't produce a direct result value
        self.base.post_async(shared, prep_res, &Value::Null).await
    }

    // Implement the flow's own run logic: prep, orchestrate, post
    async fn _run_async(&mut self, shared: &mut Shared) -> NodeResult<Action> {
        // Clone anything needed from self to avoid borrowing self across await points
        let own_params = self.get_params().clone();
        
        // First we need to do prep phase
        let prep_result = {
            // Call prep_async but don't let the borrow of self extend beyond this block
            self.prep_async(shared).await?
        };
        
        // Then orchestrate phase
        self.orchestrate(shared, Some(own_params.clone())).await?;
        
        // Finally post phase - use a separate block to limit self borrow
        let post_result = {
            // Call post_async but don't let the borrow of self extend
            self.post_async(shared, &prep_result, &Value::Null).await?
        };
        
        // Return the action determined by post logic
        Ok(Action::from(&post_result))
    }

    // Delegate async successor management to base AsyncNodeImpl
    fn add_async_successor(&mut self, action: String, successor: AsyncNodeRef) {
        self.base.add_async_successor(action, successor);
    }
    fn get_async_successors(&self) -> &HashMap<String, AsyncNodeRef> {
        self.base.get_async_successors()
    }
    fn get_async_successors_mut(&mut self) -> &mut HashMap<String, AsyncNodeRef> {
        self.base.get_async_successors_mut()
    }
}

// =============================================
//          Async Helpers
// =============================================

/// Create an AsyncNodeRef from an AsyncNode implementation
pub fn async_node<T: AsyncNode + Send + 'static>(node: T) -> AsyncNodeRef {
    Arc::new(TokioMutex::new(node)) // Use TokioMutex::new
}

/// Helper function to chain async nodes using the default action
pub async fn async_then(from: &AsyncNodeRef, to: AsyncNodeRef) -> AsyncNodeRef {
    from.lock().await  // Lock asynchronously
        .add_async_successor("default".to_string(), to.clone());
    to
}

/// Struct and helper for adding a conditional async transition
pub struct AsyncConditionalTransition {
    pub src: AsyncNodeRef,
    pub action: String,
}

impl AsyncConditionalTransition {
    /// Chain the async transition to a target node
    pub async fn then(self, tgt: AsyncNodeRef) -> AsyncNodeRef {
        self.src
            .lock().await  // Lock asynchronously
            .add_async_successor(self.action, tgt.clone());
        tgt
    }
}

/// Helper function to create a conditional async transition
pub fn async_when(node: &AsyncNodeRef, action: impl Into<String>) -> AsyncConditionalTransition {
    AsyncConditionalTransition {
        src: node.clone(),
        action: action.into(),
    }
}
