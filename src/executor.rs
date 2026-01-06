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
             log::info!("Found EventTick node: {:?}", current_node_id);
             loop {
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
                             // Evaluate Input "String"
                             let val = Self::evaluate_input(graph, current_node_id, "String")?;
                             // We should really have a proper value system.
                             log::info!("PRINT: {:?}", val);
                             
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
             }
        }
        
        Ok(())
    }
    
    fn follow_flow(graph: &BlueprintGraph, node_id: Uuid, port_name: &str) -> Option<Uuid> {
        // Find connection from node_id:port_name
        for conn in &graph.connections {
            if conn.from_node == node_id && conn.from_port == port_name {
                return Some(conn.to_node);
            }
        }
        None
    }
    
    fn evaluate_input(graph: &BlueprintGraph, node_id: Uuid, port_name: &str) -> anyhow::Result<VariableValue> {
        // Find connection to node_id:port_name (is_input)
        // logic: connection.to_node == node_id && connection.to_port == port_name
        for conn in &graph.connections {
            if conn.to_node == node_id && conn.to_port == port_name {
                let from_node = graph.nodes.get(&conn.from_node).unwrap();
                return Self::evaluate_node(graph, from_node, &conn.from_port);
            }
        }
        // Default value if disconnected?
        Ok(VariableValue::String("Default".into()))
    }
    
    fn evaluate_node(_graph: &BlueprintGraph, _node: &Node, _output_port: &str) -> anyhow::Result<VariableValue> {
        // Simple evaluation
        Ok(VariableValue::String("Evaluated Value".into()))
    }
}
