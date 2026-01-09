//! Execution context for variable storage during blueprint execution.

use crate::graph::VariableValue;
use std::collections::HashMap;

/// Stores the execution state including all variables.
pub struct ExecutionContext {
    pub variables: HashMap<String, VariableValue>,
}

impl ExecutionContext {
    /// Creates a new empty execution context.
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}
