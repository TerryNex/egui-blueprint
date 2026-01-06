use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DataType {
    ExecutionFlow,
    Boolean,
    Integer,
    Float,
    String,
    Vector3,
    Custom(String),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    BlueprintFunction { name: String },
    Branch,
    ForLoop,
    GetVariable { name: String },
    SetVariable { name: String },
    Add,
    Subtract,
    Multiply,
    Divide,
    Equals,
    GreaterThan,
    LessThan,
    InputParam, 
    OutputParam,
    // Entry point for the graph
    Entry,
}

impl Default for NodeType {
    fn default() -> Self {
        NodeType::Entry
    }
}
