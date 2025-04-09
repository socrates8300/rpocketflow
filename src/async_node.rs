#![allow(unused)]
use tracing::{info, warn, error, debug};
use crate::marker_traits::AsyncContext;
use crate::sync::{Action, Node, NodeRef, NodeResult, Shared, SyncNode};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

#[async_trait]
pub trait AsyncNode: Node + AsyncContext {
    async fn prep_async(&mut self, shared: &mut Shared) -> NodeResult<Value> {
        Ok(Value::Null)
    }

    async fn exec_async(&mut self, prep_res: &Value) -> NodeResult<Value> {
        debug!(
            target: "rpocketflow::async::node",
            node = %self.get_name(),
            "AsyncNode exec_async called"
        );
        Ok(Value::Null)
    }

    async fn post_async(
        &mut self,
        shared: &mut Shared,
        prep_res: &Value,
        exec_res: &Value,
    ) -> NodeResult<Value> {
        Ok(Value::Null)
    }

    async fn exec_fallback_async(&mut self, prep_res: &Value, err: String) -> NodeResult<Value> {
        Err(err)
    }

    async fn _exec_async(&mut self, prep_res: &Value) -> NodeResult<Value> {
        let max_retries = self.get_max_retries();
        for attempt_idx in 0..max_retries {
            let res = self.exec_async(prep_res).await;
            match res {
                Ok(val) => return Ok(val),
                Err(e) => {
                    if attempt_idx == max_retries - 1 {
                        return self.exec_fallback_async(prep_res, e).await;
                    } else {
                        let wait = self.get_wait_duration();
                        if wait > Duration::from_secs(0) {
                            debug!(
                                target: "rpocketflow::async::node",
                                node = %self.get_name(),
                                attempt = attempt_idx + 1,
                                max_attempts = max_retries,
                                wait_duration = ?wait,
                                "Node execution failed, retrying"
                            );
                            sleep(wait).await;
                        }
                    }
                }
            }
        }
        unreachable!()
    }

    async fn _run_async(&mut self, shared: &mut Shared) -> NodeResult<Action> {
        let prep_res = self.prep_async(shared).await?;
        let exec_res = self._exec_async(&prep_res).await?;
        let post_res = self.post_async(shared, &prep_res, &exec_res).await?;
        Ok(Action::from(&post_res))
    }

    async fn run_async(&mut self, shared: &mut Shared) -> NodeResult<Action> {
        if !self.get_successors().is_empty() {
            warn!(
                target: "rpocketflow::async::node",
                node = %self.get_name(),
                "Node won't run successors. Use AsyncFlow."
            );
        }
        self._run_async(shared).await
    }
}

pub struct AsyncNodeImpl {
    pub base: crate::sync::BaseNode,
}

impl AsyncNodeImpl {
    pub fn new(name: impl Into<String>) -> Self {
        AsyncNodeImpl {
            base: crate::sync::BaseNode::new(name),
        }
    }

    pub fn with_max_retries(mut self, max_retries: usize) -> Self {
        self.base = self.base.with_max_retries(max_retries);
        self
    }

    pub fn with_wait_duration(mut self, wait_duration: Duration) -> Self {
        self.base = self.base.with_wait_duration(wait_duration);
        self
    }
}

impl Node for AsyncNodeImpl {
    fn get_params(&self) -> &HashMap<String, Value> {
        self.base.get_params()
    }

    fn set_params(&mut self, params: HashMap<String, Value>) {
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

#[async_trait]
impl AsyncNode for AsyncNodeImpl {
    async fn prep_async(&mut self, shared: &mut Shared) -> NodeResult<Value> {
        // Simply delegate to the sync implementation for default behavior
        self.base.prep(shared)
    }

    async fn exec_async(&mut self, prep_res: &Value) -> NodeResult<Value> {
        debug!(
            target: "rpocketflow::async::node",
            node = %self.get_name(),
            "AsyncNodeImpl exec_async called"
        );
        Ok(Value::Null)
    }

    async fn post_async(
        &mut self,
        shared: &mut Shared,
        prep_res: &Value,
        exec_res: &Value,
    ) -> NodeResult<Value> {
        // Simply delegate to the sync implementation for default behavior
        self.base.post(shared, prep_res, exec_res)
    }
}

pub struct AsyncFlow {
    pub base: AsyncNodeImpl,
    pub start: NodeRef,
}

impl AsyncFlow {
    pub fn new(name: impl Into<String>, start: NodeRef) -> Self {
        AsyncFlow {
            base: AsyncNodeImpl::new(name),
            start,
        }
    }

    pub async fn orchestrate(
        &self,
        shared: &mut Shared,
        params_override: Option<HashMap<String, Value>>,
    ) -> NodeResult<()> {
        let p = params_override.unwrap_or_else(|| self.base.get_params().clone());
        let mut curr = self.start.clone();

        loop {
            let action = {
                let mut node = match curr.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => {
                        error!(
                            target: "rpocketflow::async::flow", 
                            node = %poisoned.to_string(),
                            "Mutex was poisoned. Recovering and continuing."
                        );
                        poisoned.into_inner()
                    }
                };

                node.set_params(p.clone());

                match node._run(shared) {
                    Ok(action) => {
                        if action == Action::Terminate {
                            info!(
                                target: "rpocketflow::async::flow",
                                flow = %self.base.get_name(),
                                node = %node.get_name(),
                                "Flow terminated by node"
                            );
                            return Ok(());
                        }

                        // Get the next node before dropping the lock
                        let action_str = action.to_string();
                        let next_node_option =
                            if let Some(next) = node.get_successors().get(&action_str) {
                                Some(next.clone())
                            } else if action != Action::Default {
                                if let Some(next) = node.get_successors().get("default") {
                                    Some(next.clone())
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                        // Release the lock before moving to the next node
                        drop(node);

                        if let Some(next_node) = next_node_option {
                            curr = next_node;
                            continue;
                        } else {
                            // No successor found, end the flow
                            info!(
                                target: "rpocketflow::async::flow",
                                flow = %self.base.get_name(),
                                "Flow ended naturally"
                            );
                            return Ok(());
                        }
                    }
                    Err(e) => {
                        error!(
                            target: "rpocketflow::async::flow",
                            flow = %self.base.get_name(),
                            error = %e,
                            "Error in flow execution"
                        );
                        return Err(format!("Flow error: {}", e));
                    }
                }
            };
        }
    }
}

impl Node for AsyncFlow {
    fn get_params(&self) -> &HashMap<String, Value> {
        self.base.get_params()
    }

    fn set_params(&mut self, params: HashMap<String, Value>) {
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

#[async_trait]
impl AsyncNode for AsyncFlow {
    async fn prep_async(&mut self, shared: &mut Shared) -> NodeResult<Value> {
        self.base.prep_async(shared).await
    }

    async fn exec_async(&mut self, _prep_res: &Value) -> NodeResult<Value> {
        Err("AsyncFlow doesn't support direct execution".to_string())
    }

    async fn post_async(
        &mut self,
        shared: &mut Shared,
        prep_res: &Value,
        _exec_res: &Value,
    ) -> NodeResult<Value> {
        self.base.post_async(shared, prep_res, &Value::Null).await
    }

    async fn _run_async(&mut self, shared: &mut Shared) -> NodeResult<Action> {
        let prep_res = self.prep_async(shared).await?;
        self.orchestrate(shared, None).await?;
        let post_result = self.post_async(shared, &prep_res, &Value::Null).await?;
        Ok(Action::from(&post_result))
    }
}
