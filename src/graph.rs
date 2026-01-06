use super::node_types::{DataType, NodeType};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlueprintGraph {
    pub nodes: HashMap<Uuid, Node>,
    pub connections: Vec<Connection>,
    pub variables: HashMap<String, Variable>,
}

impl Default for BlueprintGraph {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
            variables: HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub node_type: NodeType,
    pub position: (f32, f32),
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Port {
    pub name: String,
    pub data_type: DataType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Connection {
    pub from_node: Uuid,
    pub from_port: String, // or index
    pub to_node: Uuid,
    pub to_port: String, // or index
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub data_type: DataType,
    pub initial_value: VariableValue,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VariableValue {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Vector3(f32, f32, f32),
    None, 
}
