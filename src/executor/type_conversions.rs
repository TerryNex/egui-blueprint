//! Type conversion utilities for the executor.
//!
//! Provides functions for converting between VariableValue types,
//! comparing values, and computing math operations.

use crate::graph::VariableValue;

/// Convert a VariableValue to a boolean.
pub fn to_bool(val: &VariableValue) -> bool {
    match val {
        VariableValue::Boolean(b) => *b,
        VariableValue::Integer(i) => *i > 0,
        VariableValue::Float(f) => *f > 0.0,
        VariableValue::String(s) => s.to_lowercase() == "true" || s == "1",
        _ => false,
    }
}

/// Convert a VariableValue to a float.
pub fn to_float(val: &VariableValue) -> f64 {
    match val {
        VariableValue::Float(f) => *f,
        VariableValue::Integer(i) => *i as f64,
        VariableValue::String(s) => s.parse().unwrap_or(0.0),
        VariableValue::Boolean(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        _ => 0.0,
    }
}

/// Convert a VariableValue to a string.
pub fn to_string(val: &VariableValue) -> String {
    match val {
        VariableValue::String(s) => s.clone(),
        VariableValue::Integer(i) => i.to_string(),
        VariableValue::Float(f) => f.to_string(),
        VariableValue::Boolean(b) => b.to_string(),
        VariableValue::Vector3(x, y, z) => format!("({}, {}, {})", x, y, z),
        VariableValue::Array(arr) => {
            let items: Vec<String> = arr.iter().map(|v| to_string(v)).collect();
            format!("[{}]", items.join(", "))
        }
        VariableValue::None => "None".to_string(),
    }
}

/// Compare two VariableValues and return their ordering.
pub fn compare_values(a: &VariableValue, b: &VariableValue) -> std::cmp::Ordering {
    match (a, b) {
        (VariableValue::Float(av), VariableValue::Float(bv)) => {
            av.partial_cmp(bv).unwrap_or(std::cmp::Ordering::Equal)
        }
        (VariableValue::Integer(av), VariableValue::Integer(bv)) => av.cmp(bv),
        (VariableValue::Float(av), VariableValue::Integer(bv)) => av
            .partial_cmp(&(*bv as f64))
            .unwrap_or(std::cmp::Ordering::Equal),
        (VariableValue::Integer(av), VariableValue::Float(bv)) => (*av as f64)
            .partial_cmp(bv)
            .unwrap_or(std::cmp::Ordering::Equal),
        (VariableValue::String(av), VariableValue::String(bv)) => av.cmp(bv),
        (VariableValue::Boolean(av), VariableValue::Boolean(bv)) => av.cmp(bv),
        _ => std::cmp::Ordering::Equal,
    }
}

/// Compute a math operation on two VariableValues.
pub fn compute_math(
    a: VariableValue,
    b: VariableValue,
    op_f: fn(f64, f64) -> f64,
    op_i: fn(i64, i64) -> i64,
) -> anyhow::Result<VariableValue> {
    match (a, b) {
        (VariableValue::Float(av), VariableValue::Float(bv)) => {
            Ok(VariableValue::Float(op_f(av, bv)))
        }
        (VariableValue::Integer(av), VariableValue::Integer(bv)) => {
            Ok(VariableValue::Integer(op_i(av, bv)))
        }
        (VariableValue::Float(av), VariableValue::Integer(bv)) => {
            Ok(VariableValue::Float(op_f(av, bv as f64)))
        }
        (VariableValue::Integer(av), VariableValue::Float(bv)) => {
            Ok(VariableValue::Float(op_f(av as f64, bv)))
        }
        _ => Ok(VariableValue::None),
    }
}

// === JSON Conversion Helpers ===

/// Convert serde_json::Value to VariableValue.
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
            let values: Vec<VariableValue> =
                arr.iter().map(|v| json_to_variable_value(v)).collect();
            VariableValue::Array(values)
        }
        serde_json::Value::Object(obj) => {
            // Convert object to JSON string for now
            VariableValue::String(serde_json::to_string(obj).unwrap_or_default())
        }
    }
}

/// Convert VariableValue to serde_json::Value.
pub fn variable_value_to_json(value: &VariableValue) -> serde_json::Value {
    match value {
        VariableValue::None => serde_json::Value::Null,
        VariableValue::Boolean(b) => serde_json::Value::Bool(*b),
        VariableValue::Integer(i) => serde_json::json!(*i),
        VariableValue::Float(f) => serde_json::json!(*f),
        VariableValue::String(s) => serde_json::Value::String(s.clone()),
        VariableValue::Vector3(x, y, z) => serde_json::json!([x, y, z]),
        VariableValue::Array(arr) => {
            let values: Vec<serde_json::Value> =
                arr.iter().map(|v| variable_value_to_json(v)).collect();
            serde_json::Value::Array(values)
        }
    }
}
