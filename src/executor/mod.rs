//! # Blueprint Executor
//!
//! This module handles the execution of blueprint graphs.
//!
//! ## Submodules
//! - [`helpers`]: Value conversion utilities (to_bool, to_float, to_string, etc.)
//! - [`json_helpers`]: JSON conversion functions
//! - [`image_matching`]: Template matching algorithms
//! - [`flow_control`]: Loop and branch execution
//! - [`node_eval`]: Node evaluation logic
//! - [`automation`]: Input automation helpers
//!
//! ## Main Entry Point
//! Use [`Interpreter::run_async`] to execute a blueprint graph.

// Submodules
pub mod automation;
pub mod context;
pub mod flow_control;
pub mod helpers;
pub mod image_matching;
pub mod image_recognition;
pub mod json_helpers;
pub mod node_eval;
pub mod type_conversions;

use crate::graph::{BlueprintGraph, Node, VariableValue};
use crate::node_types::NodeType;
use enigo::{Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use uuid::Uuid;
use xcap::Monitor;

pub struct ExecutionContext {
    pub variables: HashMap<String, VariableValue>,
}

impl ExecutionContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
}

pub struct Interpreter;

impl Interpreter {
    pub fn run_async(graph: &BlueprintGraph) -> Receiver<String> {
        let (tx, rx) = channel();

        // Initial log
        let tx_main = tx.clone();
        tx_main
            .send("Interpreter started (Async).".to_string())
            .unwrap_or_default();

        // Clone graph and create shared context
        let graph = Arc::new(graph.clone());
        let context = Arc::new(Mutex::new(ExecutionContext::new()));

        // Initialize variables
        {
            let mut ctx = context.lock().unwrap();
            for (name, var) in &graph.variables {
                ctx.variables
                    .insert(name.clone(), var.initial_value.clone());
            }
        }

        // Find all Event Tick nodes
        let mut start_nodes = Vec::new();
        for node in graph.nodes.values() {
            if let NodeType::BlueprintFunction { name } = &node.node_type {
                if name == "Event Tick" {
                    start_nodes.push(node.id);
                }
            }
        }

        if start_nodes.is_empty() {
            tx_main
                .send("No 'Event Tick' node found. Execution aborted.".to_string())
                .unwrap_or_default();
            // We return rx, connection closes, main thread detects it? No, rx stays open but sender dropped?
            // Actually tx_main is dropped here. If threads spawn, they hold tx clones.
            // If no threads spawn, all tx dropped, rx recv returns Err.
            return rx;
        }

        for start_id in start_nodes {
            let graph_clone = graph.clone();
            let context_clone = context.clone();
            let tx_clone = tx.clone();

            thread::spawn(move || {
                Self::execute_flow(graph_clone, start_id, context_clone, tx_clone);
            });
        }

        rx
    }

    pub fn execute_flow(
        graph: Arc<BlueprintGraph>,
        start_id: Uuid,
        context: Arc<Mutex<ExecutionContext>>,
        tx: Sender<String>,
    ) {
        let logger = |msg: String| {
            let _ = tx.send(msg);
        };

        let mut current_node_id = start_id;

        // Move to first Next
        if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
            current_node_id = next;
        } else {
            return;
        }

        let mut steps = 0;
        let max_steps = 5000;
        let mut enigo = Enigo::new(&Settings::default()).ok(); // Initial attempt
        if enigo.is_none() {
             logger("Warning: Failed to initialize Input Simulator (Enigo). Mouse/Keyboard actions will fail.".into());
        }

        while steps < max_steps {
            let node = match graph.nodes.get(&current_node_id) {
                Some(n) => n,
                None => break,
            };

            match &node.node_type {
                NodeType::SetVariable { name } => {
                    // Check input
                    if let Ok(val) =
                        Self::evaluate_input(&graph, current_node_id, "Value", &context)
                    {
                        let mut ctx = context.lock().unwrap();
                        ctx.variables.insert(name.clone(), val);
                    } else {
                        logger("Error evaluating SetVariable input.".into());
                        break;
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }
                NodeType::BlueprintFunction { name } => {
                    if name == "Print String" {
                        if let Ok(val) =
                            Self::evaluate_input(&graph, current_node_id, "String", &context)
                        {
                            // Auto-convert any type to string
                            let output = Self::to_string(&val);
                            logger(output);
                        }

                        if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                            current_node_id = next;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                NodeType::Branch => {
                    let condition =
                        Self::evaluate_input(&graph, current_node_id, "Condition", &context)
                            .unwrap_or(VariableValue::Boolean(false));

                    let bool_val = match condition {
                        VariableValue::Boolean(b) => b,
                        VariableValue::Integer(i) => i > 0,
                        VariableValue::Float(f) => f > 0.0,
                        _ => false,
                    };

                    let port = if bool_val { "True" } else { "False" };
                    if let Some(next) = Self::follow_flow(&graph, current_node_id, port) {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }
                NodeType::Delay => {
                    let duration_ms = match Self::evaluate_input(
                        &graph,
                        current_node_id,
                        "Duration (ms)",
                        &context,
                    ) {
                        Ok(VariableValue::Integer(ms)) => ms as u64,
                        Ok(VariableValue::Float(ms)) => ms as u64,
                        _ => 1000,
                    };
                    logger(format!("Delay: Sleeping for {}ms", duration_ms));
                    thread::sleep(std::time::Duration::from_millis(duration_ms));

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }
                NodeType::ForLoop => {
                    let start =
                        match Self::evaluate_input(&graph, current_node_id, "Start", &context) {
                            Ok(VariableValue::Integer(i)) => i,
                            _ => 0,
                        };
                    let end = match Self::evaluate_input(&graph, current_node_id, "End", &context) {
                        Ok(VariableValue::Integer(i)) => i,
                        _ => 10,
                    };

                    // Execute Loop body for each iteration
                    for i in start..end {
                        // Set Index output (we need to store it for GetVariable to access)
                        {
                            let mut ctx = context.lock().unwrap();
                            ctx.variables
                                .insert("__loop_index".into(), VariableValue::Integer(i));
                        }

                        // Execute the Loop body
                        if let Some(loop_body) = Self::follow_flow(&graph, current_node_id, "Loop")
                        {
                            Self::execute_flow_from(
                                graph.clone(),
                                loop_body,
                                context.clone(),
                                tx.clone(),
                                steps,
                                max_steps,
                            );
                        }
                    }

                    // Continue to Done
                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Done") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }
                NodeType::WhileLoop => {
                    let max_iterations = 1000; // Safety limit
                    let mut iteration = 0;

                    while iteration < max_iterations {
                        let condition =
                            Self::evaluate_input(&graph, current_node_id, "Condition", &context)
                                .unwrap_or(VariableValue::Boolean(false));

                        if !Self::to_bool(&condition) {
                            break;
                        }

                        // Execute the Loop body
                        if let Some(loop_body) = Self::follow_flow(&graph, current_node_id, "Loop")
                        {
                            Self::execute_flow_from(
                                graph.clone(),
                                loop_body,
                                context.clone(),
                                tx.clone(),
                                steps,
                                max_steps,
                            );
                        }
                        iteration += 1;
                    }

                    // Continue to Done
                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Done") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }
                NodeType::Sequence => {
                    // Execute each output in order
                    for i in 0..3 {
                        let port_name = format!("Then {}", i);
                        if let Some(next) = Self::follow_flow(&graph, current_node_id, &port_name) {
                            Self::execute_flow_from(
                                graph.clone(),
                                next,
                                context.clone(),
                                tx.clone(),
                                steps,
                                max_steps,
                            );
                        }
                    }
                    // Sequence node doesn't continue to a "Next", it just ends
                    break;
                }
                NodeType::Gate => {
                    let is_open = Self::evaluate_input(&graph, current_node_id, "Open", &context)
                        .unwrap_or(VariableValue::Boolean(true));

                    if Self::to_bool(&is_open) {
                        if let Some(next) = Self::follow_flow(&graph, current_node_id, "Out") {
                            current_node_id = next;
                        } else {
                            break;
                        }
                    } else {
                        // Gate is closed, stop execution
                        break;
                    }
                }
                NodeType::FileWrite => {
                    let path = Self::evaluate_input(&graph, current_node_id, "Path", &context)
                        .unwrap_or(VariableValue::String("".into()));
                    let content =
                        Self::evaluate_input(&graph, current_node_id, "Content", &context)
                            .unwrap_or(VariableValue::String("".into()));

                    let path_s = match path {
                        VariableValue::String(s) => s,
                        _ => "".to_string(),
                    };
                    let content_s = match content {
                        VariableValue::String(s) => s,
                        _ => "".to_string(),
                    };

                    match std::fs::write(&path_s, &content_s) {
                        Ok(_) => logger(format!("FileWrite: Successfully wrote to {}", path_s)),
                        Err(e) => logger(format!("FileWrite: Error writing to {}: {}", path_s, e)),
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                // === Module H: Array Mutation Nodes ===
                NodeType::ArrayPush => {
                    let var_name =
                        Self::evaluate_input(&graph, current_node_id, "Variable", &context)
                            .map(|v| Self::to_string(&v))
                            .unwrap_or_default();
                    let value = Self::evaluate_input(&graph, current_node_id, "Value", &context)
                        .unwrap_or(VariableValue::None);

                    {
                        let mut ctx = context.lock().unwrap();
                        if let Some(VariableValue::Array(arr)) = ctx.variables.get_mut(&var_name) {
                            arr.push(value);
                            logger(format!("ArrayPush: Added element to '{}'", var_name));
                        } else {
                            // Create new array if variable doesn't exist or isn't an array
                            ctx.variables
                                .insert(var_name.clone(), VariableValue::Array(vec![value]));
                            logger(format!("ArrayPush: Created new array '{}'", var_name));
                        }
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::ArrayPop => {
                    let var_name =
                        Self::evaluate_input(&graph, current_node_id, "Variable", &context)
                            .map(|v| Self::to_string(&v))
                            .unwrap_or_default();

                    {
                        let mut ctx = context.lock().unwrap();
                        if let Some(VariableValue::Array(arr)) = ctx.variables.get_mut(&var_name) {
                            if let Some(popped) = arr.pop() {
                                // Store popped value temporarily for output port
                                ctx.variables.insert("__array_pop_result".into(), popped);
                                logger(format!("ArrayPop: Removed element from '{}'", var_name));
                            } else {
                                ctx.variables
                                    .insert("__array_pop_result".into(), VariableValue::None);
                                logger(format!("ArrayPop: Array '{}' is empty", var_name));
                            }
                        } else {
                            ctx.variables
                                .insert("__array_pop_result".into(), VariableValue::None);
                            logger(format!("ArrayPop: Variable '{}' is not an array", var_name));
                        }
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::ArraySet => {
                    let var_name =
                        Self::evaluate_input(&graph, current_node_id, "Variable", &context)
                            .map(|v| Self::to_string(&v))
                            .unwrap_or_default();
                    let index = Self::evaluate_input(&graph, current_node_id, "Index", &context)
                        .map(|v| match v {
                            VariableValue::Integer(i) => i as usize,
                            VariableValue::Float(f) => f as usize,
                            _ => 0,
                        })
                        .unwrap_or(0);
                    let value = Self::evaluate_input(&graph, current_node_id, "Value", &context)
                        .unwrap_or(VariableValue::None);

                    {
                        let mut ctx = context.lock().unwrap();
                        if let Some(VariableValue::Array(arr)) = ctx.variables.get_mut(&var_name) {
                            if index < arr.len() {
                                arr[index] = value;
                                logger(format!("ArraySet: Set index {} of '{}'", index, var_name));
                            } else {
                                // Extend array if necessary
                                while arr.len() <= index {
                                    arr.push(VariableValue::None);
                                }
                                arr[index] = value;
                                logger(format!(
                                    "ArraySet: Extended '{}' and set index {}",
                                    var_name, index
                                ));
                            }
                        } else {
                            logger(format!("ArraySet: Variable '{}' is not an array", var_name));
                        }
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::HTTPRequest => {
                    let url = Self::evaluate_input(&graph, current_node_id, "URL", &context)
                        .map(|v| Self::to_string(&v))
                        .unwrap_or_default();
                    let method = Self::evaluate_input(&graph, current_node_id, "Method", &context)
                        .map(|v| Self::to_string(&v))
                        .unwrap_or_else(|_| "GET".to_string());
                    let body = Self::evaluate_input(&graph, current_node_id, "Body", &context)
                        .map(|v| Self::to_string(&v))
                        .unwrap_or_default();

                    logger(format!("HTTPRequest: {} {}", method.to_uppercase(), url));

                    // Simple synchronous HTTP using std::process::Command with curl
                    // Note: For production, consider using reqwest crate with async
                    let result = if method.to_uppercase() == "POST" {
                        std::process::Command::new("curl")
                            .args(["-s", "-X", "POST", "-d", &body, &url])
                            .output()
                    } else {
                        std::process::Command::new("curl")
                            .args(["-s", &url])
                            .output()
                    };

                    {
                        let mut ctx = context.lock().unwrap();
                        let node_id_str = current_node_id.to_string();
                        match result {
                            Ok(output) => {
                                let response = String::from_utf8_lossy(&output.stdout).to_string();
                                ctx.variables.insert(
                                    format!("__out_{}_Response", node_id_str),
                                    VariableValue::String(response.clone()),
                                );
                                ctx.variables.insert(
                                    format!("__out_{}_Success", node_id_str),
                                    VariableValue::Boolean(output.status.success()),
                                );
                                logger(format!(
                                    "HTTPRequest: Response received ({} bytes)",
                                    response.len()
                                ));
                            }
                            Err(e) => {
                                ctx.variables.insert(
                                    format!("__out_{}_Response", node_id_str),
                                    VariableValue::String("".into()),
                                );
                                ctx.variables.insert(
                                    format!("__out_{}_Success", node_id_str),
                                    VariableValue::Boolean(false),
                                );
                                logger(format!("HTTPRequest: Error - {}", e));
                            }
                        }
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                // === Module A: Desktop Input Automation ===
                NodeType::Click => {
                    let x = Self::evaluate_input(&graph, current_node_id, "X", &context)
                        .map(|v| match v {
                            VariableValue::Integer(i) => i as i32,
                            VariableValue::Float(f) => f as i32,
                            _ => 0,
                        })
                        .unwrap_or(0);
                    let y = Self::evaluate_input(&graph, current_node_id, "Y", &context)
                        .map(|v| match v {
                            VariableValue::Integer(i) => i as i32,
                            VariableValue::Float(f) => f as i32,
                            _ => 0,
                        })
                        .unwrap_or(0);

                    logger(format!("Click: ({}, {})", x, y));

                    if let Some(enigo) = &mut enigo {
                        let _ = enigo.move_mouse(x, y, Coordinate::Abs);
                        let _ = enigo.button(Button::Left, Direction::Click);
                    } else {
                        logger("Click Error: Enigo not initialized".into());
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::DoubleClick => {
                    let x = Self::evaluate_input(&graph, current_node_id, "X", &context)
                        .map(|v| match v {
                            VariableValue::Integer(i) => i as i32,
                            VariableValue::Float(f) => f as i32,
                            _ => 0,
                        })
                        .unwrap_or(0);
                    let y = Self::evaluate_input(&graph, current_node_id, "Y", &context)
                        .map(|v| match v {
                            VariableValue::Integer(i) => i as i32,
                            VariableValue::Float(f) => f as i32,
                            _ => 0,
                        })
                        .unwrap_or(0);

                    logger(format!("DoubleClick: ({}, {})", x, y));

                    if let Some(enigo) = &mut enigo {
                        let _ = enigo.move_mouse(x, y, Coordinate::Abs);
                        let _ = enigo.button(Button::Left, Direction::Click);
                        let _ = enigo.button(Button::Left, Direction::Click);
                    } else {
                         logger("DoubleClick Error: Enigo not initialized".into());
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::RightClick => {
                    let x = Self::evaluate_input(&graph, current_node_id, "X", &context)
                        .map(|v| match v {
                            VariableValue::Integer(i) => i as i32,
                            VariableValue::Float(f) => f as i32,
                            _ => 0,
                        })
                        .unwrap_or(0);
                    let y = Self::evaluate_input(&graph, current_node_id, "Y", &context)
                        .map(|v| match v {
                            VariableValue::Integer(i) => i as i32,
                            VariableValue::Float(f) => f as i32,
                            _ => 0,
                        })
                        .unwrap_or(0);

                    logger(format!("RightClick: ({}, {})", x, y));

                    if let Some(enigo) = &mut enigo {
                        let _ = enigo.move_mouse(x, y, Coordinate::Abs);
                        let _ = enigo.button(Button::Right, Direction::Click);
                    } else {
                        logger("RightClick Error: Enigo not initialized".into());
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::MouseMove => {
                    let x = Self::evaluate_input(&graph, current_node_id, "X", &context)
                        .map(|v| match v {
                            VariableValue::Integer(i) => i as i32,
                            VariableValue::Float(f) => f as i32,
                            _ => 0,
                        })
                        .unwrap_or(0);
                    let y = Self::evaluate_input(&graph, current_node_id, "Y", &context)
                        .map(|v| match v {
                            VariableValue::Integer(i) => i as i32,
                            VariableValue::Float(f) => f as i32,
                            _ => 0,
                        })
                        .unwrap_or(0);

                    logger(format!("MouseMove: ({}, {})", x, y));

                    if let Some(enigo) = &mut enigo {
                        let _ = enigo.move_mouse(x, y, Coordinate::Abs);
                    } else {
                        logger("MouseMove Error: Enigo not initialized".into());
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::MouseDown => {
                    let button_str =
                        Self::evaluate_input(&graph, current_node_id, "Button", &context)
                            .map(|v| Self::to_string(&v))
                            .unwrap_or_else(|_| "left".to_string());

                    let button = match button_str.to_lowercase().as_str() {
                        "right" => Button::Right,
                        "middle" => Button::Middle,
                        _ => Button::Left,
                    };

                    let x_val = Self::evaluate_input(&graph, current_node_id, "X", &context).unwrap_or(VariableValue::Integer(0));
                    let y_val = Self::evaluate_input(&graph, current_node_id, "Y", &context).unwrap_or(VariableValue::Integer(0));
                    
                    let x = match x_val {
                        VariableValue::Integer(i) => i as i32,
                        VariableValue::Float(f) => f as i32,
                        _ => 0,
                    };
                    let y = match y_val {
                        VariableValue::Integer(i) => i as i32,
                        VariableValue::Float(f) => f as i32,
                        _ => 0,
                    };

                    // Check if inputs are connected
                    let x_connected = graph.connections.iter().any(|c| c.to_node == current_node_id && c.to_port == "X");
                    let y_connected = graph.connections.iter().any(|c| c.to_node == current_node_id && c.to_port == "Y");

                    // Move only if inputs are explicitly provided (connected or non-zero)
                    let should_move = x_connected || y_connected || x != 0 || y != 0;

                    if should_move {
                        logger(format!("MouseDown: {} at ({}, {})", button_str, x, y));
                    } else {
                        logger(format!("MouseDown: {} (Current Position)", button_str));
                    }

                    if let Some(enigo) = &mut enigo {
                        if should_move {
                            let _ = enigo.move_mouse(x, y, Coordinate::Abs);
                            // macOS requires a delay after move before press, otherwise events are ignored
                            std::thread::sleep(std::time::Duration::from_millis(50));
                        }
                        let _ = enigo.button(button, Direction::Press);
                    } else {
                        logger("MouseDown Error: Enigo not initialized".into());
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::MouseUp => {
                     let button_str =
                        Self::evaluate_input(&graph, current_node_id, "Button", &context)
                            .map(|v| Self::to_string(&v))
                            .unwrap_or_else(|_| "left".to_string());

                    let button = match button_str.to_lowercase().as_str() {
                        "right" => Button::Right,
                        "middle" => Button::Middle,
                        _ => Button::Left,
                    };

                    let x_val = Self::evaluate_input(&graph, current_node_id, "X", &context).unwrap_or(VariableValue::Integer(0));
                    let y_val = Self::evaluate_input(&graph, current_node_id, "Y", &context).unwrap_or(VariableValue::Integer(0));
                    
                    let x = match x_val {
                        VariableValue::Integer(i) => i as i32,
                        VariableValue::Float(f) => f as i32,
                        _ => 0,
                    };
                    let y = match y_val {
                        VariableValue::Integer(i) => i as i32,
                        VariableValue::Float(f) => f as i32,
                        _ => 0,
                    };

                    // Check if inputs are connected
                    let x_connected = graph.connections.iter().any(|c| c.to_node == current_node_id && c.to_port == "X");
                    let y_connected = graph.connections.iter().any(|c| c.to_node == current_node_id && c.to_port == "Y");

                    // Move only if inputs are explicitly provided (connected or non-zero)
                    let should_move = x_connected || y_connected || x != 0 || y != 0;

                    if should_move {
                        logger(format!("MouseUp: {} at ({}, {})", button_str, x, y));
                    } else {
                        logger(format!("MouseUp: {} (Current Position)", button_str));
                    }

                    if let Some(enigo) = &mut enigo {
                        if should_move {
                            let _ = enigo.move_mouse(x, y, Coordinate::Abs);
                            // macOS requires a delay after move before release
                            std::thread::sleep(std::time::Duration::from_millis(50));
                        }
                        let _ = enigo.button(button, Direction::Release);
                    } else {
                        logger("MouseUp Error: Enigo not initialized".into());
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::Scroll => {
                    let x = Self::evaluate_input(&graph, current_node_id, "X", &context)
                        .map(|v| match v {
                            VariableValue::Integer(i) => i as i32,
                            VariableValue::Float(f) => f as i32,
                            _ => 0,
                        })
                        .unwrap_or(0);
                    let y = Self::evaluate_input(&graph, current_node_id, "Y", &context)
                        .map(|v| match v {
                            VariableValue::Integer(i) => i as i32,
                            VariableValue::Float(f) => f as i32,
                            _ => 0,
                        })
                        .unwrap_or(-3);

                    logger(format!("Scroll: ({}, {})", x, y));

                    if let Some(enigo) = &mut enigo {
                        let _ = enigo.scroll(x, enigo::Axis::Horizontal);
                        let _ = enigo.scroll(y, enigo::Axis::Vertical);
                    } else {
                        logger("Scroll Error: Enigo not initialized".into());
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::KeyPress => {
                    let key_str = Self::evaluate_input(&graph, current_node_id, "Key", &context)
                        .map(|v| Self::to_string(&v))
                        .unwrap_or_else(|_| "Return".to_string());

                    logger(format!("KeyPress: {}", key_str));

                    if let Some(enigo) = &mut enigo {
                        if let Some(key) = Self::string_to_key(&key_str) {
                            let _ = enigo.key(key, Direction::Press);
                            thread::sleep(Duration::from_millis(50));
                            let _ = enigo.key(key, Direction::Release);
                        } else {
                            if let Some(c) = key_str.chars().next() {
                                let _ = enigo.key(Key::Unicode(c), Direction::Click);
                            }
                        }
                    } else {
                        logger("KeyPress Error: Enigo not initialized".into());
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::KeyDown => {
                    let key_str = Self::evaluate_input(&graph, current_node_id, "Key", &context)
                        .map(|v| Self::to_string(&v))
                        .unwrap_or_else(|_| "Shift".to_string());

                    logger(format!("KeyDown: {}", key_str));

                    if let Some(enigo) = &mut enigo {
                        if let Some(key) = Self::string_to_key(&key_str) {
                            let _ = enigo.key(key, Direction::Press);
                        } else {
                            if let Some(c) = key_str.chars().next() {
                                let _ = enigo.key(Key::Unicode(c), Direction::Press);
                            }
                        }
                    } else {
                        logger("KeyDown Error: Enigo not initialized".into());
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::KeyUp => {
                    let key_str = Self::evaluate_input(&graph, current_node_id, "Key", &context)
                        .map(|v| Self::to_string(&v))
                        .unwrap_or_else(|_| "Shift".to_string());

                    logger(format!("KeyUp: {}", key_str));

                    if let Some(enigo) = &mut enigo {
                        if let Some(key) = Self::string_to_key(&key_str) {
                            let _ = enigo.key(key, Direction::Release);
                        } else {
                            if let Some(c) = key_str.chars().next() {
                                let _ = enigo.key(Key::Unicode(c), Direction::Release);
                            }
                        }
                    } else {
                        logger("KeyUp Error: Enigo not initialized".into());
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::TypeText => {
                    let text = Self::evaluate_input(&graph, current_node_id, "Text", &context)
                        .map(|v| Self::to_string(&v))
                        .unwrap_or_default();

                    logger(format!("TypeText: \"{}\"", text));

                    if let Some(enigo) = &mut enigo {
                         let _ = enigo.text(&text);
                    } else {
                         logger("TypeText Error: Enigo not initialized".into());
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::TypeString => {
                    let text = Self::evaluate_input(&graph, current_node_id, "Text", &context)
                        .map(|v| Self::to_string(&v))
                        .unwrap_or_default();
                    let delay_ms = Self::evaluate_input(&graph, current_node_id, "Delay", &context)
                        .map(|v| match v {
                            VariableValue::Integer(i) => i as u64,
                            VariableValue::Float(f) => f as u64,
                            _ => 50,
                        })
                        .unwrap_or(50);

                    logger(format!("TypeString: \"{}\" (delay: {}ms)", text, delay_ms));

                    if let Some(enigo) = &mut enigo {
                        for c in text.chars() {
                            // Handle special characters that need key conversion
                            let key_pressed = match c {
                                ' ' => {
                                    let _ = enigo.key(Key::Space, Direction::Click);
                                    true
                                }
                                '\n' => {
                                    let _ = enigo.key(Key::Return, Direction::Click);
                                    true
                                }
                                '\t' => {
                                    let _ = enigo.key(Key::Tab, Direction::Click);
                                    true
                                }
                                _ => {
                                    // For regular characters, use Unicode key press
                                    let _ = enigo.key(Key::Unicode(c), Direction::Click);
                                    true
                                }
                            };
                            
                            if key_pressed && delay_ms > 0 {
                                thread::sleep(Duration::from_millis(delay_ms));
                            }
                        }
                    } else {
                        logger("TypeString Error: Enigo not initialized".into());
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::HotKey => {
                    let key_str = Self::evaluate_input(&graph, current_node_id, "Key", &context)
                        .map(|v| Self::to_string(&v))
                        .unwrap_or_else(|_| "c".to_string());
                    let ctrl = Self::evaluate_input(&graph, current_node_id, "Ctrl", &context)
                        .map(|v| Self::to_bool(&v))
                        .unwrap_or(false);
                    let shift = Self::evaluate_input(&graph, current_node_id, "Shift", &context)
                        .map(|v| Self::to_bool(&v))
                        .unwrap_or(false);
                    let alt = Self::evaluate_input(&graph, current_node_id, "Alt", &context)
                        .map(|v| Self::to_bool(&v))
                        .unwrap_or(false);
                    let meta = Self::evaluate_input(&graph, current_node_id, "Command", &context)
                        .map(|v| Self::to_bool(&v))
                        .unwrap_or(false);

                    let mut modifiers = Vec::new();
                    if ctrl {
                        modifiers.push("Ctrl");
                    }
                    if shift {
                        modifiers.push("Shift");
                    }
                    if alt {
                        modifiers.push("Alt");
                    }
                    if meta {
                        modifiers.push("Command");
                    }
                    logger(format!("HotKey: {}+{}", modifiers.join("+"), key_str));

                    // Refactored HotKey to reuse enigo instance
                    if let Some(enigo) = &mut enigo {
                        // Press modifiers
                        if ctrl {
                            let _ = enigo.key(Key::Control, Direction::Press);
                        }
                        if shift {
                            let _ = enigo.key(Key::Shift, Direction::Press);
                        }
                        if alt {
                            let _ = enigo.key(Key::Alt, Direction::Press);
                        }
                        if meta {
                            let _ = enigo.key(Key::Meta, Direction::Press);
                        }

                        thread::sleep(Duration::from_millis(100));

                        if let Some(key) = Self::string_to_key(&key_str) {
                            let _ = enigo.key(key, Direction::Click);
                        } else if let Some(c) = key_str.chars().next() {
                            let _ = enigo.key(Key::Unicode(c), Direction::Click);
                        }

                        thread::sleep(Duration::from_millis(50));

                        // Release modifiers (in reverse order)
                        if meta {
                            let _ = enigo.key(Key::Meta, Direction::Release);
                        }
                        if alt {
                            let _ = enigo.key(Key::Alt, Direction::Release);
                        }
                        if shift {
                            let _ = enigo.key(Key::Shift, Direction::Release);
                        }
                        if ctrl {
                            let _ = enigo.key(Key::Control, Direction::Release);
                        }
                    } else {
                        logger("HotKey Error: Enigo not initialized".into());
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                // === Module G: System Control ===
                NodeType::RunCommand => {
                    let cmd = Self::evaluate_input(&graph, current_node_id, "Command", &context)
                        .map(|v| Self::to_string(&v))
                        .unwrap_or_default();
                    let args_str = Self::evaluate_input(&graph, current_node_id, "Args", &context)
                        .map(|v| Self::to_string(&v))
                        .unwrap_or_default();

                    logger(format!("RunCommand: {} {}", cmd, args_str));

                    let args: Vec<&str> = args_str.split_whitespace().collect();

                    match std::process::Command::new(&cmd).args(&args).output() {
                        Ok(output) => {
                            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                            // let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                            let exit_code = output.status.code().unwrap_or(0) as i64;
                            let success = output.status.success();

                            let mut ctx = context.lock().unwrap();
                            let node_id_str = current_node_id.to_string();
                            ctx.variables.insert(
                                format!("__out_{}_Output", node_id_str),
                                VariableValue::String(stdout),
                            );
                            ctx.variables.insert(
                                format!("__out_{}_ExitCode", node_id_str),
                                VariableValue::Integer(exit_code),
                            );
                            ctx.variables.insert(
                                format!("__out_{}_Success", node_id_str),
                                VariableValue::Boolean(success),
                            );
                        }
                        Err(e) => {
                            logger(format!("RunCommand Error: {}", e));
                            let mut ctx = context.lock().unwrap();
                            let node_id_str = current_node_id.to_string();
                            ctx.variables.insert(
                                format!("__out_{}_Success", node_id_str),
                                VariableValue::Boolean(false),
                            );
                        }
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::LaunchApp => {
                    let path = Self::evaluate_input(&graph, current_node_id, "Path", &context)
                        .map(|v| Self::to_string(&v))
                        .unwrap_or_default();
                    let args_str = Self::evaluate_input(&graph, current_node_id, "Args", &context)
                        .map(|v| Self::to_string(&v))
                        .unwrap_or_default();

                    logger(format!("LaunchApp: {}", path));

                    let args: Vec<&str> = args_str.split_whitespace().collect();

                    #[cfg(target_os = "macos")]
                    let result = std::process::Command::new("open")
                        .arg(&path)
                        .args(&args)
                        .spawn();
                    #[cfg(target_os = "windows")]
                    let result = std::process::Command::new("cmd")
                        .arg("/C")
                        .arg("start")
                        .arg(&path)
                        .args(&args)
                        .spawn();
                    #[cfg(target_os = "linux")]
                    let result = std::process::Command::new("xdg-open")
                        .arg(&path)
                        .args(&args)
                        .spawn();
                    #[cfg(not(any(
                        target_os = "macos",
                        target_os = "windows",
                        target_os = "linux"
                    )))]
                    let result = std::process::Command::new(&path).args(&args).spawn();

                    let success = result.is_ok();
                    if let Err(e) = result {
                        logger(format!("LaunchApp Error: {}", e));
                    }

                    {
                        let mut ctx = context.lock().unwrap();
                        let node_id_str = current_node_id.to_string();
                        ctx.variables.insert(
                            format!("__out_{}_Success", node_id_str),
                            VariableValue::Boolean(success),
                        );
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::CloseApp => {
                    let name = Self::evaluate_input(&graph, current_node_id, "Name", &context)
                        .map(|v| Self::to_string(&v))
                        .unwrap_or_default();

                    logger(format!("CloseApp: {}", name));

                    #[cfg(target_os = "windows")]
                    let result = std::process::Command::new("taskkill")
                        .args(["/IM", &name, "/F"])
                        .output();
                    #[cfg(not(target_os = "windows"))]
                    let result = std::process::Command::new("pkill")
                        .arg("-x")
                        .arg(&name)
                        .output();

                    let success = result.is_ok() && result.unwrap().status.success();

                    {
                        let mut ctx = context.lock().unwrap();
                        let node_id_str = current_node_id.to_string();
                        ctx.variables.insert(
                            format!("__out_{}_Success", node_id_str),
                            VariableValue::Boolean(success),
                        );
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::GetWindowPosition => {
                    // Trigger evaluation to cache results
                   if let Some(node) = graph.nodes.get(&current_node_id) {
                        let _ = Self::evaluate_node(&graph, node, "X", &context);
                   }
                    
                    logger("GetWindowPosition: Executed".to_string());

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::FocusWindow => {
                    let title = Self::evaluate_input(&graph, current_node_id, "Title", &context)
                        .map(|v| Self::to_string(&v))
                        .unwrap_or_default();

                    logger(format!("FocusWindow: {}", title));

                    // Platform-specific implementation
                    #[cfg(target_os = "macos")]
                    let success = {
                        // Use AppleScript to find and focus window by title
                        let escaped_title = title.replace("\"", "\\\"");
                        let script = format!(
                            r#"tell application "System Events"
                                set foundWindow to false
                                repeat with proc in (every process whose visible is true)
                                    try
                                        repeat with w in windows of proc
                                            if name of w contains "{}" then
                                                set frontmost of proc to true
                                                perform action "AXRaise" of w
                                                set foundWindow to true
                                                exit repeat
                                            end if
                                        end repeat
                                        if foundWindow then exit repeat
                                    end try
                                end repeat
                                return foundWindow
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
                                output.status.success() && stdout.trim() == "true"
                            }
                            Err(e) => {
                                logger(format!("FocusWindow Error: {}", e));
                                false
                            }
                        }
                    };

                    #[cfg(target_os = "windows")]
                    let success = {
                        // Windows: Use PowerShell to find and focus window
                        logger(format!("FocusWindow (Windows stub): {}", title));
                        true // Stub for Windows
                    };

                    #[cfg(target_os = "linux")]
                    let success = {
                        // Linux: Use wmctrl or xdotool
                        let result = std::process::Command::new("wmctrl")
                            .args(["-a", &title])
                            .output();
                        result.map(|o| o.status.success()).unwrap_or(false)
                    };

                    #[cfg(not(any(
                        target_os = "macos",
                        target_os = "windows",
                        target_os = "linux"
                    )))]
                    let success = false;

                    {
                        let mut ctx = context.lock().unwrap();
                        let node_id_str = current_node_id.to_string();
                        ctx.variables.insert(
                            format!("__out_{}_Success", node_id_str),
                            VariableValue::Boolean(success),
                        );
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::SetWindowPosition => {
                    let title = Self::evaluate_input(&graph, current_node_id, "Title", &context)
                        .map(|v| Self::to_string(&v))
                        .unwrap_or_default();
                    let x = Self::evaluate_input(&graph, current_node_id, "X", &context)
                        .map(|v| Self::to_float(&v) as i32)
                        .unwrap_or(0);
                    let y = Self::evaluate_input(&graph, current_node_id, "Y", &context)
                        .map(|v| Self::to_float(&v) as i32)
                        .unwrap_or(0);
                    let w = Self::evaluate_input(&graph, current_node_id, "Width", &context)
                        .map(|v| Self::to_float(&v) as i32)
                        .unwrap_or(800);
                    let h = Self::evaluate_input(&graph, current_node_id, "Height", &context)
                        .map(|v| Self::to_float(&v) as i32)
                        .unwrap_or(600);

                    logger(format!(
                        "SetWindowPosition: '{}' -> {},{}, {}x{}",
                        title, x, y, w, h
                    ));

                    // Platform-specific implementation
                    #[cfg(target_os = "macos")]
                    let success = {
                        let escaped_title = title.replace("\"", "\\\"");
                        // AppleScript to set window position and size
                        let script = format!(
                            r#"tell application "System Events"
                                set foundWindow to false
                                repeat with proc in (every process whose visible is true)
                                    try
                                        repeat with w in windows of proc
                                            if name of w contains "{title}" then
                                                set position of w to {{{x}, {y}}}
                                                set size of w to {{{w}, {h}}}
                                                set foundWindow to true
                                                exit repeat
                                            end if
                                        end repeat
                                        if foundWindow then exit repeat
                                    end try
                                end repeat
                                return foundWindow
                            end tell"#,
                            title = escaped_title,
                            x = x,
                            y = y,
                            w = w,
                            h = h
                        );
                        match std::process::Command::new("osascript")
                            .arg("-e")
                            .arg(&script)
                            .output()
                        {
                            Ok(output) => {
                                let stdout = String::from_utf8_lossy(&output.stdout);
                                output.status.success() && stdout.trim() == "true"
                            }
                            Err(e) => {
                                logger(format!("SetWindowPosition Error: {}", e));
                                false
                            }
                        }
                    };

                    #[cfg(target_os = "windows")]
                    let success = {
                        logger(format!(
                            "SetWindowPosition (Windows stub): {} -> {},{},{}x{}",
                            title, x, y, w, h
                        ));
                        true // Stub for Windows
                    };

                    #[cfg(target_os = "linux")]
                    let success = {
                        // Linux: Use wmctrl to move window
                        let result = std::process::Command::new("wmctrl")
                            .args(["-r", &title, "-e", &format!("0,{},{},{},{}", x, y, w, h)])
                            .output();
                        result.map(|o| o.status.success()).unwrap_or(false)
                    };

                    #[cfg(not(any(
                        target_os = "macos",
                        target_os = "windows",
                        target_os = "linux"
                    )))]
                    let success = false;

                    {
                        let mut ctx = context.lock().unwrap();
                        let node_id_str = current_node_id.to_string();
                        ctx.variables.insert(
                            format!("__out_{}_Success", node_id_str),
                            VariableValue::Boolean(success),
                        );
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                // === Module C: Screenshot & Image Tools ===
                NodeType::ScreenCapture => {
                    let display_index =
                        Self::evaluate_input(&graph, current_node_id, "Display", &context)
                            .map(|v| match v {
                                VariableValue::Integer(i) => i as usize,
                                VariableValue::Float(f) => f as usize,
                                _ => 0,
                            })
                            .unwrap_or(0);

                    logger(format!("ScreenCapture: Display {}", display_index));

                    // Ensure screenshots directory exists
                    let _ = std::fs::create_dir_all("scripts/screenshots");

                    // Capture screen using xcap
                    let (success, image_path) = match Monitor::all() {
                        Ok(monitors) => {
                            if let Some(monitor) = monitors.get(display_index) {
                                match monitor.capture_image() {
                                    Ok(image) => {
                                        let timestamp =
                                            chrono::Local::now().format("%Y%m%d_%H%M%S_%3f");
                                        let filename = format!(
                                            "scripts/screenshots/capture_{}.png",
                                            timestamp
                                        );
                                        match image.save(&filename) {
                                            Ok(_) => {
                                                logger(format!(
                                                    "ScreenCapture: Saved to {}",
                                                    filename
                                                ));
                                                (true, filename)
                                            }
                                            Err(e) => {
                                                logger(format!(
                                                    "ScreenCapture: Save error - {}",
                                                    e
                                                ));
                                                (false, String::new())
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        logger(format!("ScreenCapture: Capture error - {}", e));
                                        (false, String::new())
                                    }
                                }
                            } else {
                                logger(format!(
                                    "ScreenCapture: Display {} not found, only {} displays available",
                                    display_index,
                                    monitors.len()
                                ));
                                (false, String::new())
                            }
                        }
                        Err(e) => {
                            logger(format!("ScreenCapture: Monitor error - {}", e));
                            (false, String::new())
                        }
                    };

                    {
                        let mut ctx = context.lock().unwrap();
                        let node_id_str = current_node_id.to_string();
                        ctx.variables.insert(
                            format!("__out_{}_ImagePath", node_id_str),
                            VariableValue::String(image_path),
                        );
                        ctx.variables.insert(
                            format!("__out_{}_Success", node_id_str),
                            VariableValue::Boolean(success),
                        );
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::SaveScreenshot => {
                    let image_path =
                        Self::evaluate_input(&graph, current_node_id, "ImagePath", &context)
                            .map(|v| Self::to_string(&v))
                            .unwrap_or_default();
                    let filename =
                        Self::evaluate_input(&graph, current_node_id, "Filename", &context)
                            .map(|v| Self::to_string(&v))
                            .unwrap_or_else(|_| "screenshot.png".to_string());

                    logger(format!("SaveScreenshot: {} -> {}", image_path, filename));

                    // Ensure target directory exists
                    if let Some(parent) = std::path::Path::new(&filename).parent() {
                        let _ = std::fs::create_dir_all(parent);
                    }

                    let (success, saved_path) = if !image_path.is_empty() {
                        match std::fs::copy(&image_path, &filename) {
                            Ok(_) => {
                                logger(format!("SaveScreenshot: Saved to {}", filename));
                                (true, filename.clone())
                            }
                            Err(e) => {
                                logger(format!("SaveScreenshot: Copy error - {}", e));
                                (false, String::new())
                            }
                        }
                    } else {
                        logger("SaveScreenshot: No image path provided".to_string());
                        (false, String::new())
                    };

                    {
                        let mut ctx = context.lock().unwrap();
                        let node_id_str = current_node_id.to_string();
                        ctx.variables.insert(
                            format!("__out_{}_SavedPath", node_id_str),
                            VariableValue::String(saved_path),
                        );
                        ctx.variables.insert(
                            format!("__out_{}_Success", node_id_str),
                            VariableValue::Boolean(success),
                        );
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::RegionCapture => {
                    let x = Self::evaluate_input(&graph, current_node_id, "X", &context)
                        .map(|v| Self::to_float(&v) as u32)
                        .unwrap_or(0);
                    let y = Self::evaluate_input(&graph, current_node_id, "Y", &context)
                        .map(|v| Self::to_float(&v) as u32)
                        .unwrap_or(0);
                    let width = Self::evaluate_input(&graph, current_node_id, "Width", &context)
                        .map(|v| Self::to_float(&v) as u32)
                        .unwrap_or(200);
                    let height = Self::evaluate_input(&graph, current_node_id, "Height", &context)
                        .map(|v| Self::to_float(&v) as u32)
                        .unwrap_or(100);
                    let custom_filename =
                        Self::evaluate_input(&graph, current_node_id, "Filename", &context)
                            .map(|v| Self::to_string(&v))
                            .unwrap_or_default();

                    logger(format!(
                        "RegionCapture: ({},{}) {}x{}",
                        x, y, width, height
                    ));

                    // Ensure templates directory exists
                    let _ = std::fs::create_dir_all("scripts/templates");

                    // Capture screen and crop to region
                    let (success, image_path) = match Monitor::all() {
                        Ok(monitors) => {
                            if let Some(monitor) = monitors.first() {
                                match monitor.capture_image() {
                                    Ok(full_image) => {
                                        // Validate bounds
                                        let img_width = full_image.width();
                                        let img_height = full_image.height();
                                        
                                        if x >= img_width || y >= img_height {
                                            logger(format!(
                                                "RegionCapture: Start position ({},{}) out of bounds ({}x{})",
                                                x, y, img_width, img_height
                                            ));
                                            (false, String::new())
                                        } else {
                                            // Clamp width/height to image bounds
                                            let crop_width = width.min(img_width - x);
                                            let crop_height = height.min(img_height - y);
                                            
                                            // Crop the image
                                            let cropped = image::imageops::crop_imm(
                                                &full_image, x, y, crop_width, crop_height
                                            ).to_image();
                                            
                                            // Generate filename
                                            let filename = if custom_filename.is_empty() {
                                                let timestamp = chrono::Local::now()
                                                    .format("%Y%m%d_%H%M%S_%3f");
                                                format!("scripts/templates/region_{}.png", timestamp)
                                            } else if custom_filename.contains('/') || custom_filename.contains('\\') {
                                                // User provided full path
                                                custom_filename.clone()
                                            } else {
                                                // Just filename, put in templates folder
                                                format!("scripts/templates/{}", custom_filename)
                                            };
                                            
                                            // Save cropped image
                                            match cropped.save(&filename) {
                                                Ok(_) => {
                                                    logger(format!(
                                                        "RegionCapture: Saved {}x{} to {}",
                                                        crop_width, crop_height, filename
                                                    ));
                                                    (true, filename)
                                                }
                                                Err(e) => {
                                                    logger(format!(
                                                        "RegionCapture: Save error - {}",
                                                        e
                                                    ));
                                                    (false, String::new())
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        logger(format!("RegionCapture: Capture error - {}", e));
                                        (false, String::new())
                                    }
                                }
                            } else {
                                logger("RegionCapture: No monitors found".to_string());
                                (false, String::new())
                            }
                        }
                        Err(e) => {
                            logger(format!("RegionCapture: Monitor error - {}", e));
                            (false, String::new())
                        }
                    };

                    {
                        let mut ctx = context.lock().unwrap();
                        let node_id_str = current_node_id.to_string();
                        ctx.variables.insert(
                            format!("__out_{}_ImagePath", node_id_str),
                            VariableValue::String(image_path),
                        );
                        ctx.variables.insert(
                            format!("__out_{}_Success", node_id_str),
                            VariableValue::Boolean(success),
                        );
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                // === Module D: Image Recognition ===
                NodeType::GetPixelColor => {
                    let x = Self::evaluate_input(&graph, current_node_id, "X", &context)
                        .map(|v| match v {
                            VariableValue::Integer(i) => i as u32,
                            VariableValue::Float(f) => f as u32,
                            _ => 0,
                        })
                        .unwrap_or(0);
                    let y = Self::evaluate_input(&graph, current_node_id, "Y", &context)
                        .map(|v| match v {
                            VariableValue::Integer(i) => i as u32,
                            VariableValue::Float(f) => f as u32,
                            _ => 0,
                        })
                        .unwrap_or(0);

                    logger(format!("GetPixelColor: ({}, {})", x, y));

                    // Capture screen and get pixel color
                    let (r, g, b, success) = match xcap::Monitor::all() {
                        Ok(monitors) => {
                            if let Some(monitor) = monitors.first() {
                                match monitor.capture_image() {
                                    Ok(img) => {
                                        if x < img.width() && y < img.height() {
                                            let pixel = img.get_pixel(x, y);
                                            (
                                                pixel[0] as i64,
                                                pixel[1] as i64,
                                                pixel[2] as i64,
                                                true,
                                            )
                                        } else {
                                            logger(format!(
                                                "GetPixelColor: Coordinates out of bounds"
                                            ));
                                            (0, 0, 0, false)
                                        }
                                    }
                                    Err(e) => {
                                        logger(format!("GetPixelColor: Capture error - {}", e));
                                        (0, 0, 0, false)
                                    }
                                }
                            } else {
                                logger("GetPixelColor: No monitors found".to_string());
                                (0, 0, 0, false)
                            }
                        }
                        Err(e) => {
                            logger(format!("GetPixelColor: Monitor error - {}", e));
                            (0, 0, 0, false)
                        }
                    };

                    {
                        let mut ctx = context.lock().unwrap();
                        let node_id_str = current_node_id.to_string();
                        ctx.variables.insert(
                            format!("__out_{}_R", node_id_str),
                            VariableValue::Integer(r),
                        );
                        ctx.variables.insert(
                            format!("__out_{}_G", node_id_str),
                            VariableValue::Integer(g),
                        );
                        ctx.variables.insert(
                            format!("__out_{}_B", node_id_str),
                            VariableValue::Integer(b),
                        );
                        ctx.variables.insert(
                            format!("__out_{}_Success", node_id_str),
                            VariableValue::Boolean(success),
                        );
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::FindColor => {
                    let target_r = Self::evaluate_input(&graph, current_node_id, "R", &context)
                        .map(|v| Self::to_float(&v) as u8)
                        .unwrap_or(255);
                    let target_g = Self::evaluate_input(&graph, current_node_id, "G", &context)
                        .map(|v| Self::to_float(&v) as u8)
                        .unwrap_or(0);
                    let target_b = Self::evaluate_input(&graph, current_node_id, "B", &context)
                        .map(|v| Self::to_float(&v) as u8)
                        .unwrap_or(0);
                    let tolerance =
                        Self::evaluate_input(&graph, current_node_id, "Tolerance", &context)
                            .map(|v| Self::to_float(&v) as i32)
                            .unwrap_or(10);
                    let region_x =
                        Self::evaluate_input(&graph, current_node_id, "RegionX", &context)
                            .map(|v| Self::to_float(&v) as u32)
                            .unwrap_or(0);
                    let region_y =
                        Self::evaluate_input(&graph, current_node_id, "RegionY", &context)
                            .map(|v| Self::to_float(&v) as u32)
                            .unwrap_or(0);
                    let region_w =
                        Self::evaluate_input(&graph, current_node_id, "RegionW", &context)
                            .map(|v| Self::to_float(&v) as u32)
                            .unwrap_or(1920);
                    let region_h =
                        Self::evaluate_input(&graph, current_node_id, "RegionH", &context)
                            .map(|v| Self::to_float(&v) as u32)
                            .unwrap_or(1080);

                    logger(format!(
                        "FindColor: RGB({},{},{}) tolerance={} in region ({},{})x{}x{}",
                        target_r,
                        target_g,
                        target_b,
                        tolerance,
                        region_x,
                        region_y,
                        region_w,
                        region_h
                    ));

                    let (found_x, found_y, found) = match xcap::Monitor::all() {
                        Ok(monitors) => {
                            if let Some(monitor) = monitors.first() {
                                match monitor.capture_image() {
                                    Ok(img) => {
                                        let mut result = (0i64, 0i64, false);
                                        let end_x = (region_x + region_w).min(img.width());
                                        let end_y = (region_y + region_h).min(img.height());

                                        'outer: for py in region_y..end_y {
                                            for px in region_x..end_x {
                                                let pixel = img.get_pixel(px, py);
                                                let dr = (pixel[0] as i32 - target_r as i32).abs();
                                                let dg = (pixel[1] as i32 - target_g as i32).abs();
                                                let db = (pixel[2] as i32 - target_b as i32).abs();
                                                if dr <= tolerance
                                                    && dg <= tolerance
                                                    && db <= tolerance
                                                {
                                                    result = (px as i64, py as i64, true);
                                                    break 'outer;
                                                }
                                            }
                                        }
                                        result
                                    }
                                    Err(e) => {
                                        logger(format!("FindColor: Capture error - {}", e));
                                        (0, 0, false)
                                    }
                                }
                            } else {
                                (0, 0, false)
                            }
                        }
                        Err(e) => {
                            logger(format!("FindColor: Monitor error - {}", e));
                            (0, 0, false)
                        }
                    };

                    logger(format!(
                        "FindColor: Found={} at ({},{})",
                        found, found_x, found_y
                    ));

                    {
                        let mut ctx = context.lock().unwrap();
                        let node_id_str = current_node_id.to_string();
                        ctx.variables.insert(
                            format!("__out_{}_X", node_id_str),
                            VariableValue::Integer(found_x),
                        );
                        ctx.variables.insert(
                            format!("__out_{}_Y", node_id_str),
                            VariableValue::Integer(found_y),
                        );
                        ctx.variables.insert(
                            format!("__out_{}_Found", node_id_str),
                            VariableValue::Boolean(found),
                        );
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::WaitForColor => {
                    let target_r = Self::evaluate_input(&graph, current_node_id, "R", &context)
                        .map(|v| Self::to_float(&v) as u8)
                        .unwrap_or(255);
                    let target_g = Self::evaluate_input(&graph, current_node_id, "G", &context)
                        .map(|v| Self::to_float(&v) as u8)
                        .unwrap_or(0);
                    let target_b = Self::evaluate_input(&graph, current_node_id, "B", &context)
                        .map(|v| Self::to_float(&v) as u8)
                        .unwrap_or(0);
                    let x = Self::evaluate_input(&graph, current_node_id, "X", &context)
                        .map(|v| Self::to_float(&v) as u32)
                        .unwrap_or(0);
                    let y = Self::evaluate_input(&graph, current_node_id, "Y", &context)
                        .map(|v| Self::to_float(&v) as u32)
                        .unwrap_or(0);
                    let tolerance =
                        Self::evaluate_input(&graph, current_node_id, "Tolerance", &context)
                            .map(|v| Self::to_float(&v) as i32)
                            .unwrap_or(10);
                    let timeout_ms =
                        Self::evaluate_input(&graph, current_node_id, "Timeout", &context)
                            .map(|v| Self::to_float(&v) as u64)
                            .unwrap_or(5000);

                    logger(format!(
                        "WaitForColor: RGB({},{},{}) at ({},{}) tolerance={} timeout={}ms",
                        target_r, target_g, target_b, x, y, tolerance, timeout_ms
                    ));

                    let start = std::time::Instant::now();
                    let mut found = false;

                    while start.elapsed().as_millis() < timeout_ms as u128 {
                        if let Ok(monitors) = xcap::Monitor::all() {
                            if let Some(monitor) = monitors.first() {
                                if let Ok(img) = monitor.capture_image() {
                                    if x < img.width() && y < img.height() {
                                        let pixel = img.get_pixel(x, y);
                                        let dr = (pixel[0] as i32 - target_r as i32).abs();
                                        let dg = (pixel[1] as i32 - target_g as i32).abs();
                                        let db = (pixel[2] as i32 - target_b as i32).abs();
                                        if dr <= tolerance && dg <= tolerance && db <= tolerance {
                                            found = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                        thread::sleep(Duration::from_millis(100)); // Poll every 100ms
                    }

                    logger(format!("WaitForColor: Found={}", found));

                    {
                        let mut ctx = context.lock().unwrap();
                        let node_id_str = current_node_id.to_string();
                        ctx.variables.insert(
                            format!("__out_{}_Found", node_id_str),
                            VariableValue::Boolean(found),
                        );
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                NodeType::FindImage => {
                    let image_path =
                        Self::evaluate_input(&graph, current_node_id, "ImagePath", &context)
                            .map(|v| Self::to_string(&v))
                            .unwrap_or_default();
                    let tolerance =
                        Self::evaluate_input(&graph, current_node_id, "Tolerance", &context)
                            .map(|v| Self::to_float(&v) as i32)
                            .unwrap_or(10);
                    let region_x =
                        Self::evaluate_input(&graph, current_node_id, "RegionX", &context)
                            .map(|v| Self::to_float(&v) as u32)
                            .unwrap_or(0);
                    let region_y =
                        Self::evaluate_input(&graph, current_node_id, "RegionY", &context)
                            .map(|v| Self::to_float(&v) as u32)
                            .unwrap_or(0);
                    let region_w =
                        Self::evaluate_input(&graph, current_node_id, "RegionW", &context)
                            .map(|v| Self::to_float(&v) as u32)
                            .unwrap_or(1920);
                    let region_h =
                        Self::evaluate_input(&graph, current_node_id, "RegionH", &context)
                            .map(|v| Self::to_float(&v) as u32)
                            .unwrap_or(1080);

                    logger(format!(
                        "FindImage: {} tolerance={} in region ({},{})x{}x{}",
                        image_path, tolerance, region_x, region_y, region_w, region_h
                    ));

                    // Check if file exists and show absolute path for debugging
                    let abs_path = std::path::Path::new(&image_path)
                        .canonicalize()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|_| format!("(not found: {})", image_path));
                    logger(format!("FindImage: Resolved path: {}", abs_path));

                    let (found_x, found_y, found) = match image::open(&image_path) {
                        Ok(template) => {
                            let mut template = template.to_rgba8();
                            logger(format!(
                                "FindImage: Template loaded {}x{} pixels",
                                template.width(), template.height()
                            ));
                            match xcap::Monitor::all() {
                                Ok(monitors) => {
                                    if let Some(monitor) = monitors.first() {
                                        match monitor.capture_image() {
                                            Ok(screen) => {
                                                logger(format!(
                                                    "FindImage: Screen captured {}x{} pixels",
                                                    screen.width(), screen.height()
                                                ));
                                                
                                                logger(format!("FindImage: Starting template matching..."));
                                                let start_time = std::time::Instant::now();
                                                let result = Self::find_template_in_image(
                                                    &screen, &template, tolerance, region_x, region_y,
                                                    region_w, region_h,
                                                );
                                                logger(format!(
                                                    "FindImage: Template matching took {:.2}s",
                                                    start_time.elapsed().as_secs_f64()
                                                ));
                                                result
                                            }

                                            Err(e) => {
                                                logger(format!("FindImage: Capture error - {}", e));
                                                (0, 0, false)
                                            }
                                        }
                                    } else {
                                        (0, 0, false)
                                    }
                                }
                                Err(e) => {
                                    logger(format!("FindImage: Monitor error - {}", e));
                                    (0, 0, false)
                                }
                            }
                        }
                        Err(e) => {
                            logger(format!("FindImage: Template load error - {} ({})", e, image_path));
                            (0, 0, false)
                        }
                    };

                    logger(format!(
                        "FindImage: Found={} at ({},{})",
                        found, found_x, found_y
                    ));

                    {
                        let mut ctx = context.lock().unwrap();
                        let node_id_str = current_node_id.to_string();
                        ctx.variables.insert(
                            format!("__out_{}_X", node_id_str),
                            VariableValue::Integer(found_x),
                        );
                        ctx.variables.insert(
                            format!("__out_{}_Y", node_id_str),
                            VariableValue::Integer(found_y),
                        );
                        ctx.variables.insert(
                            format!("__out_{}_Found", node_id_str),
                            VariableValue::Boolean(found),
                        );
                    }

                    logger(format!("FindImage: Execution complete, looking for Next connection..."));
                    
                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        logger(format!("FindImage: Following flow to next node"));
                        current_node_id = next;
                    } else {
                        logger(format!("FindImage: No Next connection found, ending flow"));
                        break;
                    }
                }

                NodeType::WaitForImage => {
                    let image_path =
                        Self::evaluate_input(&graph, current_node_id, "ImagePath", &context)
                            .map(|v| Self::to_string(&v))
                            .unwrap_or_default();
                    let tolerance =
                        Self::evaluate_input(&graph, current_node_id, "Tolerance", &context)
                            .map(|v| Self::to_float(&v) as i32)
                            .unwrap_or(10);
                    let timeout_ms =
                        Self::evaluate_input(&graph, current_node_id, "Timeout", &context)
                            .map(|v| Self::to_float(&v) as u64)
                            .unwrap_or(5000);

                    logger(format!(
                        "WaitForImage: {} tolerance={} timeout={}ms",
                        image_path, tolerance, timeout_ms
                    ));

                    let (found_x, found_y, found) = match image::open(&image_path) {
                        Ok(template) => {
                            let template = template.to_rgba8();
                            let start = std::time::Instant::now();
                            let mut result = (0i64, 0i64, false);

                            while start.elapsed().as_millis() < timeout_ms as u128 {
                                if let Ok(monitors) = xcap::Monitor::all() {
                                    if let Some(monitor) = monitors.first() {
                                        if let Ok(screen) = monitor.capture_image() {
                                            let (fx, fy, f) = Self::find_template_in_image(
                                                &screen,
                                                &template,
                                                tolerance,
                                                0,
                                                0,
                                                screen.width(),
                                                screen.height(),
                                            );
                                            if f {
                                                result = (fx, fy, true);
                                                break;
                                            }
                                        }
                                    }
                                }
                                thread::sleep(Duration::from_millis(200)); // Poll every 200ms
                            }
                            result
                        }
                        Err(e) => {
                            logger(format!("WaitForImage: Template load error - {}", e));
                            (0, 0, false)
                        }
                    };

                    logger(format!(
                        "WaitForImage: Found={} at ({},{})",
                        found, found_x, found_y
                    ));

                    {
                        let mut ctx = context.lock().unwrap();
                        let node_id_str = current_node_id.to_string();
                        ctx.variables.insert(
                            format!("__out_{}_X", node_id_str),
                            VariableValue::Integer(found_x),
                        );
                        ctx.variables.insert(
                            format!("__out_{}_Y", node_id_str),
                            VariableValue::Integer(found_y),
                        );
                        ctx.variables.insert(
                            format!("__out_{}_Found", node_id_str),
                            VariableValue::Boolean(found),
                        );
                    }

                    if let Some(next) = Self::follow_flow(&graph, current_node_id, "Next") {
                        current_node_id = next;
                    } else {
                        break;
                    }
                }

                _ => break,
            }
            steps += 1;
        }
        if steps >= max_steps {
            logger("Execution stopped: Step limit reached.".to_string());
        }
    }

    fn follow_flow(graph: &BlueprintGraph, node_id: Uuid, port_name: &str) -> Option<Uuid> {
        for conn in &graph.connections {
            if conn.from_node == node_id && conn.from_port == port_name {
                return Some(conn.to_node);
            }
        }
        None
    }

    /// Helper for executing nested flows (like loop bodies)
    fn execute_flow_from(
        graph: Arc<BlueprintGraph>,
        start_id: Uuid,
        context: Arc<Mutex<ExecutionContext>>,
        tx: Sender<String>,
        mut steps: usize,
        max_steps: usize,
    ) {
        let logger = |msg: String| {
            let _ = tx.send(msg);
        };
        let mut current = start_id;

        while steps < max_steps {
            let node = match graph.nodes.get(&current) {
                Some(n) => n,
                None => break,
            };

            match &node.node_type {
                NodeType::BlueprintFunction { name } if name == "Print String" => {
                    if let Ok(val) = Self::evaluate_input(&graph, current, "String", &context) {
                        let output = match val {
                            VariableValue::String(s) => s,
                            other => format!("{:?}", other),
                        };
                        logger(format!("PRINT [Loop]: {}", output));
                    }
                    if let Some(next) = Self::follow_flow(&graph, current, "Next") {
                        current = next;
                    } else {
                        break;
                    }
                }
                NodeType::SetVariable { name } => {
                    if let Ok(val) = Self::evaluate_input(&graph, current, "Value", &context) {
                        context.lock().unwrap().variables.insert(name.clone(), val);
                    }
                    if let Some(next) = Self::follow_flow(&graph, current, "Next") {
                        current = next;
                    } else {
                        break;
                    }
                }
                NodeType::Delay => {
                    let ms = match Self::evaluate_input(&graph, current, "Duration (ms)", &context)
                    {
                        Ok(VariableValue::Integer(ms)) => ms as u64,
                        _ => 100,
                    };
                    thread::sleep(std::time::Duration::from_millis(ms));
                    if let Some(next) = Self::follow_flow(&graph, current, "Next") {
                        current = next;
                    } else {
                        break;
                    }
                }
                _ => break,
            }
            steps += 1;
        }
    }

    fn evaluate_input(
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
                return Self::evaluate_node(graph, from_node, &conn.from_port, context);
            }
        }

        if let Some(node) = graph.nodes.get(&node_id) {
            if let Some(port) = node.inputs.iter().find(|p| p.name == port_name) {
                return Ok(port.default_value.clone());
            }
        }

        Ok(VariableValue::None)
    }

    fn evaluate_node(
        graph: &BlueprintGraph,
        node: &Node,
        _output_port: &str,
        context: &Arc<Mutex<ExecutionContext>>,
    ) -> anyhow::Result<VariableValue> {
        match &node.node_type {
            NodeType::Add => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                Self::compute_math(a, b, |a, b| a + b, |a, b| a + b)
            }
            NodeType::Subtract => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                Self::compute_math(a, b, |a, b| a - b, |a, b| a - b)
            }
            NodeType::Multiply => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                Self::compute_math(a, b, |a, b| a * b, |a, b| a * b)
            }
            NodeType::Divide => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let mut b = Self::evaluate_input(graph, node.id, "B", context)?;
                // Divide by zero protection: if B is 0, use 1 instead
                let is_zero = match &b {
                    VariableValue::Float(f) => *f == 0.0,
                    VariableValue::Integer(i) => *i == 0,
                    _ => false,
                };
                if is_zero {
                    // LOG: Division by zero warning would be here if we had tx access
                    // For now, silently replace with 1
                    b = match &b {
                        VariableValue::Float(_) => VariableValue::Float(1.0),
                        VariableValue::Integer(_) => VariableValue::Integer(1),
                        _ => VariableValue::Integer(1),
                    };
                }
                Self::compute_math(a, b, |a, b| a / b, |a, b| a / b)
            }
            NodeType::GetVariable { name } => {
                let ctx = context.lock().unwrap();
                ctx.variables
                    .get(name)
                    .cloned()
                    .ok_or_else(|| anyhow::anyhow!("Variable not found: {}", name))
            }
            NodeType::ToInteger => {
                let input = Self::evaluate_input(graph, node.id, "In", context)?;
                match input {
                    VariableValue::Integer(i) => Ok(VariableValue::Integer(i)),
                    VariableValue::Float(f) => Ok(VariableValue::Integer(f as i64)),
                    VariableValue::String(s) => Ok(VariableValue::Integer(s.parse().unwrap_or(0))),
                    VariableValue::Boolean(b) => Ok(VariableValue::Integer(if b { 1 } else { 0 })),
                    _ => Ok(VariableValue::Integer(0)),
                }
            }
            NodeType::ToFloat => {
                let input = Self::evaluate_input(graph, node.id, "In", context)?;
                match input {
                    VariableValue::Float(f) => Ok(VariableValue::Float(f)),
                    VariableValue::Integer(i) => Ok(VariableValue::Float(i as f64)),
                    VariableValue::String(s) => Ok(VariableValue::Float(s.parse().unwrap_or(0.0))),
                    VariableValue::Boolean(b) => {
                        Ok(VariableValue::Float(if b { 1.0 } else { 0.0 }))
                    }
                    _ => Ok(VariableValue::Float(0.0)),
                }
            }
            NodeType::ToString => {
                let input = Self::evaluate_input(graph, node.id, "In", context)?;
                let s = match input {
                    VariableValue::String(s) => s,
                    VariableValue::Integer(i) => i.to_string(),
                    VariableValue::Float(f) => f.to_string(),
                    VariableValue::Boolean(b) => b.to_string(),
                    VariableValue::Vector3(x, y, z) => format!("({}, {}, {})", x, y, z),
                    VariableValue::Array(arr) => Self::to_string(&VariableValue::Array(arr)),
                    VariableValue::None => "None".to_string(),
                };
                Ok(VariableValue::String(s))
            }
            // Comparison operations
            NodeType::Equals => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                Ok(VariableValue::Boolean(
                    Self::compare_values(&a, &b) == std::cmp::Ordering::Equal,
                ))
            }
            NodeType::NotEquals => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                Ok(VariableValue::Boolean(
                    Self::compare_values(&a, &b) != std::cmp::Ordering::Equal,
                ))
            }
            NodeType::GreaterThan => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                Ok(VariableValue::Boolean(
                    Self::compare_values(&a, &b) == std::cmp::Ordering::Greater,
                ))
            }
            NodeType::GreaterThanOrEqual => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                let cmp = Self::compare_values(&a, &b);
                Ok(VariableValue::Boolean(
                    cmp == std::cmp::Ordering::Greater || cmp == std::cmp::Ordering::Equal,
                ))
            }
            NodeType::LessThan => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                Ok(VariableValue::Boolean(
                    Self::compare_values(&a, &b) == std::cmp::Ordering::Less,
                ))
            }
            NodeType::LessThanOrEqual => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                let cmp = Self::compare_values(&a, &b);
                Ok(VariableValue::Boolean(
                    cmp == std::cmp::Ordering::Less || cmp == std::cmp::Ordering::Equal,
                ))
            }
            // Logic operations
            NodeType::And => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                Ok(VariableValue::Boolean(
                    Self::to_bool(&a) && Self::to_bool(&b),
                ))
            }
            NodeType::Or => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                Ok(VariableValue::Boolean(
                    Self::to_bool(&a) || Self::to_bool(&b),
                ))
            }
            NodeType::Not => {
                let input = Self::evaluate_input(graph, node.id, "In", context)?;
                Ok(VariableValue::Boolean(!Self::to_bool(&input)))
            }
            // ForLoop - returns the current loop index from context
            NodeType::ForLoop => {
                let ctx = context.lock().unwrap();
                Ok(ctx
                    .variables
                    .get("__loop_index")
                    .cloned()
                    .unwrap_or(VariableValue::Integer(0)))
            }
            // Xor
            NodeType::Xor => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                Ok(VariableValue::Boolean(
                    Self::to_bool(&a) ^ Self::to_bool(&b),
                ))
            }
            // Modulo
            NodeType::Modulo => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
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
            // Power
            NodeType::Power => {
                let base = Self::evaluate_input(graph, node.id, "Base", context)?;
                let exp = Self::evaluate_input(graph, node.id, "Exponent", context)?;
                let base_f = Self::to_float(&base);
                let exp_f = Self::to_float(&exp);
                Ok(VariableValue::Float(base_f.powf(exp_f)))
            }
            // Abs
            NodeType::Abs => {
                let input = Self::evaluate_input(graph, node.id, "In", context)?;
                match input {
                    VariableValue::Float(f) => Ok(VariableValue::Float(f.abs())),
                    VariableValue::Integer(i) => Ok(VariableValue::Integer(i.abs())),
                    _ => Ok(VariableValue::Float(0.0)),
                }
            }
            // Min
            NodeType::Min => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                let af = Self::to_float(&a);
                let bf = Self::to_float(&b);
                Ok(VariableValue::Float(af.min(bf)))
            }
            // Max
            NodeType::Max => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                let af = Self::to_float(&a);
                let bf = Self::to_float(&b);
                Ok(VariableValue::Float(af.max(bf)))
            }
            // Clamp
            NodeType::Clamp => {
                let value = Self::evaluate_input(graph, node.id, "Value", context)?;
                let min = Self::evaluate_input(graph, node.id, "Min", context)?;
                let max = Self::evaluate_input(graph, node.id, "Max", context)?;
                let vf = Self::to_float(&value);
                let minf = Self::to_float(&min);
                let maxf = Self::to_float(&max);
                Ok(VariableValue::Float(vf.clamp(minf, maxf)))
            }
            // Random
            NodeType::Random => {
                let min = Self::evaluate_input(graph, node.id, "Min", context)?;
                let max = Self::evaluate_input(graph, node.id, "Max", context)?;
                let minf = Self::to_float(&min);
                let maxf = Self::to_float(&max);
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
            // Concat
            NodeType::Concat => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                let sa = Self::to_string(&a);
                let sb = Self::to_string(&b);
                Ok(VariableValue::String(format!("{}{}", sa, sb)))
            }
            // Split
            NodeType::Split => {
                let input = Self::evaluate_input(graph, node.id, "String", context)?;
                let delim = Self::evaluate_input(graph, node.id, "Delimiter", context)?;
                let index = Self::evaluate_input(graph, node.id, "Index", context)?;
                let s = Self::to_string(&input);
                let d = Self::to_string(&delim);
                let idx = match index {
                    VariableValue::Integer(i) => i as usize,
                    _ => 0,
                };
                let parts: Vec<&str> = s.split(&d).collect();
                let result = parts.get(idx).unwrap_or(&"").to_string();
                Ok(VariableValue::String(result))
            }
            // Length
            NodeType::Length => {
                let input = Self::evaluate_input(graph, node.id, "String", context)?;
                let s = Self::to_string(&input);
                Ok(VariableValue::Integer(s.len() as i64))
            }
            // Contains
            NodeType::Contains => {
                let input = Self::evaluate_input(graph, node.id, "String", context)?;
                let sub = Self::evaluate_input(graph, node.id, "Substring", context)?;
                let s = Self::to_string(&input);
                let sub_s = Self::to_string(&sub);
                Ok(VariableValue::Boolean(s.contains(&sub_s)))
            }
            // Replace
            NodeType::Replace => {
                let input = Self::evaluate_input(graph, node.id, "String", context)?;
                let from = Self::evaluate_input(graph, node.id, "From", context)?;
                let to = Self::evaluate_input(graph, node.id, "To", context)?;
                let s = Self::to_string(&input);
                let from_s = Self::to_string(&from);
                let to_s = Self::to_string(&to);
                Ok(VariableValue::String(s.replace(&from_s, &to_s)))
            }
            // Format
            NodeType::Format => {
                let template = Self::evaluate_input(graph, node.id, "Template", context)?;
                let arg0 = Self::evaluate_input(graph, node.id, "Arg0", context)?;
                let t = Self::to_string(&template);
                let a = Self::to_string(&arg0);
                // Simple {} replacement
                Ok(VariableValue::String(t.replacen("{}", &a, 1)))
            }
            // StringJoin - Dynamic string concatenation with auto-expanding inputs
            NodeType::StringJoin => {
                let mut result = String::new();
                let mut idx = 0;
                loop {
                    let port_name = format!("Input {}", idx);
                    match Self::evaluate_input(graph, node.id, &port_name, context) {
                        Ok(val) if !matches!(val, VariableValue::None) => {
                            result.push_str(&Self::to_string(&val));
                            idx += 1;
                        }
                        _ => break,
                    }
                }
                Ok(VariableValue::String(result))
            }
            // StringBetween - Extract content between two delimiter strings
            NodeType::StringBetween => {
                let source = Self::evaluate_input(graph, node.id, "Source", context)?;
                let before = Self::evaluate_input(graph, node.id, "Before", context)?;
                let after = Self::evaluate_input(graph, node.id, "After", context)?;

                let source_s = Self::to_string(&source);
                let before_s = Self::to_string(&before);
                let after_s = Self::to_string(&after);

                let result = if before_s.is_empty() && after_s.is_empty() {
                    source_s.clone()
                } else if before_s.is_empty() {
                    // From start to "after"
                    source_s.split(&after_s).next().unwrap_or("").to_string()
                } else if after_s.is_empty() {
                    // From "before" to end
                    match source_s.split_once(&before_s) {
                        Some((_, rest)) => rest.to_string(),
                        None => String::new(),
                    }
                } else {
                    // Between "before" and "after"
                    match source_s.split_once(&before_s) {
                        Some((_, rest)) => rest.split(&after_s).next().unwrap_or("").to_string(),
                        None => String::new(),
                    }
                };

                Ok(VariableValue::String(result))
            }
            // FileRead
            NodeType::FileRead => {
                let path = Self::evaluate_input(graph, node.id, "Path", context)?;
                let path_s = Self::to_string(&path);
                match std::fs::read_to_string(&path_s) {
                    Ok(content) => Ok(VariableValue::String(content)),
                    Err(_) => Ok(VariableValue::String("".into())),
                }
            }
            // === Module H: Data Operations ===

            // ArrayCreate - Creates an empty array
            NodeType::ArrayCreate => Ok(VariableValue::Array(Vec::new())),

            // ArrayGet - Get element at index from array
            NodeType::ArrayGet => {
                let array = Self::evaluate_input(graph, node.id, "Array", context)?;
                let index = Self::evaluate_input(graph, node.id, "Index", context)?;

                let idx = match index {
                    VariableValue::Integer(i) => i as usize,
                    VariableValue::Float(f) => f as usize,
                    _ => 0,
                };

                match array {
                    VariableValue::Array(arr) => {
                        Ok(arr.get(idx).cloned().unwrap_or(VariableValue::None))
                    }
                    // If input is a String, treat it as character array access
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

            // ArrayLength - Get the length of an array
            NodeType::ArrayLength => {
                let array = Self::evaluate_input(graph, node.id, "Array", context)?;
                match array {
                    VariableValue::Array(arr) => Ok(VariableValue::Integer(arr.len() as i64)),
                    VariableValue::String(s) => Ok(VariableValue::Integer(s.len() as i64)),
                    _ => Ok(VariableValue::Integer(0)),
                }
            }

            // JSONParse - Parse JSON string into VariableValue
            NodeType::JSONParse => {
                let input = Self::evaluate_input(graph, node.id, "JSON", context)?;
                let json_str = Self::to_string(&input);

                match serde_json::from_str::<serde_json::Value>(&json_str) {
                    Ok(value) => Ok(Self::json_to_variable_value(&value)),
                    Err(_) => Ok(VariableValue::None),
                }
            }

            // JSONStringify - Convert VariableValue to JSON string
            NodeType::JSONStringify => {
                let input = Self::evaluate_input(graph, node.id, "Value", context)?;
                let json_value = Self::variable_value_to_json(&input);
                Ok(VariableValue::String(json_value.to_string()))
            }

            // === Module D: Image Recognition (Pure Functions) ===

            // ImageSimilarity - Compare two images and return similarity score
            NodeType::ImageSimilarity => {
                let path1 = Self::evaluate_input(graph, node.id, "ImagePath1", context)
                    .map(|v| Self::to_string(&v))
                    .unwrap_or_default();
                let path2 = Self::evaluate_input(graph, node.id, "ImagePath2", context)
                    .map(|v| Self::to_string(&v))
                    .unwrap_or_default();
                let tolerance = Self::evaluate_input(graph, node.id, "Tolerance", context)
                    .map(|v| Self::to_float(&v) as i32)
                    .unwrap_or(10);

                let similarity = match (image::open(&path1), image::open(&path2)) {
                    (Ok(img1), Ok(img2)) => {
                        let img1 = img1.to_rgba8();
                        let img2 = img2.to_rgba8();
                        Self::compare_images(&img1, &img2, tolerance)
                    }
                    _ => 0.0,
                };

                // Return based on output port requested
                let port = _output_port;
                if port == "Similarity" {
                    Ok(VariableValue::Float(similarity))
                } else if port == "Match" {
                    Ok(VariableValue::Boolean(similarity >= 0.95))
                } else {
                    Ok(VariableValue::Float(similarity))
                }
            }

            // GetWindowPosition (Impure-like data node with cached results)
            NodeType::GetWindowPosition => {
                let output_port = _output_port;
                let cache_key = format!("__winpos_{}", node.id);

                // Check if we have cached results
                let (x, y, w, h, found) = {
                    let ctx = context.lock().unwrap();
                    if let Some(VariableValue::String(cached)) = ctx.variables.get(&cache_key) {
                        // Parse cached: "x,y,w,h,found"
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
                        drop(ctx); // Release lock before running command

                        // Get window title input
                        let title = Self::evaluate_input(graph, node.id, "Title", context)
                            .map(|v| Self::to_string(&v))
                            .unwrap_or_default();

                        // Platform-specific implementation
                        #[cfg(target_os = "macos")]
                        let (x, y, w, h, found) = {
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
                        };

                        #[cfg(target_os = "linux")]
                        let (x, y, w, h, found) = {
                            // Linux: Use xdotool to get window geometry
                            let id_result = std::process::Command::new("xdotool")
                                .args(["search", "--name", &title])
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
                                        (x, y, w, h, true)
                                    } else {
                                        (0, 0, 1920, 1080, false)
                                    }
                                } else {
                                    (0, 0, 1920, 1080, false)
                                }
                            } else {
                                (0, 0, 1920, 1080, false)
                            }
                        };

                        #[cfg(target_os = "windows")]
                        let (x, y, w, h, found) = (0i64, 0i64, 1920i64, 1080i64, true); // Windows stub

                        #[cfg(not(any(
                            target_os = "macos",
                            target_os = "linux",
                            target_os = "windows"
                        )))]
                        let (x, y, w, h, found) = (0i64, 0i64, 1920i64, 1080i64, false);

                        // Cache the result
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

            // System Control & Image Recognition Outputs (retrieved from context storage)
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
            | NodeType::SaveScreenshot
            | NodeType::RegionCapture
            | NodeType::HTTPRequest
            | NodeType::ArrayPop => {
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

    fn to_bool(val: &VariableValue) -> bool {
        helpers::to_bool(val)
    }

    fn to_float(val: &VariableValue) -> f64 {
        helpers::to_float(val)
    }

    fn to_string(val: &VariableValue) -> String {
        helpers::to_string(val)
    }

    fn compare_values(a: &VariableValue, b: &VariableValue) -> std::cmp::Ordering {
        helpers::compare_values(a, b)
    }

    fn compute_math(
        a: VariableValue,
        b: VariableValue,
        op_f: fn(f64, f64) -> f64,
        op_i: fn(i64, i64) -> i64,
    ) -> anyhow::Result<VariableValue> {
        helpers::compute_math(a, b, op_f, op_i)
    }

    fn json_to_variable_value(value: &serde_json::Value) -> VariableValue {
        json_helpers::json_to_variable_value(value)
    }

    fn variable_value_to_json(value: &VariableValue) -> serde_json::Value {
        json_helpers::variable_value_to_json(value)
    }

    fn string_to_key(key_str: &str) -> Option<Key> {
        helpers::string_to_key(key_str)
    }

    fn find_template_in_image(
        screen: &image::RgbaImage,
        template: &image::RgbaImage,
        tolerance: i32,
        region_x: u32,
        region_y: u32,
        region_w: u32,
        region_h: u32,
    ) -> (i64, i64, bool) {
        image_matching::find_template_in_image(screen, template, tolerance, region_x, region_y, region_w, region_h)
    }

    fn compare_images(img1: &image::RgbaImage, img2: &image::RgbaImage, tolerance: i32) -> f64 {
        image_matching::compare_images(img1, img2, tolerance)
    }
}

pub mod test_drag;
pub mod test_drag_verification;
pub mod test_find_image;
