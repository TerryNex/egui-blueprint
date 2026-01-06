use crate::graph::{BlueprintGraph, Node, VariableValue};
use crate::node_types::NodeType;
use std::collections::HashMap;
use uuid::Uuid;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;

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
        tx_main.send("Interpreter started (Async).".to_string()).unwrap_or_default();
        
        // Clone graph and create shared context
        let graph = Arc::new(graph.clone());
        let context = Arc::new(Mutex::new(ExecutionContext::new()));
        
        // Initialize variables
        {
            let mut ctx = context.lock().unwrap();
            for (name, var) in &graph.variables {
                ctx.variables.insert(name.clone(), var.initial_value.clone());
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
            tx_main.send("No 'Event Tick' node found. Execution aborted.".to_string()).unwrap_or_default();
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
    
    fn execute_flow(graph: Arc<BlueprintGraph>, start_id: Uuid, context: Arc<Mutex<ExecutionContext>>, tx: Sender<String>) {
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

        while steps < max_steps {
             let node = match graph.nodes.get(&current_node_id) {
                 Some(n) => n,
                 None => break,
             };
             
             match &node.node_type {
                 NodeType::SetVariable { name } => {
                     // Check input
                     if let Ok(val) = Self::evaluate_input(&graph, current_node_id, "Value", &context) {
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
                          if let Ok(val) = Self::evaluate_input(&graph, current_node_id, "String", &context) {
                              let output = match val {
                                 VariableValue::String(s) => s,
                                 other => format!("{:?}", other),
                             };
                             logger(format!("PRINT [Thread {:?}]: {}", thread::current().id(), output));
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
                      let condition = Self::evaluate_input(&graph, current_node_id, "Condition", &context)
                           .unwrap_or(VariableValue::Boolean(false));
                           
                      let bool_val = match condition {
                          VariableValue::Boolean(b) => b,
                          _ => false,
                      };
                      
                      let port = if bool_val { "True" } else { "False" };
                      if let Some(next) = Self::follow_flow(&graph, current_node_id, port) {
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
    
    fn evaluate_input(graph: &BlueprintGraph, node_id: Uuid, port_name: &str, context: &Arc<Mutex<ExecutionContext>>) -> anyhow::Result<VariableValue> {
        for conn in &graph.connections {
            if conn.to_node == node_id && conn.to_port == port_name {
                let from_node = graph.nodes.get(&conn.from_node).ok_or_else(|| anyhow::anyhow!("Source node not found"))?;
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
    
    fn evaluate_node(graph: &BlueprintGraph, node: &Node, _output_port: &str, context: &Arc<Mutex<ExecutionContext>>) -> anyhow::Result<VariableValue> {
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
                ctx.variables.get(name).cloned().ok_or_else(|| anyhow::anyhow!("Variable not found: {}", name))
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
                    VariableValue::Boolean(b) => Ok(VariableValue::Float(if b { 1.0 } else { 0.0 })),
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
                    VariableValue::None => "None".to_string(),
                };
                Ok(VariableValue::String(s))
            }
            // Comparison operations
            NodeType::Equals => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                Ok(VariableValue::Boolean(Self::compare_values(&a, &b) == std::cmp::Ordering::Equal))
            }
            NodeType::NotEquals => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                Ok(VariableValue::Boolean(Self::compare_values(&a, &b) != std::cmp::Ordering::Equal))
            }
            NodeType::GreaterThan => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                Ok(VariableValue::Boolean(Self::compare_values(&a, &b) == std::cmp::Ordering::Greater))
            }
            NodeType::GreaterThanOrEqual => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                let cmp = Self::compare_values(&a, &b);
                Ok(VariableValue::Boolean(cmp == std::cmp::Ordering::Greater || cmp == std::cmp::Ordering::Equal))
            }
            NodeType::LessThan => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                Ok(VariableValue::Boolean(Self::compare_values(&a, &b) == std::cmp::Ordering::Less))
            }
            NodeType::LessThanOrEqual => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                let cmp = Self::compare_values(&a, &b);
                Ok(VariableValue::Boolean(cmp == std::cmp::Ordering::Less || cmp == std::cmp::Ordering::Equal))
            }
            // Logic operations
            NodeType::And => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                Ok(VariableValue::Boolean(Self::to_bool(&a) && Self::to_bool(&b)))
            }
            NodeType::Or => {
                let a = Self::evaluate_input(graph, node.id, "A", context)?;
                let b = Self::evaluate_input(graph, node.id, "B", context)?;
                Ok(VariableValue::Boolean(Self::to_bool(&a) || Self::to_bool(&b)))
            }
            NodeType::Not => {
                let input = Self::evaluate_input(graph, node.id, "In", context)?;
                Ok(VariableValue::Boolean(!Self::to_bool(&input)))
            }
            _ => Ok(VariableValue::None)
        }
    }
    
    fn to_bool(val: &VariableValue) -> bool {
        match val {
            VariableValue::Boolean(b) => *b,
            VariableValue::Integer(i) => *i > 0,
            VariableValue::Float(f) => *f > 0.0,
            VariableValue::String(s) => s.to_lowercase() == "true" || s == "1",
            _ => false,
        }
    }
    
    fn compare_values(a: &VariableValue, b: &VariableValue) -> std::cmp::Ordering {
        match (a, b) {
            (VariableValue::Float(av), VariableValue::Float(bv)) => av.partial_cmp(bv).unwrap_or(std::cmp::Ordering::Equal),
            (VariableValue::Integer(av), VariableValue::Integer(bv)) => av.cmp(bv),
            (VariableValue::Float(av), VariableValue::Integer(bv)) => av.partial_cmp(&(*bv as f64)).unwrap_or(std::cmp::Ordering::Equal),
            (VariableValue::Integer(av), VariableValue::Float(bv)) => (*av as f64).partial_cmp(bv).unwrap_or(std::cmp::Ordering::Equal),
            (VariableValue::String(av), VariableValue::String(bv)) => av.cmp(bv),
            (VariableValue::Boolean(av), VariableValue::Boolean(bv)) => av.cmp(bv),
            _ => std::cmp::Ordering::Equal,
        }
    }
    
    fn compute_math(a: VariableValue, b: VariableValue, op_f: fn(f64, f64) -> f64, op_i: fn(i64, i64) -> i64) -> anyhow::Result<VariableValue> {
         match (a, b) {
            (VariableValue::Float(av), VariableValue::Float(bv)) => Ok(VariableValue::Float(op_f(av, bv))),
            (VariableValue::Integer(av), VariableValue::Integer(bv)) => Ok(VariableValue::Integer(op_i(av, bv))),
            (VariableValue::Float(av), VariableValue::Integer(bv)) => Ok(VariableValue::Float(op_f(av, bv as f64))),
            (VariableValue::Integer(av), VariableValue::Float(bv)) => Ok(VariableValue::Float(op_f(av as f64, bv))),
            _ => Ok(VariableValue::None),
        }
    }
}
