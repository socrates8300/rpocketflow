use tracing::debug;
use serde_json::Value;
use std::thread;

use crate::marker_traits::SyncContext;
use super::action::Action;
use super::node::Node;
use super::types::{NodeResult, Shared};

/// Trait for synchronous nodes
pub trait SyncNode: Node + SyncContext {
    /// Preparation phase - fetches necessary data before execution
    fn prep(&mut self, _shared: &mut Shared) -> NodeResult<Value> {
        Ok(Value::Null)
    }

    /// Execution phase - performs the node's main functionality
    fn exec(&mut self, _prep_res: &Value) -> NodeResult<Value> {
        Ok(Value::Null)
    }

    /// Post-processing phase - processes results and updates shared state
    fn post(
        &mut self,
        _shared: &mut Shared,
        _prep_res: &Value,
        _exec_res: &Value,
    ) -> NodeResult<Value> {
        Ok(Value::Null)
    }

    /// Fallback execution when retries are exhausted
    fn exec_fallback(&mut self, _prep_res: &Value, err: String) -> NodeResult<Value> {
        Err(err)
    }

    /// Internal execution method with retry logic
    fn _exec(&mut self, prep_res: &Value) -> NodeResult<Value> {
        let max_retries = self.get_max_retries();

        let mut last_error = None;

        for attempt_idx in 0..max_retries {
            match self.exec(prep_res) {
                Ok(value) => return Ok(value),
                Err(e) => {
                    last_error = Some(e);

                    if attempt_idx + 1 < max_retries {
                        let wait = self.get_wait_duration();
                        if wait > std::time::Duration::from_secs(0) {
                            debug!(
                                target: "rpocketflow::sync::node",
                                node = %self.get_name(),
                                attempt = attempt_idx + 1,
                                max_attempts = max_retries,
                                wait_duration = ?wait,
                                "Node execution failed, retrying"
                            );
                            thread::sleep(wait);
                        }
                    }
                }
            }
        }

        // All retries failed, call fallback
        let err = last_error.unwrap_or_else(|| "Unknown error".to_string());
        self.exec_fallback(prep_res, err)
    }

    /// Internal method to run the node through all phases
    fn _run(&mut self, shared: &mut Shared) -> NodeResult<Action> {
        let prep_res = self.prep(shared)?;
        let exec_res = self._exec(&prep_res)?;
        let post_res = self.post(shared, &prep_res, &exec_res)?;

        Ok(Action::from(&post_res))
    }

    /// Run the node and return the next action
    fn run(&mut self, shared: &mut Shared) -> NodeResult<Action> {
        self._run(shared)
    }
}

