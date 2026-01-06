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
    WhileLoop,
    GetVariable { name: String },
    SetVariable { name: String },
    // Math operations
    Add,
    Subtract,
    Multiply,
    Divide,
    // Comparison operations
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    // Logic operations
    And,
    Or,
    Not,
    // Other
    InputParam, 
    OutputParam,
    // Entry point for the graph
    Entry,
    // Type conversions
    ToInteger,
    ToFloat,
    ToString,
    // Timing
    Delay,
}

impl Default for NodeType {
    fn default() -> Self {
        NodeType::Entry
    }
}
