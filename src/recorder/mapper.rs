use super::RecordedAction;
use crate::graph::{Node, Port, VariableValue};
use crate::node_types::{DataType, NodeType};
use rdev::{Button, Event, EventType};
use uuid::Uuid;

pub fn map_action_to_node(action: RecordedAction, position: (f32, f32)) -> Option<Node> {
    let (x, y) = action.cursor_position;

    match action.event.event_type {
        EventType::ButtonPress(button) => {
            // Default to MouseDown. Main loop logic will consolidate to Click if needed.
            let node_type = NodeType::MouseDown;
            let btn_str = match button {
                Button::Left => "Left",
                Button::Right => "Right",
                Button::Middle => "Middle",
                _ => "Left",
            };
            Some(create_mouse_btn_node(
                node_type,
                position,
                btn_str.to_string(),
                x as i64,
                y as i64,
            ))
        }
        EventType::ButtonRelease(button) => {
            let node_type = NodeType::MouseUp;
            let btn_str = match button {
                Button::Left => "Left",
                Button::Right => "Right",
                Button::Middle => "Middle",
                _ => "Left",
            };
            Some(create_mouse_btn_node(
                node_type,
                position,
                btn_str.to_string(),
                x as i64,
                y as i64,
            ))
        }
        EventType::KeyPress(key) => {
            // Map key to string.
            // event.name is Option<String>, key is valid enum.
            // We prefer event.name for readable chars, or key enum debug for others.
            let key_name = action
                .event
                .name
                .or_else(|| Some(format!("{:?}", key)))
                .unwrap();
            Some(create_key_node(NodeType::KeyPress, position, key_name))
        }
        // We explicitly ignore Scroll and MouseMove here as they are filtered in mod.rs or handled elsewhere
        _ => None,
    }
}

pub fn create_click_node(node_type: NodeType, position: (f32, f32), x: i64, y: i64) -> Node {
    let inputs = vec![
        Port {
            name: "In".to_string(),
            data_type: DataType::ExecutionFlow,
            default_value: VariableValue::None,
        },
        Port {
            name: "X".to_string(),
            data_type: DataType::Integer,
            default_value: VariableValue::Integer(x),
        },
        Port {
            name: "Y".to_string(),
            data_type: DataType::Integer,
            default_value: VariableValue::Integer(y),
        },
    ];

    let outputs = vec![Port {
        name: "Next".to_string(),
        data_type: DataType::ExecutionFlow,
        default_value: VariableValue::None,
    }];

    Node {
        id: Uuid::new_v4(),
        node_type,
        position,
        inputs,
        outputs,
        z_order: 0,
        display_name: None,
    }
}

fn create_key_node(node_type: NodeType, position: (f32, f32), key: String) -> Node {
    let inputs = vec![
        Port {
            name: "Exec In".to_string(),
            data_type: DataType::ExecutionFlow,
            default_value: VariableValue::None,
        },
        Port {
            name: "Key".to_string(),
            data_type: DataType::String,
            default_value: VariableValue::String(key),
        },
    ];

    let outputs = vec![Port {
        name: "Exec Out".to_string(),
        data_type: DataType::ExecutionFlow,
        default_value: VariableValue::None,
    }];

    Node {
        id: Uuid::new_v4(),
        node_type,
        position,
        inputs,
        outputs,
        z_order: 0,
        display_name: None,
    }
}

pub fn create_delay_node(position: (f32, f32), duration: f32) -> Node {
    let inputs = vec![
        Port {
            name: "In".to_string(),
            data_type: DataType::ExecutionFlow,
            default_value: VariableValue::None,
        },
        Port {
            name: "Duration".to_string(),
            data_type: DataType::Float,
            default_value: VariableValue::Float(duration as f64),
        },
    ];

    let outputs = vec![Port {
        name: "Next".to_string(),
        data_type: DataType::ExecutionFlow,
        default_value: VariableValue::None,
    }];

    Node {
        id: Uuid::new_v4(),
        node_type: NodeType::Delay,
        position,
        inputs,
        outputs,
        z_order: 0,
        display_name: None,
    }
}

pub fn create_mouse_btn_node(
    node_type: NodeType,
    position: (f32, f32),
    button: String,
    x: i64,
    y: i64,
) -> Node {
    let inputs = vec![
        Port {
            name: "In".to_string(),
            data_type: DataType::ExecutionFlow,
            default_value: VariableValue::None,
        },
        Port {
            name: "Button".to_string(),
            data_type: DataType::String,
            default_value: VariableValue::String(button),
        },
        Port {
            name: "X".to_string(),
            data_type: DataType::Integer,
            default_value: VariableValue::Integer(x),
        },
        Port {
            name: "Y".to_string(),
            data_type: DataType::Integer,
            default_value: VariableValue::Integer(y),
        },
    ];

    let outputs = vec![Port {
        name: "Next".to_string(),
        data_type: DataType::ExecutionFlow,
        default_value: VariableValue::None,
    }];

    Node {
        id: Uuid::new_v4(),
        node_type,
        position,
        inputs,
        outputs,
        z_order: 0,
        display_name: None,
    }
}

pub fn create_mouse_move_node(position: (f32, f32), x: i64, y: i64) -> Node {
    let inputs = vec![
        Port {
            name: "In".to_string(),
            data_type: DataType::ExecutionFlow,
            default_value: VariableValue::None,
        },
        Port {
            name: "X".to_string(),
            data_type: DataType::Integer,
            default_value: VariableValue::Integer(x),
        },
        Port {
            name: "Y".to_string(),
            data_type: DataType::Integer,
            default_value: VariableValue::Integer(y),
        },
    ];

    let outputs = vec![Port {
        name: "Next".to_string(),
        data_type: DataType::ExecutionFlow,
        default_value: VariableValue::None,
    }];

    Node {
        id: Uuid::new_v4(),
        node_type: NodeType::MouseMove,
        position,
        inputs,
        outputs,
        z_order: 0,
        display_name: None,
    }
}
