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
    /// Z-order for rendering: higher values are drawn on top.
    #[serde(default)]
    pub z_order: u64,
    /// Custom display name for the node.
    #[serde(default)]
    pub display_name: Option<String>,
    /// Whether this node is enabled for execution.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Group ID for visual grouping of nodes.
    #[serde(default)]
    pub group_id: Option<Uuid>,
    /// Note text content (for Notes node type only)
    #[serde(default)]
    pub note_text: String,
    /// Note size (width, height) for Notes node
    #[serde(default = "default_note_size")]
    pub note_size: (f32, f32),
}

fn default_note_size() -> (f32, f32) {
    (200.0, 100.0)
}

fn default_enabled() -> bool {
    true
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
    /// Array of values (for Module H: Data Operations)
    Array(Vec<VariableValue>),
    None, 
}
