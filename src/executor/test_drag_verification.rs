use crate::graph::{BlueprintGraph, Connection, Node, Port, VariableValue};
use crate::node_types::{NodeType, DataType};
use crate::executor::ExecutionContext;
use crate::executor::Interpreter;
use crate::executor::events::ExecutionEvent;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use uuid::Uuid;
use std::thread;

fn create_node(id: Uuid, node_type: NodeType, x: f32, y: f32) -> Node {
    Node {
        id,
        node_type,
        position: (x, y),
        inputs: vec![],
        outputs: vec![],
        z_order: 0,
        display_name: None,
        enabled: true,
        group_id: None,
        note_text: String::new(),
        note_size: (200.0, 100.0),
    }
}

// Helper for Port creation
fn mk_port(name: &str, dt: DataType) -> Port {
    Port {
        name: name.to_string(),
        data_type: dt,
        default_value: VariableValue::None,
    }
}

fn mk_port_def(name: &str, dt: DataType, val: VariableValue) -> Port {
    Port {
        name: name.to_string(),
        data_type: dt,
        default_value: val,
    }
}

pub fn run_drag_verification() {
    println!("🧪 Starting Automated Drag Verification Test...");
    println!("TARGET: Dragging window 'egui Blueprint Node Editor' by 200px.");

    let mut graph = BlueprintGraph::default();

    // 1. Tick
    let tick_id = Uuid::new_v4();
    let mut tick = create_node(tick_id, NodeType::BlueprintFunction { name: "Event Tick".into() }, 0.0, 0.0);
    tick.outputs.push(mk_port("Next", DataType::ExecutionFlow));
    graph.nodes.insert(tick_id, tick);

    // 2. GetWindowPosition (ID 1)
    let get_pos_id = Uuid::new_v4();
    let mut get_pos = create_node(get_pos_id, NodeType::GetWindowPosition, 200.0, 0.0);
    get_pos.inputs.push(mk_port("Exec", DataType::ExecutionFlow));
    get_pos.inputs.push(mk_port_def("Title", DataType::String, VariableValue::String("egui Blueprint Node Editor".into())));
    get_pos.outputs.push(mk_port("Next", DataType::ExecutionFlow));
    get_pos.outputs.push(mk_port("X", DataType::Integer));
    get_pos.outputs.push(mk_port("Y", DataType::Integer));
    graph.nodes.insert(get_pos_id, get_pos);

    graph.connections.push(Connection { from_node: tick_id, from_port: "Next".into(), to_node: get_pos_id, to_port: "Exec".into() });

    // Math: X + 200
    let add_x_id = Uuid::new_v4();
    let mut add_x = create_node(add_x_id, NodeType::Add, 300.0, 100.0);
    add_x.inputs.push(mk_port("A", DataType::Integer)); 
    add_x.inputs.push(mk_port_def("B", DataType::Integer, VariableValue::Integer(200)));
    add_x.outputs.push(mk_port("Result", DataType::Integer));
    graph.nodes.insert(add_x_id, add_x);

    // Math: Y + 10
    let add_y_id = Uuid::new_v4();
    let mut add_y = create_node(add_y_id, NodeType::Add, 300.0, 200.0);
    add_y.inputs.push(mk_port("A", DataType::Integer)); 
    add_y.inputs.push(mk_port_def("B", DataType::Integer, VariableValue::Integer(10)));
    add_y.outputs.push(mk_port("Result", DataType::Integer));
    graph.nodes.insert(add_y_id, add_y);

    graph.connections.push(Connection { from_node: get_pos_id, from_port: "X".into(), to_node: add_x_id, to_port: "A".into() });
    graph.connections.push(Connection { from_node: get_pos_id, from_port: "Y".into(), to_node: add_y_id, to_port: "A".into() });

    // 3. MouseDown
    let down_id = Uuid::new_v4();
    let mut down = create_node(down_id, NodeType::MouseDown, 400.0, 0.0);
    down.inputs.push(mk_port("Exec", DataType::ExecutionFlow));
    down.inputs.push(mk_port_def("Button", DataType::String, VariableValue::String("Left".into())));
    down.inputs.push(mk_port("X", DataType::Integer)); 
    down.inputs.push(mk_port("Y", DataType::Integer)); 
    down.outputs.push(mk_port("Next", DataType::ExecutionFlow));
    graph.nodes.insert(down_id, down);

    graph.connections.push(Connection { from_node: get_pos_id, from_port: "Next".into(), to_node: down_id, to_port: "Exec".into() });
    graph.connections.push(Connection { from_node: add_x_id, from_port: "Result".into(), to_node: down_id, to_port: "X".into() });
    graph.connections.push(Connection { from_node: add_y_id, from_port: "Result".into(), to_node: down_id, to_port: "Y".into() });

    // 4. Delay
    let delay1_id = Uuid::new_v4();
    let mut delay1 = create_node(delay1_id, NodeType::Delay, 600.0, 0.0);
    delay1.inputs.push(mk_port("Exec", DataType::ExecutionFlow));
    delay1.inputs.push(mk_port_def("Duration (ms)", DataType::Integer, VariableValue::Integer(200)));
    delay1.outputs.push(mk_port("Next", DataType::ExecutionFlow));
    graph.nodes.insert(delay1_id, delay1);
    graph.connections.push(Connection { from_node: down_id, from_port: "Next".into(), to_node: delay1_id, to_port: "Exec".into() });

    // Math: X + 400
    let add_x2_id = Uuid::new_v4();
    let mut add_x2 = create_node(add_x2_id, NodeType::Add, 500.0, 100.0);
    add_x2.inputs.push(mk_port("A", DataType::Integer)); // Connect GetPos.X
    add_x2.inputs.push(mk_port_def("B", DataType::Integer, VariableValue::Integer(400)));
    add_x2.outputs.push(mk_port("Result", DataType::Integer));
    graph.nodes.insert(add_x2_id, add_x2);
    graph.connections.push(Connection { from_node: get_pos_id, from_port: "X".into(), to_node: add_x2_id, to_port: "A".into() });

    // 5. MouseMove
    let move_id = Uuid::new_v4();
    let mut move_node = create_node(move_id, NodeType::MouseMove, 800.0, 0.0);
    move_node.inputs.push(mk_port("Exec", DataType::ExecutionFlow));
    move_node.inputs.push(mk_port("X", DataType::Integer)); 
    move_node.inputs.push(mk_port("Y", DataType::Integer)); 
    move_node.outputs.push(mk_port("Next", DataType::ExecutionFlow));
    graph.nodes.insert(move_id, move_node);
    
    graph.connections.push(Connection { from_node: delay1_id, from_port: "Next".into(), to_node: move_id, to_port: "Exec".into() });
    graph.connections.push(Connection { from_node: add_x2_id, from_port: "Result".into(), to_node: move_id, to_port: "X".into() });
    graph.connections.push(Connection { from_node: add_y_id, from_port: "Result".into(), to_node: move_id, to_port: "Y".into() });

    // 6. Delay
    let delay2_id = Uuid::new_v4();
    let mut delay2 = create_node(delay2_id, NodeType::Delay, 1000.0, 0.0);
    delay2.inputs.push(mk_port("Exec", DataType::ExecutionFlow));
    delay2.inputs.push(mk_port_def("Duration (ms)", DataType::Integer, VariableValue::Integer(200)));
    delay2.outputs.push(mk_port("Next", DataType::ExecutionFlow));
    graph.nodes.insert(delay2_id, delay2);
    graph.connections.push(Connection { from_node: move_id, from_port: "Next".into(), to_node: delay2_id, to_port: "Exec".into() }); // Fixed move_node usage to move_id

    // 7. MouseUp
    let up_id = Uuid::new_v4();
    let mut up = create_node(up_id, NodeType::MouseUp, 1200.0, 0.0);
    up.inputs.push(mk_port("Exec", DataType::ExecutionFlow));
    up.inputs.push(mk_port_def("Button", DataType::String, VariableValue::String("Left".into())));
    up.outputs.push(mk_port("Next", DataType::ExecutionFlow));
    graph.nodes.insert(up_id, up);
    graph.connections.push(Connection { from_node: delay2_id, from_port: "Next".into(), to_node: up_id, to_port: "Exec".into() });

    // 8. Delay
    let delay3_id = Uuid::new_v4();
    let mut delay3 = create_node(delay3_id, NodeType::Delay, 1300.0, 0.0);
    delay3.inputs.push(mk_port("Exec", DataType::ExecutionFlow));
    delay3.inputs.push(mk_port_def("Duration (ms)", DataType::Integer, VariableValue::Integer(500)));
    delay3.outputs.push(mk_port("Next", DataType::ExecutionFlow));
    graph.nodes.insert(delay3_id, delay3);
    graph.connections.push(Connection { from_node: up_id, from_port: "Next".into(), to_node: delay3_id, to_port: "Exec".into() });

    // 9. GetWindowPos
    let get_pos2_id = Uuid::new_v4();
    let mut get_pos2 = create_node(get_pos2_id, NodeType::GetWindowPosition, 1400.0, 0.0);
    get_pos2.inputs.push(mk_port("Exec", DataType::ExecutionFlow));
    get_pos2.inputs.push(mk_port_def("Title", DataType::String, VariableValue::String("egui Blueprint Node Editor".into())));
    get_pos2.outputs.push(mk_port("Next", DataType::ExecutionFlow));
    get_pos2.outputs.push(mk_port("X", DataType::Integer));
    graph.nodes.insert(get_pos2_id, get_pos2);
    graph.connections.push(Connection { from_node: delay3_id, from_port: "Next".into(), to_node: get_pos2_id, to_port: "Exec".into() });

    // Execute
    let (tx, rx) = channel();
    let context = Arc::new(Mutex::new(ExecutionContext::new()));
    
    thread::spawn(move || {
        Interpreter::execute_flow(Arc::new(graph), tick_id, context, tx);
    });

    thread::spawn(move || {
        while let Ok(event) = rx.recv() {
            if let ExecutionEvent::Log(msg) = event {
                println!("TEST_LOG: {}", msg);
            }
        }
    });
}
