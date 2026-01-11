//! # Value Conversion Helpers
//!
//! This module provides utility functions for converting between different
//! `VariableValue` types and performing common operations.
//!
//! ## Key Functions
//! - [`to_bool`]: Convert any VariableValue to boolean
//! - [`to_float`]: Convert any VariableValue to f64
//! - [`to_string`]: Convert any VariableValue to String
//! - [`compare_values`]: Compare two VariableValues
//! - [`compute_math`]: Perform math operations preserving type
//! - [`string_to_key`]: Convert string key name to enigo Key
//!
//! ## Usage
//! These functions are used throughout the executor to handle
//! dynamic typing in blueprint nodes.

use crate::graph::VariableValue;
use enigo::Key;

/// Convert a VariableValue to boolean.
///
/// # Conversion Rules
/// - Boolean: direct value
/// - Integer: true if > 0
/// - Float: true if > 0.0
/// - String: true if "true" or "1" (case-insensitive)
/// - Other: false
pub fn to_bool(val: &VariableValue) -> bool {
    match val {
        VariableValue::Boolean(b) => *b,
        VariableValue::Integer(i) => *i > 0,
        VariableValue::Float(f) => *f > 0.0,
        VariableValue::String(s) => s.to_lowercase() == "true" || s == "1",
        _ => false,
    }
}

/// Convert a VariableValue to f64.
///
/// # Conversion Rules
/// - Float: direct value
/// - Integer: cast to f64
/// - String: parse, fallback to 0.0
/// - Boolean: 1.0 if true, 0.0 if false
/// - Other: 0.0
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

/// Convert a VariableValue to String.
///
/// # Conversion Rules
/// - String: direct value
/// - Integer/Float/Boolean: to_string()
/// - Vector3: "(x, y, z)" format
/// - Array: "[item1, item2, ...]" format
/// - None: "None"
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

/// Compare two VariableValues.
///
/// Returns Ordering based on the values. Mixed numeric types
/// are compared as f64.
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

/// Perform math operation on two VariableValues.
///
/// Preserves integer type when both operands are integers,
/// otherwise uses float arithmetic.
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

/// Convert a string key name to an enigo Key variant.
///
/// Supports:
/// - Modifier keys: shift, control/ctrl, alt/option, meta/command/cmd/win
/// - Function keys: f1-f12
/// - Navigation keys: arrows, home, end, pageup, pagedown
/// - Special keys: enter, escape, tab, backspace, delete, space, capslock
/// - Single characters: converted to Unicode key
pub fn string_to_key(key_str: &str) -> Option<Key> {
    match key_str.to_lowercase().as_str() {
        // Modifier keys
        "shift" | "lshift" => Some(Key::Shift),
        "control" | "ctrl" | "lcontrol" => Some(Key::Control),
        "alt" | "option" | "lalt" => Some(Key::Alt),
        "meta" | "command" | "cmd" | "win" | "super" => Some(Key::Meta),

        // Function keys
        "f1" => Some(Key::F1),
        "f2" => Some(Key::F2),
        "f3" => Some(Key::F3),
        "f4" => Some(Key::F4),
        "f5" => Some(Key::F5),
        "f6" => Some(Key::F6),
        "f7" => Some(Key::F7),
        "f8" => Some(Key::F8),
        "f9" => Some(Key::F9),
        "f10" => Some(Key::F10),
        "f11" => Some(Key::F11),
        "f12" => Some(Key::F12),

        // Navigation keys
        "up" | "uparrow" => Some(Key::UpArrow),
        "down" | "downarrow" => Some(Key::DownArrow),
        "left" | "leftarrow" => Some(Key::LeftArrow),
        "right" | "rightarrow" => Some(Key::RightArrow),
        "home" => Some(Key::Home),
        "end" => Some(Key::End),
        "pageup" | "pgup" => Some(Key::PageUp),
        "pagedown" | "pgdn" => Some(Key::PageDown),

        // Special keys
        "return" | "enter" => Some(Key::Return),
        "escape" | "esc" => Some(Key::Escape),
        "tab" => Some(Key::Tab),
        "backspace" | "back" => Some(Key::Backspace),
        "delete" | "del" => Some(Key::Delete),
        "space" | " " => Some(Key::Space),
        "capslock" | "caps" => Some(Key::CapsLock),

        // If single character, return as Unicode key
        _ if key_str.len() == 1 => key_str.chars().next().map(Key::Unicode),

        // Unknown key
        _ => None,
    }
}
