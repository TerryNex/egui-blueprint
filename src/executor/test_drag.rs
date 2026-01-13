use crate::graph::{BlueprintGraph, Connection, Node, Port, VariableValue};
use crate::node_types::NodeType;
use crate::executor::Interpreter;
use crate::executor::events::ExecutionEvent;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::sync::mpsc::channel;
use uuid::Uuid;
use rdev::{EventType, listen};

fn create_node(id: Uuid, node_type: NodeType) -> Node {
    Node {
        id,
        node_type,
        position: (0.0, 0.0),
        inputs: Vec::new(),
        outputs: Vec::new(),
        z_order: 0,
        display_name: None,
        enabled: true,
        group_id: None,
        note_text: String::new(),
        note_size: (200.0, 100.0),
    }
}

// Function to run the verification
pub fn run_drag_test() {
    println!("🧪 Starting Mouse Action Verification Test...");

    // 1. Setup Event Listener to track MouseMoves
    let (tx, rx) = channel();
    let listener_tx = tx.clone();
    
    // Spawn listener thread
    thread::spawn(move || {
        if let Err(error) = listen(move |event| {
            if let EventType::MouseMove { x, y } = event.event_type {
                let _ = listener_tx.send((x, y));
            }
        }) {
            println!("Listener error: {:?}", error);
        }
    });

    // Wait for listener to initialize
    thread::sleep(Duration::from_secs(1));

    // 2. Create Graph + Interpreter
    let mut graph = BlueprintGraph::default();
    let context = Arc::new(Mutex::new(crate::executor::ExecutionContext::new()));
    // Interpreter::new doesn't exist/needed for static methods

    println!("--- Test Case 1: MouseDown with defaults (Should NOT move) ---");
    
    let node_id = Uuid::new_v4();
    let node = create_node(node_id, NodeType::MouseDown);
    graph.nodes.insert(node_id, node);

    // Run just this node
    // We need to create an "Event Tick" node to start execution via execute_flow
    let tick_id = Uuid::new_v4();
    let tick_node = create_node(tick_id, NodeType::BlueprintFunction { name: "Event Tick".to_string() });
    
    // Connect Tick -> MouseDown
    graph.nodes.insert(tick_id, tick_node);
    graph.connections.push(Connection {
        from_node: tick_id,
        from_port: "Next".to_string(),
        to_node: node_id,
        to_port: "Exec".to_string(),
    });

    let (tx_dummy, _) = channel();
    Interpreter::execute_flow(
        Arc::new(graph.clone()),
        tick_id,
        context.clone(),
        tx_dummy.clone()
    );

    // Analyze events
    thread::sleep(Duration::from_millis(500));
    let mut moves_to_zero = false;
    while let Ok((x, y)) = rx.try_recv() {
        if x < 1.0 && y < 1.0 {
            moves_to_zero = true;
        }
    }
    
    if moves_to_zero {
        println!("❌ Test 1 FAILED: Mouse moved to (0,0) with empty inputs!");
    } else {
        println!("✅ Test 1 PASSED: Mouse did not reset to (0,0).");
    }

    println!("--- Test Case 2: MouseDown with Inputs (Should Move) ---");
    let node_id_2 = Uuid::new_v4();
    let mut node_2 = create_node(node_id_2, NodeType::MouseDown);
    // Set inputs manually
    node_2.inputs.push(Port {
        name: "X".to_string(),
        data_type: crate::node_types::DataType::Integer,
        default_value: VariableValue::Integer(500),
    });
    node_2.inputs.push(Port {
        name: "Y".to_string(),
        data_type: crate::node_types::DataType::Integer,
        default_value: VariableValue::Integer(500),
    });
    
    let mut graph2 = BlueprintGraph::default();
    graph2.nodes.insert(node_id_2, node_2.clone());
    
    let tick_id_2 = Uuid::new_v4();
    let tick_node_2 = create_node(tick_id_2, NodeType::BlueprintFunction { name: "Event Tick".to_string() });
    graph2.nodes.insert(tick_id_2, tick_node_2);
    graph2.connections.push(Connection {
        from_node: tick_id_2,
        from_port: "Next".to_string(),
        to_node: node_id_2,
        to_port: "Exec".to_string(),
    });

    let (tx_dummy, rx_log) = channel();
    Interpreter::execute_flow(
        Arc::new(graph2),
        tick_id_2,
        context.clone(),
        tx_dummy.clone()
    );
    
    // Print logs
    while let Ok(event) = rx_log.try_recv() {
         if let ExecutionEvent::Log(msg) = event {
            println!("LOG: {}", msg);
         }
    }

    thread::sleep(Duration::from_millis(1000));
    
    let mut moved_to_target = false;
    while let Ok((x, y)) = rx.try_recv() {
        if (x - 500.0).abs() < 10.0 && (y - 500.0).abs() < 10.0 {
            moved_to_target = true;
        }
    }

    if moved_to_target {
        println!("✅ Test 2 PASSED: Mouse moved to (500,500) as requested.");
    } else {
        println!("❌ Test 2 FAILED: Mouse did not move to target.");
    }

    println!("🧪 Verification complete.");
}
