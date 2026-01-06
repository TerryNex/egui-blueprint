use crate::graph::{BlueprintGraph, Node, VariableValue};
use crate::node_types::NodeType;
use std::collections::HashMap;
use uuid::Uuid;

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
    pub fn run(graph: &BlueprintGraph, mut logger: impl FnMut(String)) -> anyhow::Result<()> {
        logger("Interpreter started.".to_string());
        
        let mut start_node_id = None;
        for node in graph.nodes.values() {
             if let NodeType::BlueprintFunction { name } = &node.node_type {
                 if name == "Event Tick" {
                     start_node_id = Some(node.id);
                     break;
                 }
             }
        }

        if let Some(mut current_node_id) = start_node_id {
             logger(format!("Execution starting from Event Tick ({:?})", current_node_id));
             let mut steps = 0;
             while steps < 1000 { // Simple infinite loop protection
                 let node = match graph.nodes.get(&current_node_id) {
                     Some(n) => n,
                     None => break,
                 };
                 
                 match &node.node_type {
                     NodeType::BlueprintFunction { name } => {
                         if name == "Event Tick" {
                              if let Some(next) = Self::follow_flow(graph, current_node_id, "Next") {
                                  current_node_id = next;
                              } else {
                                  break;
                              }
                         } else if name == "Print String" {
                             let val = Self::evaluate_input(graph, current_node_id, "String")?;
                             let output = match val {
                                 VariableValue::String(s) => s,
                                 other => format!("{:?}", other),
                             };
                             logger(format!("PRINT: {}", output));
                             
                             if let Some(next) = Self::follow_flow(graph, current_node_id, "Next") {
                                  current_node_id = next;
                              } else {
                                  break;
                              }
                         } else {
                             break;
                         }
                     }
                     _ => break,
                 }
                 steps += 1;
             }
             if steps >= 1000 {
                 logger("Execution stopped: Step limit reached (possible infinite loop).".to_string());
             }
        } else {
            logger("No 'Event Tick' node found. Execution aborted.".to_string());
        }
        
        Ok(())
    }
    
    fn follow_flow(graph: &BlueprintGraph, node_id: Uuid, port_name: &str) -> Option<Uuid> {
        for conn in &graph.connections {
            if conn.from_node == node_id && conn.from_port == port_name {
                return Some(conn.to_node);
            }
        }
        None
    }
    
    fn evaluate_input(graph: &BlueprintGraph, node_id: Uuid, port_name: &str) -> anyhow::Result<VariableValue> {
        for conn in &graph.connections {
            if conn.to_node == node_id && conn.to_port == port_name {
                let from_node = graph.nodes.get(&conn.from_node).ok_or_else(|| anyhow::anyhow!("Source node not found"))?;
                return Self::evaluate_node(graph, from_node, &conn.from_port);
            }
        }
        
        // If disconnected, use default value from the input port
        if let Some(node) = graph.nodes.get(&node_id) {
            if let Some(port) = node.inputs.iter().find(|p| p.name == port_name) {
                return Ok(port.default_value.clone());
            }
        }

        Ok(VariableValue::None)
    }
    
    fn evaluate_node(graph: &BlueprintGraph, node: &Node, _output_port: &str) -> anyhow::Result<VariableValue> {
        match &node.node_type {
            NodeType::Add => {
                let a = Self::evaluate_input(graph, node.id, "A")?;
                let b = Self::evaluate_input(graph, node.id, "B")?;
                match (a, b) {
                    (VariableValue::Float(av), VariableValue::Float(bv)) => Ok(VariableValue::Float(av + bv)),
                    (VariableValue::Integer(av), VariableValue::Integer(bv)) => Ok(VariableValue::Integer(av + bv)),
                    _ => Ok(VariableValue::None),
                }
            }
            NodeType::Subtract => {
                let a = Self::evaluate_input(graph, node.id, "A")?;
                let b = Self::evaluate_input(graph, node.id, "B")?;
                match (a, b) {
                    (VariableValue::Float(av), VariableValue::Float(bv)) => Ok(VariableValue::Float(av - bv)),
                    (VariableValue::Integer(av), VariableValue::Integer(bv)) => Ok(VariableValue::Integer(av - bv)),
                    _ => Ok(VariableValue::None),
                }
            }
            _ => Ok(VariableValue::None)
        }
    }
}
