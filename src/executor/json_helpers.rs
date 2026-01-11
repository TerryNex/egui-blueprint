//! # JSON Conversion Helpers
//!
//! This module provides functions for converting between JSON values
//! and the blueprint's `VariableValue` type.
//!
//! ## Key Functions
//! - [`json_to_variable_value`]: Convert serde_json::Value to VariableValue
//! - [`variable_value_to_json`]: Convert VariableValue to serde_json::Value
//!
//! ## Dependencies
//! - `serde_json`: JSON parsing and serialization
//! - `crate::graph::VariableValue`: Blueprint value type

use crate::graph::VariableValue;

/// Convert serde_json::Value to VariableValue.
///
/// # Mapping
/// - Null → None
/// - Bool → Boolean
/// - Number (i64) → Integer
/// - Number (f64) → Float
/// - String → String
/// - Array → Array (recursive conversion)
/// - Object → String (JSON serialized)
pub fn json_to_variable_value(value: &serde_json::Value) -> VariableValue {
    match value {
        serde_json::Value::Null => VariableValue::None,
        serde_json::Value::Bool(b) => VariableValue::Boolean(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                VariableValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                VariableValue::Float(f)
            } else {
                VariableValue::None
            }
        }
        serde_json::Value::String(s) => VariableValue::String(s.clone()),
        serde_json::Value::Array(arr) => {
            let values: Vec<VariableValue> = arr
                .iter()
                .map(|v| json_to_variable_value(v))
                .collect();
            VariableValue::Array(values)
        }
        serde_json::Value::Object(obj) => {
            // Convert object to JSON string for now (can be extended later)
            VariableValue::String(serde_json::to_string(obj).unwrap_or_default())
        }
    }
}

/// Convert VariableValue to serde_json::Value.
///
/// # Mapping
/// - None → Null
/// - Boolean → Bool
/// - Integer → Number
/// - Float → Number
/// - String → String
/// - Vector3 → Array [x, y, z]
/// - Array → Array (recursive conversion)
pub fn variable_value_to_json(value: &VariableValue) -> serde_json::Value {
    match value {
        VariableValue::None => serde_json::Value::Null,
        VariableValue::Boolean(b) => serde_json::Value::Bool(*b),
        VariableValue::Integer(i) => serde_json::json!(*i),
        VariableValue::Float(f) => serde_json::json!(*f),
        VariableValue::String(s) => serde_json::Value::String(s.clone()),
        VariableValue::Vector3(x, y, z) => serde_json::json!([x, y, z]),
        VariableValue::Array(arr) => {
            let values: Vec<serde_json::Value> = arr
                .iter()
                .map(|v| variable_value_to_json(v))
                .collect();
            serde_json::Value::Array(values)
        }
    }
}
