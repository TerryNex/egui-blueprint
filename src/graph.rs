use super::node_types::{DataType, NodeType};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlueprintGraph {
    pub nodes: HashMap<Uuid, Node>,
    pub connections: Vec<Connection>,
    pub variables: HashMap<String, Variable>,
    /// Node groups for organizing nodes (UE5 BP-style grouping)
    #[serde(default)]
    pub groups: HashMap<Uuid, NodeGroup>,
}

/// A visual group that can contain multiple nodes.
/// Dragging the group title bar moves all contained nodes together.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeGroup {
    pub id: Uuid,
    pub name: String,
    pub position: (f32, f32),
    pub size: (f32, f32),
    /// Color for the group border/background (RGBA)
    pub color: [u8; 4],
    /// Node IDs contained in this group
    pub contained_nodes: Vec<Uuid>,
}

impl Default for BlueprintGraph {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            connections: Vec::new(),
            variables: HashMap::new(),
            groups: HashMap::new(),
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
    /// Z-order for rendering: higher values are drawn on top. Updated when node is clicked.
    #[serde(default)]
    pub z_order: u64,
    /// Custom display name for the node. If None, uses default type name.
    #[serde(default)]
    pub display_name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Port {
    pub name: String,
    pub data_type: DataType,
    pub default_value: VariableValue,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
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
