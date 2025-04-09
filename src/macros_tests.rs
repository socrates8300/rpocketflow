#![allow(unused)]
#[cfg(test)]
mod tests {
    use crate::*;
    use serde_json::json;
    use std::collections::HashMap;
    use std::time::Duration;

    // Helper function to create a shared state with test data
    fn test_shared() -> Shared {
        let mut shared = HashMap::new();
        shared.insert("input".to_string(), json!({"value": 10}));
        shared.insert("score".to_string(), json!(0.75));
        shared
    }

    #[test]
    fn test_node_impl_macro() {
        // Test basic node creation
        let basic_node = node_impl! {
            name: "BasicNode",
            exec: |_prep_res: &Value| {
                Ok(json!("executed"))
            }
        };

        // Test full lifecycle node creation
        let full_node = node_impl! {
            name: "FullNode",
            prep: |shared: &mut Shared| {
                shared.insert("from_prep".to_string(), json!("prepped"));
                Ok(json!({"prep_data": true}))
            },
            exec: |prep_res: &Value| {
                assert!(prep_res.as_object().unwrap().contains_key("prep_data"));
                Ok(json!({"exec_result": true}))
            },
            post: |shared: &mut Shared, prep_res: &Value, exec_res: &Value| {
                assert!(prep_res.as_object().unwrap().contains_key("prep_data"));
                assert!(exec_res.as_object().unwrap().contains_key("exec_result"));
                shared.insert("from_post".to_string(), json!("posted"));
                Ok(json!("done"))
            },
            max_retries: 2,
            wait_duration: Duration::from_millis(10)
        };

        // Run tests
        let mut shared = test_shared();

        // Test basic node
        let basic_result = basic_node.lock().unwrap().run(&mut shared).unwrap();
        assert_eq!(basic_result, Action::from_str("default"));

        // Test full lifecycle node
        let full_result = full_node.lock().unwrap().run(&mut shared).unwrap();

        // Check shared state was updated correctly
        assert_eq!(shared.get("from_prep").unwrap().as_str(), Some("prepped"));
        assert_eq!(shared.get("from_post").unwrap().as_str(), Some("posted"));
        assert_eq!(full_result, Action::from_str("done"));
    }

    #[test]
    fn test_flow_macro_linear() {
        // Create test nodes
        let node1 = node_impl! {
            name: "Node1",
            exec: |_: &Value| {
                Ok(json!("node1_executed"))
            }
        };

        let node2 = node_impl! {
            name: "Node2",
            exec: |_: &Value| {
                Ok(json!("node2_executed"))
            }
        };

        let node3 = node_impl! {
            name: "Node3",
            exec: |_: &Value| {
                Ok(json!("node3_executed"))
            }
        };

        // Create linear flow using the macro
        let flow = flow! {
            name: "LinearTestFlow",
            nodes: [node1, node2, node3]
        };

        // Execute the flow
        let mut shared = test_shared();
        let result = flow.orchestrate(&mut shared, None);

        // Test that it completed successfully
        assert!(result.is_ok());
    }

    #[test]
    fn test_flow_macro_branching() {
        // Create a decision node
        let decision_node = decision_node! {
            name: "DecisionNode",
            condition: |_, shared| {
                if let Some(score) = shared.get("score") {
                    if score.as_f64().unwrap_or(0.0) > 0.7 {
                        "high".to_string()
                    } else {
                        "low".to_string()
                    }
                } else {
                    "unknown".to_string()
                }
            }
        };

        // Create branch nodes
        let high_node = node_impl! {
            name: "HighNode",
            exec: |_: &Value| {
                Ok(json!("high_executed"))
            },
            post: |shared: &mut Shared, _: &Value, _: &Value| {
                shared.insert("branch_taken".to_string(), json!("high"));
                Ok(json!("default"))
            }
        };

        let low_node = node_impl! {
            name: "LowNode",
            exec: |_: &Value| {
                Ok(json!("low_executed"))
            },
            post: |shared: &mut Shared, _: &Value, _: &Value| {
                shared.insert("branch_taken".to_string(), json!("low"));
                Ok(json!("default"))
            }
        };

        let unknown_node = node_impl! {
            name: "UnknownNode",
            exec: |_: &Value| {
                Ok(json!("unknown_executed"))
            },
            post: |shared: &mut Shared, _: &Value, _: &Value| {
                shared.insert("branch_taken".to_string(), json!("unknown"));
                Ok(json!("default"))
            }
        };

        let end_node = node_impl! {
            name: "EndNode",
            exec: |_: &Value| {
                Ok(json!("end_executed"))
            },
            post: |shared: &mut Shared, _: &Value, _: &Value| {
                shared.insert("flow_completed".to_string(), json!(true));
                Ok(json!("terminate"))
            }
        };

        // Create branching flow using the macro
        let flow = flow! {
            name: "BranchingTestFlow",
            start: decision_node.clone(),
            connections: [
                (decision_node.clone(), "high", high_node.clone()),
                (decision_node.clone(), "low", low_node.clone()),
                (decision_node.clone(), "unknown", unknown_node.clone()),
                (high_node.clone(), "default", end_node.clone()),
                (low_node.clone(), "default", end_node.clone()),
                (unknown_node.clone(), "default", end_node.clone())
            ]
        };

        // Execute the flow with score > 0.7
        let mut shared = test_shared(); // Contains a score of 0.75
        let result = flow.orchestrate(&mut shared, None);

        // Test that it completed successfully and took the high branch
        assert!(result.is_ok());
        assert_eq!(shared.get("branch_taken").unwrap().as_str(), Some("high"));
        assert_eq!(shared.get("flow_completed").unwrap().as_bool(), Some(true));

        // Change the score and test again
        let mut shared = test_shared();
        shared.insert("score".to_string(), json!(0.5)); // Lower score
        let result = flow.orchestrate(&mut shared, None);

        // Test that it took the low branch
        assert!(result.is_ok());
        assert_eq!(shared.get("branch_taken").unwrap().as_str(), Some("low"));
        assert_eq!(shared.get("flow_completed").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_decision_node_macro() {
        // Create a decision node
        let decision = decision_node! {
            name: "TestDecision",
            condition: |params, shared| {
                // Check shared state
                if let Some(score) = shared.get("score") {
                    if score.as_f64().unwrap_or(0.0) > 0.7 {
                        "high".to_string()
                    } else if score.as_f64().unwrap_or(0.0) > 0.3 {
                        "medium".to_string()
                    } else {
                        "low".to_string()
                    }
                } else {
                    // Check params
                    if let Some(fallback) = params.get("fallback") {
                        fallback.as_str().unwrap_or("unknown").to_string()
                    } else {
                        "unknown".to_string()
                    }
                }
            }
        };

        // Test with shared state containing score
        let mut shared1 = test_shared(); // Contains score = 0.75
        let result1 = decision.lock().unwrap().run(&mut shared1).unwrap();
        assert_eq!(result1, Action::from_str("high"));

        // Test with lower score
        let mut shared2 = test_shared();
        shared2.insert("score".to_string(), json!(0.5));
        let result2 = decision.lock().unwrap().run(&mut shared2).unwrap();
        assert_eq!(result2, Action::from_str("medium"));

        // Test with very low score
        let mut shared3 = test_shared();
        shared3.insert("score".to_string(), json!(0.2));
        let result3 = decision.lock().unwrap().run(&mut shared3).unwrap();
        assert_eq!(result3, Action::from_str("low"));

        // Test with missing score but fallback param
        let mut shared4 = HashMap::new();
        let mut params = HashMap::new();
        params.insert("fallback".to_string(), json!("custom_path"));

        let mut node = decision.lock().unwrap();
        node.set_params(params);
        let result4 = node.run(&mut shared4).unwrap();
        assert_eq!(result4, Action::from_str("custom_path"));
    }

    #[test]
    fn test_processing_chain_macro() {
        // Create a processing chain with multiple steps
        let processing_chain = processing_chain! {
            name: "TestProcessingChain",
            steps: [
                // Step 1: Validate input
                |data: &Value| {
                    if data.is_object() {
                        Ok(data.clone())
                    } else {
                        Err(crate::errors::FlowError::NodeExecution("Data must be an object".to_string()))
                    }
                },
                // Step 2: Extract value
                |data: &Value| {
                    if let Some(value) = data.get("value") {
                        Ok(value.clone())
                    } else {
                        Err(crate::errors::FlowError::NodeExecution("Missing 'value' field".to_string()))
                    }
                },
                // Step 3: Double the value if numeric - Explicitly return an integer
                |data: &Value| {
                    if let Some(num) = data.as_f64() {
                        Ok(json!((num * 2.0) as i64))
                    } else if let Some(num) = data.as_i64() {
                        Ok(json!(num * 2))
                    } else {
                        Err(crate::errors::FlowError::NodeExecution("Value is not numeric".to_string()))
                    }
                }
            ],
            max_retries: 1
        };

        // Test with valid input
        let mut shared = test_shared(); // Contains "input": {"value": 10}
        let mut node = processing_chain.lock().unwrap();

        // Run with valid input
        let prep_result = node.prep(&mut shared).unwrap();
        let exec_result = node.exec(&prep_result).unwrap();

        // Check the result using a value comparison that doesn't rely on exact type
        let expected_value = json!(20);
        assert!(
            (exec_result.as_i64() == Some(20)) || (exec_result.as_f64() == Some(20.0)),
            "Expected 20, got {:?}",
            exec_result
        );

        // Test with invalid input (missing value field)
        let mut shared2 = HashMap::new();
        shared2.insert("input".to_string(), json!({"wrong_field": 10}));

        let prep_result2 = node.prep(&mut shared2).unwrap();
        let exec_result2 = node.exec(&prep_result2);

        // Should fail at step 2
        assert!(exec_result2.is_err());
        assert!(exec_result2
            .unwrap_err()
            .to_string()
            .contains("Missing 'value' field"));

        // Test with non-numeric value
        let mut shared3 = HashMap::new();
        shared3.insert("input".to_string(), json!({"value": "not a number"}));

        let prep_result3 = node.prep(&mut shared3).unwrap();
        let exec_result3 = node.exec(&prep_result3);

        // Should fail at step 3
        assert!(exec_result3.is_err());
        assert!(exec_result3
            .unwrap_err()
            .to_string()
            .contains("Value is not numeric"));
    }
}
