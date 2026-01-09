//! Node evaluation logic for computing output values.
//!
//! This module handles the evaluation of individual nodes to produce their output values.
//! It does not handle flow execution - see `flow_control.rs` for that.

use super::context::ExecutionContext;
use super::image_recognition::compare_images;
use super::type_conversions::{compare_values, compute_math, to_bool, to_float, to_string};
use super::type_conversions::{json_to_variable_value, variable_value_to_json};
use crate::graph::{BlueprintGraph, Node, VariableValue};
use crate::node_types::NodeType;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Evaluate an input port by finding the connected node and evaluating it.
pub fn evaluate_input(
    graph: &BlueprintGraph,
    node_id: Uuid,
    port_name: &str,
    context: &Arc<Mutex<ExecutionContext>>,
) -> anyhow::Result<VariableValue> {
    for conn in &graph.connections {
        if conn.to_node == node_id && conn.to_port == port_name {
            let from_node = graph
                .nodes
                .get(&conn.from_node)
                .ok_or_else(|| anyhow::anyhow!("Source node not found"))?;
            return evaluate_node(graph, from_node, &conn.from_port, context);
        }
    }

    if let Some(node) = graph.nodes.get(&node_id) {
        if let Some(port) = node.inputs.iter().find(|p| p.name == port_name) {
            return Ok(port.default_value.clone());
        }
    }

    Ok(VariableValue::None)
}

/// Evaluate a node's output port to get its value.
pub fn evaluate_node(
    graph: &BlueprintGraph,
    node: &Node,
    _output_port: &str,
    context: &Arc<Mutex<ExecutionContext>>,
) -> anyhow::Result<VariableValue> {
    match &node.node_type {
        // === Math Operations ===
        NodeType::Add => {
            let a = evaluate_input(graph, node.id, "A", context)?;
            let b = evaluate_input(graph, node.id, "B", context)?;
            compute_math(a, b, |a, b| a + b, |a, b| a + b)
        }
        NodeType::Subtract => {
            let a = evaluate_input(graph, node.id, "A", context)?;
            let b = evaluate_input(graph, node.id, "B", context)?;
            compute_math(a, b, |a, b| a - b, |a, b| a - b)
        }
        NodeType::Multiply => {
            let a = evaluate_input(graph, node.id, "A", context)?;
            let b = evaluate_input(graph, node.id, "B", context)?;
            compute_math(a, b, |a, b| a * b, |a, b| a * b)
        }
        NodeType::Divide => {
            let a = evaluate_input(graph, node.id, "A", context)?;
            let mut b = evaluate_input(graph, node.id, "B", context)?;
            // Divide by zero protection
            let is_zero = match &b {
                VariableValue::Float(f) => *f == 0.0,
                VariableValue::Integer(i) => *i == 0,
                _ => false,
            };
            if is_zero {
                b = match &b {
                    VariableValue::Float(_) => VariableValue::Float(1.0),
                    VariableValue::Integer(_) => VariableValue::Integer(1),
                    _ => VariableValue::Integer(1),
                };
            }
            compute_math(a, b, |a, b| a / b, |a, b| a / b)
        }
        NodeType::Modulo => {
            let a = evaluate_input(graph, node.id, "A", context)?;
            let b = evaluate_input(graph, node.id, "B", context)?;
            match (a, b) {
                (VariableValue::Integer(av), VariableValue::Integer(bv)) => {
                    if bv == 0 {
                        Ok(VariableValue::Integer(0))
                    } else {
                        Ok(VariableValue::Integer(av % bv))
                    }
                }
                _ => Ok(VariableValue::Integer(0)),
            }
        }
        NodeType::Power => {
            let base = evaluate_input(graph, node.id, "Base", context)?;
            let exp = evaluate_input(graph, node.id, "Exponent", context)?;
            let base_f = to_float(&base);
            let exp_f = to_float(&exp);
            Ok(VariableValue::Float(base_f.powf(exp_f)))
        }
        NodeType::Abs => {
            let input = evaluate_input(graph, node.id, "In", context)?;
            match input {
                VariableValue::Float(f) => Ok(VariableValue::Float(f.abs())),
                VariableValue::Integer(i) => Ok(VariableValue::Integer(i.abs())),
                _ => Ok(VariableValue::Float(0.0)),
            }
        }
        NodeType::Min => {
            let a = evaluate_input(graph, node.id, "A", context)?;
            let b = evaluate_input(graph, node.id, "B", context)?;
            let af = to_float(&a);
            let bf = to_float(&b);
            Ok(VariableValue::Float(af.min(bf)))
        }
        NodeType::Max => {
            let a = evaluate_input(graph, node.id, "A", context)?;
            let b = evaluate_input(graph, node.id, "B", context)?;
            let af = to_float(&a);
            let bf = to_float(&b);
            Ok(VariableValue::Float(af.max(bf)))
        }
        NodeType::Clamp => {
            let value = evaluate_input(graph, node.id, "Value", context)?;
            let min = evaluate_input(graph, node.id, "Min", context)?;
            let max = evaluate_input(graph, node.id, "Max", context)?;
            let vf = to_float(&value);
            let minf = to_float(&min);
            let maxf = to_float(&max);
            Ok(VariableValue::Float(vf.clamp(minf, maxf)))
        }
        NodeType::Random => {
            let min = evaluate_input(graph, node.id, "Min", context)?;
            let max = evaluate_input(graph, node.id, "Max", context)?;
            let minf = to_float(&min);
            let maxf = to_float(&max);
            // Simple random using time-based seed
            let random_val = {
                use std::time::{SystemTime, UNIX_EPOCH};
                let seed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as u64;
                let pseudo = ((seed.wrapping_mul(1103515245).wrapping_add(12345)) % (1 << 31))
                    as f64
                    / (1u64 << 31) as f64;
                minf + pseudo * (maxf - minf)
            };
            Ok(VariableValue::Float(random_val))
        }

        // === Variable Access ===
        NodeType::GetVariable { name } => {
            let ctx = context.lock().unwrap();
            ctx.variables
                .get(name)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Variable not found: {}", name))
        }

        // === Type Conversions ===
        NodeType::ToInteger => {
            let input = evaluate_input(graph, node.id, "In", context)?;
            match input {
                VariableValue::Integer(i) => Ok(VariableValue::Integer(i)),
                VariableValue::Float(f) => Ok(VariableValue::Integer(f as i64)),
                VariableValue::String(s) => Ok(VariableValue::Integer(s.parse().unwrap_or(0))),
                VariableValue::Boolean(b) => Ok(VariableValue::Integer(if b { 1 } else { 0 })),
                _ => Ok(VariableValue::Integer(0)),
            }
        }
        NodeType::ToFloat => {
            let input = evaluate_input(graph, node.id, "In", context)?;
            match input {
                VariableValue::Float(f) => Ok(VariableValue::Float(f)),
                VariableValue::Integer(i) => Ok(VariableValue::Float(i as f64)),
                VariableValue::String(s) => Ok(VariableValue::Float(s.parse().unwrap_or(0.0))),
                VariableValue::Boolean(b) => Ok(VariableValue::Float(if b { 1.0 } else { 0.0 })),
                _ => Ok(VariableValue::Float(0.0)),
            }
        }
        NodeType::ToString => {
            let input = evaluate_input(graph, node.id, "In", context)?;
            let s = match input {
                VariableValue::String(s) => s,
                VariableValue::Integer(i) => i.to_string(),
                VariableValue::Float(f) => f.to_string(),
                VariableValue::Boolean(b) => b.to_string(),
                VariableValue::Vector3(x, y, z) => format!("({}, {}, {})", x, y, z),
                VariableValue::Array(arr) => to_string(&VariableValue::Array(arr)),
                VariableValue::None => "None".to_string(),
            };
            Ok(VariableValue::String(s))
        }

        // === Comparison Operations ===
        NodeType::Equals => {
            let a = evaluate_input(graph, node.id, "A", context)?;
            let b = evaluate_input(graph, node.id, "B", context)?;
            Ok(VariableValue::Boolean(
                compare_values(&a, &b) == std::cmp::Ordering::Equal,
            ))
        }
        NodeType::NotEquals => {
            let a = evaluate_input(graph, node.id, "A", context)?;
            let b = evaluate_input(graph, node.id, "B", context)?;
            Ok(VariableValue::Boolean(
                compare_values(&a, &b) != std::cmp::Ordering::Equal,
            ))
        }
        NodeType::GreaterThan => {
            let a = evaluate_input(graph, node.id, "A", context)?;
            let b = evaluate_input(graph, node.id, "B", context)?;
            Ok(VariableValue::Boolean(
                compare_values(&a, &b) == std::cmp::Ordering::Greater,
            ))
        }
        NodeType::GreaterThanOrEqual => {
            let a = evaluate_input(graph, node.id, "A", context)?;
            let b = evaluate_input(graph, node.id, "B", context)?;
            let cmp = compare_values(&a, &b);
            Ok(VariableValue::Boolean(
                cmp == std::cmp::Ordering::Greater || cmp == std::cmp::Ordering::Equal,
            ))
        }
        NodeType::LessThan => {
            let a = evaluate_input(graph, node.id, "A", context)?;
            let b = evaluate_input(graph, node.id, "B", context)?;
            Ok(VariableValue::Boolean(
                compare_values(&a, &b) == std::cmp::Ordering::Less,
            ))
        }
        NodeType::LessThanOrEqual => {
            let a = evaluate_input(graph, node.id, "A", context)?;
            let b = evaluate_input(graph, node.id, "B", context)?;
            let cmp = compare_values(&a, &b);
            Ok(VariableValue::Boolean(
                cmp == std::cmp::Ordering::Less || cmp == std::cmp::Ordering::Equal,
            ))
        }

        // === Logic Operations ===
        NodeType::And => {
            let a = evaluate_input(graph, node.id, "A", context)?;
            let b = evaluate_input(graph, node.id, "B", context)?;
            Ok(VariableValue::Boolean(to_bool(&a) && to_bool(&b)))
        }
        NodeType::Or => {
            let a = evaluate_input(graph, node.id, "A", context)?;
            let b = evaluate_input(graph, node.id, "B", context)?;
            Ok(VariableValue::Boolean(to_bool(&a) || to_bool(&b)))
        }
        NodeType::Not => {
            let input = evaluate_input(graph, node.id, "In", context)?;
            Ok(VariableValue::Boolean(!to_bool(&input)))
        }
        NodeType::Xor => {
            let a = evaluate_input(graph, node.id, "A", context)?;
            let b = evaluate_input(graph, node.id, "B", context)?;
            Ok(VariableValue::Boolean(to_bool(&a) ^ to_bool(&b)))
        }

        // === Loop Index ===
        NodeType::ForLoop => {
            let ctx = context.lock().unwrap();
            Ok(ctx
                .variables
                .get("__loop_index")
                .cloned()
                .unwrap_or(VariableValue::Integer(0)))
        }

        // === String Operations ===
        NodeType::Concat => {
            let a = evaluate_input(graph, node.id, "A", context)?;
            let b = evaluate_input(graph, node.id, "B", context)?;
            let sa = to_string(&a);
            let sb = to_string(&b);
            Ok(VariableValue::String(format!("{}{}", sa, sb)))
        }
        NodeType::Split => {
            let input = evaluate_input(graph, node.id, "String", context)?;
            let delim = evaluate_input(graph, node.id, "Delimiter", context)?;
            let index = evaluate_input(graph, node.id, "Index", context)?;
            let s = to_string(&input);
            let d = to_string(&delim);
            let idx = match index {
                VariableValue::Integer(i) => i as usize,
                _ => 0,
            };
            let parts: Vec<&str> = s.split(&d).collect();
            let result = parts.get(idx).unwrap_or(&"").to_string();
            Ok(VariableValue::String(result))
        }
        NodeType::Length => {
            let input = evaluate_input(graph, node.id, "String", context)?;
            let s = to_string(&input);
            Ok(VariableValue::Integer(s.len() as i64))
        }
        NodeType::Contains => {
            let input = evaluate_input(graph, node.id, "String", context)?;
            let sub = evaluate_input(graph, node.id, "Substring", context)?;
            let s = to_string(&input);
            let sub_s = to_string(&sub);
            Ok(VariableValue::Boolean(s.contains(&sub_s)))
        }
        NodeType::Replace => {
            let input = evaluate_input(graph, node.id, "String", context)?;
            let from = evaluate_input(graph, node.id, "From", context)?;
            let to = evaluate_input(graph, node.id, "To", context)?;
            let s = to_string(&input);
            let from_s = to_string(&from);
            let to_s = to_string(&to);
            Ok(VariableValue::String(s.replace(&from_s, &to_s)))
        }
        NodeType::Format => {
            let template = evaluate_input(graph, node.id, "Template", context)?;
            let arg0 = evaluate_input(graph, node.id, "Arg0", context)?;
            let t = to_string(&template);
            let a = to_string(&arg0);
            Ok(VariableValue::String(t.replacen("{}", &a, 1)))
        }
        NodeType::StringJoin => {
            let mut result = String::new();
            let mut idx = 0;
            loop {
                let port_name = format!("Input {}", idx);
                match evaluate_input(graph, node.id, &port_name, context) {
                    Ok(val) if !matches!(val, VariableValue::None) => {
                        result.push_str(&to_string(&val));
                        idx += 1;
                    }
                    _ => break,
                }
            }
            Ok(VariableValue::String(result))
        }
        NodeType::StringBetween => {
            let source = evaluate_input(graph, node.id, "Source", context)?;
            let before = evaluate_input(graph, node.id, "Before", context)?;
            let after = evaluate_input(graph, node.id, "After", context)?;

            let source_s = to_string(&source);
            let before_s = to_string(&before);
            let after_s = to_string(&after);

            let result = if before_s.is_empty() && after_s.is_empty() {
                source_s.clone()
            } else if before_s.is_empty() {
                source_s.split(&after_s).next().unwrap_or("").to_string()
            } else if after_s.is_empty() {
                match source_s.split_once(&before_s) {
                    Some((_, rest)) => rest.to_string(),
                    None => String::new(),
                }
            } else {
                match source_s.split_once(&before_s) {
                    Some((_, rest)) => rest.split(&after_s).next().unwrap_or("").to_string(),
                    None => String::new(),
                }
            };

            Ok(VariableValue::String(result))
        }

        // === File Operations ===
        NodeType::FileRead => {
            let path = evaluate_input(graph, node.id, "Path", context)?;
            let path_s = to_string(&path);
            match std::fs::read_to_string(&path_s) {
                Ok(content) => Ok(VariableValue::String(content)),
                Err(_) => Ok(VariableValue::String("".into())),
            }
        }

        // === Array Operations ===
        NodeType::ArrayCreate => Ok(VariableValue::Array(Vec::new())),
        NodeType::ArrayGet => {
            let array = evaluate_input(graph, node.id, "Array", context)?;
            let index = evaluate_input(graph, node.id, "Index", context)?;

            let idx = match index {
                VariableValue::Integer(i) => i as usize,
                VariableValue::Float(f) => f as usize,
                _ => 0,
            };

            match array {
                VariableValue::Array(arr) => {
                    Ok(arr.get(idx).cloned().unwrap_or(VariableValue::None))
                }
                VariableValue::String(s) => {
                    let chars: Vec<char> = s.chars().collect();
                    Ok(chars
                        .get(idx)
                        .map(|c| VariableValue::String(c.to_string()))
                        .unwrap_or(VariableValue::None))
                }
                _ => Ok(VariableValue::None),
            }
        }
        NodeType::ArrayLength => {
            let array = evaluate_input(graph, node.id, "Array", context)?;
            match array {
                VariableValue::Array(arr) => Ok(VariableValue::Integer(arr.len() as i64)),
                VariableValue::String(s) => Ok(VariableValue::Integer(s.len() as i64)),
                _ => Ok(VariableValue::Integer(0)),
            }
        }

        // === JSON Operations ===
        NodeType::JSONParse => {
            let input = evaluate_input(graph, node.id, "JSON", context)?;
            let json_str = to_string(&input);

            match serde_json::from_str::<serde_json::Value>(&json_str) {
                Ok(value) => Ok(json_to_variable_value(&value)),
                Err(_) => Ok(VariableValue::None),
            }
        }
        NodeType::JSONStringify => {
            let input = evaluate_input(graph, node.id, "Value", context)?;
            let json_value = variable_value_to_json(&input);
            Ok(VariableValue::String(json_value.to_string()))
        }

        // === Image Similarity ===
        NodeType::ImageSimilarity => {
            let path1 = evaluate_input(graph, node.id, "ImagePath1", context)
                .map(|v| to_string(&v))
                .unwrap_or_default();
            let path2 = evaluate_input(graph, node.id, "ImagePath2", context)
                .map(|v| to_string(&v))
                .unwrap_or_default();
            let tolerance = evaluate_input(graph, node.id, "Tolerance", context)
                .map(|v| to_float(&v) as i32)
                .unwrap_or(10);

            let similarity = match (image::open(&path1), image::open(&path2)) {
                (Ok(img1), Ok(img2)) => {
                    let img1 = img1.to_rgba8();
                    let img2 = img2.to_rgba8();
                    compare_images(&img1, &img2, tolerance)
                }
                _ => 0.0,
            };

            let port = _output_port;
            if port == "Similarity" {
                Ok(VariableValue::Float(similarity))
            } else if port == "Match" {
                Ok(VariableValue::Boolean(similarity >= 0.95))
            } else {
                Ok(VariableValue::Float(similarity))
            }
        }

        // === Window Position (Pure data retrieval) ===
        NodeType::GetWindowPosition => {
            let output_port = _output_port;
            let cache_key = format!("__winpos_{}", node.id);

            let (x, y, w, h, found) = {
                let ctx = context.lock().unwrap();
                if let Some(VariableValue::String(cached)) = ctx.variables.get(&cache_key) {
                    let parts: Vec<&str> = cached.split(',').collect();
                    if parts.len() == 5 {
                        (
                            parts[0].parse::<i64>().unwrap_or(0),
                            parts[1].parse::<i64>().unwrap_or(0),
                            parts[2].parse::<i64>().unwrap_or(1920),
                            parts[3].parse::<i64>().unwrap_or(1080),
                            parts[4] == "true",
                        )
                    } else {
                        (0, 0, 1920, 1080, true)
                    }
                } else {
                    drop(ctx);

                    let title = evaluate_input(graph, node.id, "Title", context)
                        .map(|v| to_string(&v))
                        .unwrap_or_default();

                    #[cfg(target_os = "macos")]
                    let (x, y, w, h, found) = get_window_position_macos(&title);

                    #[cfg(target_os = "linux")]
                    let (x, y, w, h, found) = get_window_position_linux(&title);

                    #[cfg(target_os = "windows")]
                    let (x, y, w, h, found) = (0i64, 0i64, 1920i64, 1080i64, true);

                    #[cfg(not(any(
                        target_os = "macos",
                        target_os = "linux",
                        target_os = "windows"
                    )))]
                    let (x, y, w, h, found) = (0i64, 0i64, 1920i64, 1080i64, false);

                    let cache_value = format!("{},{},{},{},{}", x, y, w, h, found);
                    context
                        .lock()
                        .unwrap()
                        .variables
                        .insert(cache_key.clone(), VariableValue::String(cache_value));

                    (x, y, w, h, found)
                }
            };

            match output_port {
                "X" => Ok(VariableValue::Integer(x)),
                "Y" => Ok(VariableValue::Integer(y)),
                "Width" => Ok(VariableValue::Integer(w)),
                "Height" => Ok(VariableValue::Integer(h)),
                "Found" => Ok(VariableValue::Boolean(found)),
                _ => Ok(VariableValue::None),
            }
        }

        // === Stored Outputs (from flow nodes that cache results) ===
        NodeType::RunCommand
        | NodeType::LaunchApp
        | NodeType::CloseApp
        | NodeType::FocusWindow
        | NodeType::SetWindowPosition
        | NodeType::FindColor
        | NodeType::GetPixelColor
        | NodeType::WaitForColor
        | NodeType::FindImage
        | NodeType::WaitForImage
        | NodeType::ScreenCapture
        | NodeType::SaveScreenshot => {
            let ctx = context.lock().unwrap();
            let key = format!("__out_{}_{}", node.id, _output_port);
            Ok(ctx
                .variables
                .get(&key)
                .cloned()
                .unwrap_or(VariableValue::None))
        }

        _ => Ok(VariableValue::None),
    }
}

// Platform-specific window position helpers

#[cfg(target_os = "macos")]
fn get_window_position_macos(title: &str) -> (i64, i64, i64, i64, bool) {
    let escaped_title = title.replace("\"", "\\\"");
    let script = format!(
        r#"tell application "System Events"
            set resultStr to "0,0,1920,1080,false"
            repeat with proc in (every process whose visible is true)
                try
                    repeat with win in windows of proc
                        if name of win contains "{}" then
                            set winPos to position of win
                            set winSize to size of win
                            set resultStr to ((item 1 of winPos) as string) & "," & ((item 2 of winPos) as string) & "," & ((item 1 of winSize) as string) & "," & ((item 2 of winSize) as string) & ",true"
                            exit repeat
                        end if
                    end repeat
                    if resultStr ends with "true" then exit repeat
                end try
            end repeat
            return resultStr
        end tell"#,
        escaped_title
    );
    match std::process::Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let parts: Vec<&str> = stdout.trim().split(',').collect();
            if parts.len() == 5 {
                (
                    parts[0].parse::<i64>().unwrap_or(0),
                    parts[1].parse::<i64>().unwrap_or(0),
                    parts[2].parse::<i64>().unwrap_or(1920),
                    parts[3].parse::<i64>().unwrap_or(1080),
                    parts[4] == "true",
                )
            } else {
                (0, 0, 1920, 1080, false)
            }
        }
        Err(_) => (0, 0, 1920, 1080, false),
    }
}

#[cfg(target_os = "linux")]
fn get_window_position_linux(title: &str) -> (i64, i64, i64, i64, bool) {
    let id_result = std::process::Command::new("xdotool")
        .args(["search", "--name", title])
        .output();
    if let Ok(id_output) = id_result {
        let wid = String::from_utf8_lossy(&id_output.stdout)
            .lines()
            .next()
            .unwrap_or("")
            .to_string();
        if !wid.is_empty() {
            if let Ok(geom) = std::process::Command::new("xdotool")
                .args(["getwindowgeometry", "--shell", &wid])
                .output()
            {
                let geom_str = String::from_utf8_lossy(&geom.stdout);
                let mut x = 0i64;
                let mut y = 0i64;
                let mut w = 1920i64;
                let mut h = 1080i64;
                for line in geom_str.lines() {
                    if line.starts_with("X=") {
                        x = line[2..].parse().unwrap_or(0);
                    }
                    if line.starts_with("Y=") {
                        y = line[2..].parse().unwrap_or(0);
                    }
                    if line.starts_with("WIDTH=") {
                        w = line[6..].parse().unwrap_or(1920);
                    }
                    if line.starts_with("HEIGHT=") {
                        h = line[7..].parse().unwrap_or(1080);
                    }
                }
                return (x, y, w, h, true);
            }
        }
    }
    (0, 0, 1920, 1080, false)
}
