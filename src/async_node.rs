use crate::sync::{NodeRef, Params, Shared, SyncNode};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;

#[async_trait]
pub trait AsyncNode: Send {
    async fn prep_async(&mut self, shared: &mut Shared) -> Result<Value, String> {
        Ok(Value::Null)
    }
    async fn exec_async(&mut self, prep_res: &Value) -> Result<Value, String> {
        println!("AsyncNode exec_async called with prep_res: {}", prep_res);
        Ok(Value::Null)
    }
    async fn post_async(
        &mut self,
        shared: &mut Shared,
        prep_res: &Value,
        exec_res: &Value,
    ) -> Result<Value, String> {
        Ok(Value::Null)
    }
    async fn exec_fallback_async(
        &mut self,
        prep_res: &Value,
        err: String,
    ) -> Result<Value, String> {
        Err(err)
    }
    async fn _exec_async(&mut self, prep_res: &Value) -> Result<Value, String> {
        let max_retries = self.get_max_retries();
        for i in 0..max_retries {
            let res = self.exec_async(prep_res).await;
            match res {
                Ok(val) => return Ok(val),
                Err(e) => {
                    if i == max_retries - 1 {
                        return self.exec_fallback_async(prep_res, e).await;
                    } else {
                        let wait = self.get_wait();
                        if wait > Duration::from_secs(0) {
                            sleep(wait).await;
                        }
                    }
                }
            }
        }
        unreachable!()
    }
    async fn _run_async(&mut self, shared: &mut Shared) -> Result<Value, String> {
        let prep_res = self.prep_async(shared).await?;
        let exec_res = self._exec_async(&prep_res).await?;
        self.post_async(shared, &prep_res, &exec_res).await
    }
    async fn run_async(&mut self, shared: &mut Shared) -> Result<Value, String> {
        if !self.get_successors().is_empty() {
            println!("Warning: Node won't run successors. Use AsyncFlow.");
        }
        self._run_async(shared).await
    }

    // Required getters.
    fn get_max_retries(&self) -> usize;
    fn get_wait(&self) -> Duration;
    fn get_successors(&self) -> &HashMap<String, NodeRef>;
    fn get_successors_mut(&mut self) -> &mut HashMap<String, NodeRef>;
}

pub struct AsyncNodeImpl {
    pub base: crate::sync::BaseNode,
    pub max_retries: usize,
    pub wait: Duration,
}

#[async_trait]
impl AsyncNode for AsyncNodeImpl {
    async fn prep_async(&mut self, shared: &mut Shared) -> Result<Value, String> {
        self.base.prep(shared)
    }
    async fn exec_async(&mut self, prep_res: &Value) -> Result<Value, String> {
        println!(
            "AsyncNodeImpl exec_async called with prep_res: {}",
            prep_res
        );
        Ok(Value::Null)
    }
    async fn post_async(
        &mut self,
        shared: &mut Shared,
        prep_res: &Value,
        exec_res: &Value,
    ) -> Result<Value, String> {
        self.base.post(shared, prep_res, exec_res)
    }
    async fn exec_fallback_async(
        &mut self,
        prep_res: &Value,
        err: String,
    ) -> Result<Value, String> {
        Err(err)
    }
    fn get_max_retries(&self) -> usize {
        self.max_retries
    }
    fn get_wait(&self) -> Duration {
        self.wait
    }
    fn get_successors(&self) -> &HashMap<String, NodeRef> {
        &self.base.successors
    }
    fn get_successors_mut(&mut self) -> &mut HashMap<String, NodeRef> {
        &mut self.base.successors
    }
}
