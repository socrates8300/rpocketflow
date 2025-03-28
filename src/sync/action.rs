use serde_json::Value;
use std::fmt;
use std::hash::{Hash, Hasher};

/// Enum representing possible actions a node can take
#[derive(Debug, Clone)]
pub enum Action {
    /// Default action for linear flows
    Default,
    /// Named action for branching flows
    Named(String),
    /// Terminate the flow
    Terminate,
}

impl Action {
    /// Creates a new Action from a string
    pub fn from_str(s: &str) -> Self {
        match s {
            "default" => Action::Default,
            "terminate" => Action::Terminate,
            _ => Action::Named(s.to_string()),
        }
    }
}

impl From<&Value> for Action {
    fn from(value: &Value) -> Self {
        if value.is_null() {
            return Action::Default;
        }

        if let Some(s) = value.as_str() {
            Action::from_str(s)
        } else {
            Action::Default
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Default => write!(f, "default"),
            Action::Named(name) => write!(f, "{}", name),
            Action::Terminate => write!(f, "terminate"),
        }
    }
}

impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Action::Default, Action::Default) => true,
            (Action::Terminate, Action::Terminate) => true,
            (Action::Named(a), Action::Named(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Action {}

impl Hash for Action {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Action::Default => {
                0.hash(state);
                "default".hash(state);
            }
            Action::Terminate => {
                1.hash(state);
                "terminate".hash(state);
            }
            Action::Named(name) => {
                2.hash(state);
                name.hash(state);
            }
        }
    }
}
