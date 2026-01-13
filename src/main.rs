mod editor;
mod executor;
mod graph;
mod history;
mod node_types;
mod recorder;

use chrono::Local;
use editor::GraphEditor;
use eframe::egui;
use graph::{BlueprintGraph, Node, Port};
use history::UndoStack;
use node_types::{DataType, NodeType};
use rdev;
use executor::events::ExecutionEvent;
use std::sync::mpsc::Receiver;
use sysinfo::{Pid, ProcessesToUpdate, System};
use uuid::Uuid;

fn main() -> eframe::Result<()> {
    env_logger::init();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Automation Blueprint",
        native_options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            // Default fonts look a bit small for blueprint, let's keep default for now.
            Ok(Box::new(MyApp::default()))
        }),
    )
}

struct MyApp {
    graph: BlueprintGraph,
    editor: GraphEditor,
    logs: Vec<String>,
    script_name: String,
    show_load_window: bool,
    show_nodes_window: bool,
    show_debug_window: bool,
    start_time: std::time::Instant,
    undo_stack: UndoStack,
    log_receiver: Option<Receiver<ExecutionEvent>>,
    /// Stop handle for force stopping execution
    stop_handle: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    // Debug window state
    system: System,
    frame_times: Vec<f32>,
    last_frame_time: std::time::Instant,
    // Recording Options
    use_relative_coords: bool,
    last_recorded_node_id: Option<Uuid>,
    // Mouse buffering for Click/Drag distinction
    pending_mouse_down: Option<(std::time::Instant, f64, f64, rdev::Button)>,
    is_dragging: bool,
    // Nodes window state
    nodes_search_filter: String,
    nodes_sort_mode: u8,
    last_event_time: Option<std::time::Instant>,
    show_style_window: bool,
    // Recording enhancements
    recorded_node_ids: Vec<Uuid>,     // IDs of nodes created during current recording session
    record_delays: bool,               // Whether to insert Delay nodes between actions
    show_record_options: bool,         // Show recording options popup
    // Global hotkey receiver for F3 stop
    global_stop_rx: std::sync::mpsc::Receiver<()>,
    // Deduplication state
    last_recorded_event_type: Option<rdev::EventType>,
    // Cursor info overlay - shows position and pixel color
    show_cursor_info: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        // Setup global F3 hotkey listener for force stop
        // Setup global hotkey listener for force stop
        let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
        std::thread::spawn(move || {
            let mut ctrl_pressed = false;
            let mut meta_pressed = false;
            
            if let Err(error) = rdev::listen(move |event| {
                match event.event_type {
                    rdev::EventType::KeyPress(key) => {
                        match key {
                            rdev::Key::F3 => {
                                println!("[System] F3 Pressed - sending stop signal");
                                let _ = stop_tx.send(());
                            }
                            rdev::Key::ControlLeft | rdev::Key::ControlRight => {
                                ctrl_pressed = true;
                            }
                            rdev::Key::MetaLeft | rdev::Key::MetaRight => {
                                meta_pressed = true;
                            }
                            rdev::Key::KeyZ => {
                                // Check for Ctrl+Z or Cmd+Z (Meta+Z)
                                if ctrl_pressed || meta_pressed {
                                    println!("[System] Ctrl/Cmd+Z Pressed - sending stop signal");
                                    let _ = stop_tx.send(());
                                }
                            }
                            _ => {}
                        }
                    }
                    rdev::EventType::KeyRelease(key) => {
                        match key {
                             rdev::Key::ControlLeft | rdev::Key::ControlRight => {
                                ctrl_pressed = false;
                            }
                            rdev::Key::MetaLeft | rdev::Key::MetaRight => {
                                meta_pressed = false;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }) {
                eprintln!("[Error] Global hotkey listener failed: {:?}", error);
            }
        });
        
        let mut app = Self {
            graph: BlueprintGraph::default(),
            editor: GraphEditor::default(),
            logs: Vec::new(),
            script_name: "blueprint".to_string(),
            show_load_window: false,
            show_nodes_window: true,
            show_debug_window: false,
            start_time: std::time::Instant::now(),
            undo_stack: UndoStack::default(),
            log_receiver: None,
            stop_handle: None,
            system: System::new_all(),
            frame_times: Vec::with_capacity(120),
            last_frame_time: std::time::Instant::now(),
            use_relative_coords: false,
            last_recorded_node_id: None,
            pending_mouse_down: None,
            is_dragging: false,
            nodes_search_filter: String::new(),
            nodes_sort_mode: 0,
            last_event_time: None,
            show_style_window: false,
            recorded_node_ids: Vec::new(),
            record_delays: true,
            show_record_options: false,
            global_stop_rx: stop_rx,
            last_recorded_event_type: None,
            show_cursor_info: false,
        };
        let _ = std::fs::create_dir_all("scripts");
        // Load settings first (may auto-load last script)
        let script_loaded = app.load_settings();
        // Only add test nodes if no script was loaded
        if !script_loaded {
            app.add_test_nodes();
        }
        app
    }
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
struct AppSettings {
    style: editor::EditorStyle,
    history_max_records: usize,
    #[serde(default)]
    last_script_name: Option<String>,
}

impl MyApp {
    fn load_settings(&mut self) -> bool {
        if let Ok(json) = std::fs::read_to_string("settings.json") {
            if let Ok(settings) = serde_json::from_str::<AppSettings>(&json) {
                self.editor.style = settings.style;
                self.undo_stack.max_records = settings.history_max_records;
                self.logs.push("[System] Settings loaded.".to_string());

                // Auto-load last script
                if let Some(ref last_script) = settings.last_script_name {
                    let path = format!("scripts/{}.json", last_script);
                    if let Ok(script_json) = std::fs::read_to_string(&path) {
                        if let Ok(graph) = serde_json::from_str(&script_json) {
                            self.graph = graph;
                            self.script_name = last_script.clone();
                            self.logs
                                .push(format!("[System] Auto-loaded last script: {}", last_script));
                            return true; // Script was loaded
                        }
                    }
                }
            }
        }
        false // No script loaded
    }

    fn save_settings(&self) {
        let settings = AppSettings {
            style: self.editor.style.clone(),
            history_max_records: self.undo_stack.max_records,
            last_script_name: Some(self.script_name.clone()),
        };
        if let Ok(json) = serde_json::to_string_pretty(&settings) {
            let _ = std::fs::write("settings.json", json);
        }
    }

    fn add_test_nodes(&mut self) {
        use crate::graph::VariableValue;
        let id1 = Uuid::new_v4();
        self.graph.nodes.insert(
            id1,
            Node {
                id: id1,
                node_type: NodeType::BlueprintFunction {
                    name: "Event Tick".into(),
                },
                position: (100.0, 100.0),
                inputs: vec![],
                outputs: vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
                z_order: 0,
                display_name: None,
                enabled: true,
                group_id: None,
                note_text: String::new(),
                note_size: (200.0, 100.0),
            },
        );

        let id2 = Uuid::new_v4();
        self.graph.nodes.insert(
            id2,
            Node {
                id: id2,
                node_type: NodeType::BlueprintFunction {
                    name: "Print String".into(),
                },
                position: (400.0, 100.0),
                inputs: vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "String".into(),
                        data_type: DataType::String,
                        default_value: VariableValue::String("Hello".into()),
                    },
                ],
                outputs: vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
                z_order: 1,
                display_name: None,
                enabled: true,
                group_id: None,
                note_text: String::new(),
                note_size: (200.0, 100.0),
            },
        );
    }

    /// Quick capture: Use macOS screencapture to interactively select a region,
    /// save it as a template, and create a FindImage node with the path pre-filled.
    fn perform_quick_capture(&mut self, _ctx: &egui::Context) {
        // Generate unique filename
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S_%3f");
        let filename = format!("scripts/templates/capture_{}.png", timestamp);
        
        // Ensure templates directory exists
        let _ = std::fs::create_dir_all("scripts/templates");
        
        self.logs.push("[Capture] Starting interactive capture...".into());
        
        // Use macOS screencapture with interactive selection
        // -i = interactive mode (user selects region)
        // -x = no sound
        // -r = don't add shadow to window captures
        #[cfg(target_os = "macos")]
        let result = std::process::Command::new("screencapture")
            .args(["-i", "-x", "-r", &filename])
            .output();
        
        #[cfg(not(target_os = "macos"))]
        let result: Result<std::process::Output, std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Quick capture only supported on macOS"
        ));
        
        match result {
            Ok(output) => {
                if output.status.success() {
                    // Check if file was actually created (user might have cancelled with Escape)
                    if std::path::Path::new(&filename).exists() {
                        self.logs.push(format!("[Capture] Saved to {}", filename));
                        
                        // Create FindImage node with the captured image
                        self.create_find_image_node(&filename);
                    } else {
                        self.logs.push("[Capture] Cancelled by user".into());
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    self.logs.push(format!("[Capture] Failed: {}", stderr));
                }
            }
            Err(e) => {
                self.logs.push(format!("[Capture] Error: {}", e));
            }
        }
    }
    
    /// Create a FindImage node pre-filled with the given image path
    fn create_find_image_node(&mut self, image_path: &str) {
        use crate::graph::VariableValue;
        
        // Get ports for FindImage node
        let (mut inputs, outputs) = GraphEditor::get_ports_for_type(&NodeType::FindImage);
        
        // Pre-fill the ImagePath input
        for port in &mut inputs {
            if port.name == "ImagePath" {
                port.default_value = VariableValue::String(image_path.to_string());
            }
        }
        
        // Calculate position: place to the right of rightmost node
        let max_x = self.graph.nodes.values()
            .map(|n| n.position.0)
            .fold(100.0f32, f32::max);
        
        let node_id = Uuid::new_v4();
        let z_order = self.editor.next_z_order;
        self.editor.next_z_order += 1;
        
        let node = Node {
            id: node_id,
            node_type: NodeType::FindImage,
            position: (max_x + 250.0, 100.0),
            inputs,
            outputs,
            z_order,
            display_name: Some("Quick Capture".into()),
            enabled: true,
            group_id: None,
            note_text: String::new(),
            note_size: (200.0, 100.0),
        };
        
        self.graph.nodes.insert(node_id, node);
        
        // Select the new node
        self.editor.selected_nodes.clear();
        self.editor.selected_nodes.insert(node_id);
        
        self.logs.push(format!("[Capture] Created FindImage node with {}", image_path));
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request continuous repaint while recording or executing
        // Use a small interval (100ms) to reduce CPU usage while staying responsive
        if self.editor.recorder.is_recording() || self.log_receiver.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        
        // Check for global F3 hotkey to force stop execution
        while let Ok(()) = self.global_stop_rx.try_recv() {
            if let Some(ref handle) = self.stop_handle {
                handle.store(true, std::sync::atomic::Ordering::Relaxed);
                self.logs.push("[System] F3 - Force Stop requested".to_string());
            }
            self.stop_handle = None;
            self.log_receiver = None;
        }
        
        // Verification Test Trigger (Added for Drag Debugging)
        if ctx.input(|i| i.key_pressed(egui::Key::F6)) {
             crate::executor::test_drag_verification::run_drag_verification();
        }
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Blueprint Editor (Custom)");
                ui.separator();
                if ui.button("Zoom In").clicked() {
                    self.editor.zoom *= 1.1;
                }
                if ui.button("Zoom Out").clicked() {
                    self.editor.zoom /= 1.1;
                }
                if ui.button("Center").clicked() {
                    // Center view on all existing nodes (Feature #2)
                    if !self.graph.nodes.is_empty() {
                        let min_x = self
                            .graph
                            .nodes
                            .values()
                            .map(|n| n.position.0)
                            .fold(f32::INFINITY, f32::min);
                        let max_x = self
                            .graph
                            .nodes
                            .values()
                            .map(|n| n.position.0)
                            .fold(f32::NEG_INFINITY, f32::max);
                        let min_y = self
                            .graph
                            .nodes
                            .values()
                            .map(|n| n.position.1)
                            .fold(f32::INFINITY, f32::min);
                        let max_y = self
                            .graph
                            .nodes
                            .values()
                            .map(|n| n.position.1)
                            .fold(f32::NEG_INFINITY, f32::max);

                        let center_x = (min_x + max_x) / 2.0;
                        let center_y = (min_y + max_y) / 2.0;

                        // Account for VIRTUAL_OFFSET (5000, 5000)
                        let virtual_offset = egui::Vec2::new(5000.0, 5000.0);
                        let node_center = egui::Vec2::new(center_x, center_y) + virtual_offset;

                        // Reset zoom and center view
                        self.editor.zoom = 1.0;
                        // Pan so that node_center is at screen center (approximate)
                        self.editor.pan = egui::Vec2::new(500.0, 350.0) - node_center;
                    } else {
                        self.editor.pan = egui::Vec2::ZERO;
                        self.editor.zoom = 1.0;
                    }
                }
                ui.separator();

                // if ui.button("Style").clicked() {
                //     self.editor.show_settings = !self.editor.show_settings;
                // }

                ui.label("Script:");
                ui.horizontal(|ui| {
                    ui.set_min_width(100.0);
                    
                    ui.set_max_width(150.0);
                    ui.text_edit_singleline(&mut self.script_name);
                });
                if ui.button("New").clicked() {
                    self.graph = graph::BlueprintGraph::default();
                    self.script_name = "untitled".to_string();
                    self.undo_stack = UndoStack::default();
                    self.logs.push("[System] New script created.".to_string());
                }

                if ui.button("Save").clicked() {
                    let name = if self.script_name.ends_with(".json") {
                        self.script_name.clone()
                    } else {
                        format!("{}.json", self.script_name)
                    };
                    if let Ok(json) = serde_json::to_string(&self.graph) {
                        let _ = std::fs::write(format!("scripts/{}", name), json);

                        // Save History
                        if let Ok(history_json) = serde_json::to_string(&self.undo_stack) {
                            let history_name =
                                format!("{}.history", name.trim_end_matches(".json"));
                            let _ =
                                std::fs::write(format!("scripts/{}", history_name), history_json);
                        }

                        self.save_settings(); // Persist settings
                        self.logs.push(format!("[System] Saved {}", name));
                    }
                }
                if ui.button("Load").clicked() {
                    self.show_load_window = !self.show_load_window;
                }
                // ... (Run button - keep as is, but I can't simple skip it if I am replacing the block)
                if ui.button("▶ Run").clicked() {
                    log::info!("Running graph (async)...");
                    self.start_time = std::time::Instant::now();
                    let (rx, stop_handle) = executor::Interpreter::run_async_with_stop(&self.graph);
                    self.log_receiver = Some(rx);
                    self.stop_handle = Some(stop_handle);
                    self.logs
                        .push("[System] Async Execution Started".to_string());
                }
                // Force Stop button - only show when execution is running
                if self.stop_handle.is_some() {
                    if ui.button(egui::RichText::new("⏹ Stop").color(egui::Color32::RED)).clicked() {
                        if let Some(ref handle) = self.stop_handle {
                            handle.store(true, std::sync::atomic::Ordering::Relaxed);
                            self.logs.push("[System] Force Stop requested".to_string());
                        }
                        self.stop_handle = None;
                    }
                }
                // ui.separator();
                // if ui.button("Debug").clicked() {
                //     self.show_debug_window = !self.show_debug_window;
                // }
                // if ui.button("🎨 Style").clicked() {
                //     self.show_style_window = !self.show_style_window;
                // }

                ui.separator();
                let is_recording = self.editor.recorder.is_recording();
                let text = if is_recording {
                    "⏹ Stop Recording"
                } else {
                    "⏺ Record"
                };
                let color = if is_recording {
                    egui::Color32::RED
                } else {
                    egui::Color32::WHITE
                };
                if ui
                    .add(egui::Button::new(egui::RichText::new(text).color(color)))
                    .clicked()
                {
                    if is_recording {
                        self.editor.recorder.stop();
                        self.logs.push("[System] Recording Stopped".into());
                        
                        // Auto-group recorded nodes
                        if !self.recorded_node_ids.is_empty() {
                            let group_id = Uuid::new_v4();
                            for node_id in &self.recorded_node_ids {
                                if let Some(node) = self.graph.nodes.get_mut(node_id) {
                                    node.group_id = Some(group_id);
                                }
                            }
                            self.logs.push(format!("[System] Created group with {} recorded nodes", self.recorded_node_ids.len()));
                            self.recorded_node_ids.clear();
                        }
                        
                        self.last_recorded_node_id = None;
                        self.pending_mouse_down = None;
                    } else {
                        self.editor.recorder.start();
                        self.logs.push("[System] Recording Started...".into());
                        self.recorded_node_ids.clear(); // Clear for new session
                        self.last_recorded_node_id = None;
                        self.last_event_time = None;
                    }
                }

                // Recording Options button (⚙ gear icon)
                if ui.button("⚙").on_hover_text("Recording Options").clicked() {
                    self.show_record_options = !self.show_record_options;
                }

                // Quick Capture Button
                if ui.button("📸").on_hover_text("Capture screen region").clicked() {
                    self.perform_quick_capture(ctx);
                }
                
                ui.separator();
                if ui.button("Debug").clicked() {
                    self.show_debug_window = !self.show_debug_window;
                }
                if ui.button("🎨").on_hover_text("Style Settings").clicked() {
                    self.show_style_window = !self.show_style_window;
                }
            });
        });
        
        // Recording Options Popup Window
        if self.show_record_options {
            egui::Window::new("Recording Options")
                .collapsible(true)
                .resizable(false)
                .default_width(200.0)
                .show(ctx, |ui| {
                    ui.checkbox(&mut self.record_delays, "Record Delays")
                        .on_hover_text("Insert Delay nodes between actions (recommended)");
                    
                    ui.checkbox(&mut self.use_relative_coords, "Relative Coords")
                        .on_hover_text("Use coordinates relative to active window");
                    
                    let mut record_moves = self.editor.recorder.is_record_mouse_move();
                    if ui.checkbox(&mut record_moves, "Record Mouse Moves")
                        .on_hover_text("Record mouse movement (creates many nodes)")
                        .changed()
                    {
                        self.editor.recorder.set_record_mouse_move(record_moves);
                    }
                    
                    ui.separator();
                    ui.checkbox(&mut self.show_cursor_info, "Show Cursor Info")
                        .on_hover_text("Display cursor coordinates and pixel color (like screenshot tool)");
                    
                    ui.separator();
                    if ui.button("Close").clicked() {
                        self.show_record_options = false;
                    }
                });
        }

        // Process Recorded Events
        while let Ok(action) = self.editor.recorder.rx.try_recv() {
            let mut final_action = action;
            // 1. Relative Coordinates
            // (Placeholder. For now use screen coords as requested basic feature)

            // 2. Click vs Drag Logic
            let mut nodes_to_add = Vec::new();


            // Helper to convert Button to string
            fn btn_to_str(b: rdev::Button) -> String {
                match b {
                    rdev::Button::Left => "Left".to_string(),
                    rdev::Button::Right => "Right".to_string(),
                    rdev::Button::Middle => "Middle".to_string(),
                    _ => "Left".to_string(),
                }
            }

            // Deduplication Logic
            // rdev on macOS can sometimes send duplicate events (e.g. 2 press events).
            // We check if the event type matches the last one and is within a small time window (50ms).
            let current_type = final_action.event.event_type; // EventType is likely Copy/Clone
            let is_duplicate = if let Some(last_type) = self.last_recorded_event_type {
                if last_type == current_type {
                    if let Some(last_time) = self.last_event_time {
                         last_time.elapsed().as_millis() < 50
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            };

            if is_duplicate {
                // self.logs.push(format!("[Record] Ignored duplicate event: {:?}", current_type));
                continue;
            }
            
            self.last_recorded_event_type = Some(current_type);

            match final_action.event.event_type {
                rdev::EventType::ButtonPress(button) => {
                    // Store press info for release comparison - don't create node yet
                    // We'll decide on Click vs Drag when button is released
                    self.pending_mouse_down = Some((
                        std::time::Instant::now(),
                        final_action.cursor_position.0,
                        final_action.cursor_position.1,
                        button,
                    ));
                    self.is_dragging = false; // Will be determined on release
                }
                rdev::EventType::ButtonRelease(button) => {
                    let release_x = final_action.cursor_position.0;
                    let release_y = final_action.cursor_position.1;
                    
                    // Check if this was a quick click (< 200ms, minimal movement < 5px)
                    let is_click = if let Some((press_time, press_x, press_y, press_btn)) = self.pending_mouse_down {
                        if press_btn == button {
                            let elapsed_ms = press_time.elapsed().as_millis();
                            let dx = (release_x - press_x).abs();
                            let dy = (release_y - press_y).abs();
                            elapsed_ms < 200 && dx < 5.0 && dy < 5.0
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    // Insert delay before this action if enabled
                    if self.record_delays {
                        if let Some(last_time) = self.last_event_time {
                            let elapsed = last_time.elapsed().as_secs_f32();
                            if elapsed > 0.05 {
                                let delay_node = recorder::mapper::create_delay_node((0.0, 0.0), elapsed);
                                nodes_to_add.push(delay_node);
                            }
                        }
                    }
                    self.last_event_time = Some(std::time::Instant::now());

                    if is_click {
                        // Create Click or RightClick node
                        let node_type = match button {
                            rdev::Button::Right => NodeType::RightClick,
                            _ => NodeType::Click,
                        };
                        let (x, y) = if let Some((_, press_x, press_y, _)) = self.pending_mouse_down {
                            (press_x as i64, press_y as i64)
                        } else {
                            (release_x as i64, release_y as i64)
                        };
                        let node = recorder::mapper::create_click_node(
                            node_type,
                            (0.0, 0.0),
                            x,
                            y,
                        );
                        nodes_to_add.push(node);
                    } else {
                        // Drag operation: create MouseDown + MouseUp pair
                        let btn_str = btn_to_str(button);
                        
                        // Create MouseDown at press position
                        if let Some((_, press_x, press_y, _)) = self.pending_mouse_down {
                            let down_node = recorder::mapper::create_mouse_btn_node(
                                NodeType::MouseDown,
                                (0.0, 0.0),
                                btn_str.clone(),
                                press_x as i64,
                                press_y as i64,
                            );
                            nodes_to_add.push(down_node);
                        }
                        
                        // Create MouseUp at release position
                        let up_node = recorder::mapper::create_mouse_btn_node(
                            NodeType::MouseUp,
                            (0.0, 0.0),
                            btn_str,
                            release_x as i64,
                            release_y as i64,
                        );
                        nodes_to_add.push(up_node);
                    }

                    // Clear pending state
                    self.pending_mouse_down = None;
                    self.is_dragging = false;
                }

                _ => {
                    // Prevent duplicate processing of Button events if they fall through
                    if matches!(final_action.event.event_type, rdev::EventType::ButtonPress(_) | rdev::EventType::ButtonRelease(_)) {
                        continue;
                    }
                
                    // KeyPress or MouseMove
                    // Insert delay before this action if enabled
                    if self.record_delays {
                        if let Some(last_time) = self.last_event_time {
                            let elapsed = last_time.elapsed().as_secs_f32();
                            if elapsed > 0.05 {
                                let delay_node = recorder::mapper::create_delay_node((0.0, 0.0), elapsed);
                                nodes_to_add.push(delay_node);
                            }
                        }
                    }
                    self.last_event_time = Some(std::time::Instant::now());

                    let pos = (0.0, 0.0);
                    if let Some(node) = recorder::mapper::map_action_to_node(final_action, pos) {
                        nodes_to_add.push(node);
                    }
                }
            }

            for mut node in nodes_to_add {
                // Layout Configuration
                const NODES_PER_ROW: usize = 30;
                const NODE_SPACING_X: f32 = 250.0;
                const ROW_SPACING_Y: f32 = 250.0; // Space between rows
                const START_X: f32 = 100.0; // Starting X coordinate for new rows

                // Calculate current count in this session
                let current_count = self.recorded_node_ids.len();
                
                // Determine layout
                if let Some(prev_id) = self.last_recorded_node_id {
                    let prev_pos = self.graph.nodes.get(&prev_id).map(|n| n.position).unwrap_or((0.0, 0.0));
                    
                    if current_count > 0 && current_count % NODES_PER_ROW == 0 {
                        // WRAP TO NEW LINE
                        // Start a new row below the previous row's starting Y?
                        // Actually, we want to go down from the PREVIOUS node's Y?
                        // Let's use a simpler approach: 
                        // If we are wrapping, we go back to START_X and move Y down.
                        
                        // We need to know the Y of the current row. 
                        // Assuming nodes in a row have same Y.
                        node.position = (START_X, prev_pos.1 + ROW_SPACING_Y);
                    } else {
                        // Continue on same row
                        node.position = (prev_pos.0 + NODE_SPACING_X, prev_pos.1);
                    }

                    // Auto-Connect
                    self.graph.connections.push(crate::graph::Connection {
                        from_node: prev_id,
                        from_port: "Next".to_string(),
                        to_node: node.id,
                        to_port: "In".to_string(),
                    });
                } else {
                    // Start of new chain - Find rightmost point of existing graph or start fresh
                    let max_x = self
                        .graph
                        .nodes
                        .values()
                        .map(|n| n.position.0)
                        .fold(START_X, f32::max);
                        
                    let start_y = self
                        .graph
                        .nodes
                        .values()
                        .map(|n| n.position.1)
                        .fold(100.0, f32::max); // Try to start below existing nodes if any? 
                        // Or just stick to y=100 if we want to extend horizontally.
                        // But user wants to avoid long lines.
                    
                    // Simple logic: Start at (max_x + spacing, 100)
                    node.position = (max_x + NODE_SPACING_X, 100.0);
                }

                self.last_recorded_node_id = Some(node.id);
                self.recorded_node_ids.push(node.id); // Track for auto-grouping
                let name = format!("{:?}", node.node_type);
                self.logs.push(format!("[Record] Captured: {}", name));
                self.graph.nodes.insert(node.id, node);
            }
        }

        // Drain logs from async threads and detect execution completion
        if let Some(rx) = &self.log_receiver {
            let mut channel_closed = false;
            loop {
                match rx.try_recv() {
                    Ok(event) => match event {
                        ExecutionEvent::Log(msg) => {
                            let now = Local::now();
                            let time_str = now.format("%H:%M:%S").to_string();
                            self.logs.push(format!("[{}] {}", time_str, msg));
                        }
                        ExecutionEvent::NodeActive(node_id) => {
                            self.editor.node_execution_times.insert(node_id, std::time::Instant::now());
                        }
                        ExecutionEvent::NodeInactive(_) => {} // Optional future use
                        ExecutionEvent::Finished => {
                            // Can be used if we send explicit finish event
                        }
                    },
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        channel_closed = true;
                        break;
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                }
            }
            if channel_closed {
                // Execution thread has finished - clear log receiver and stop handle
                self.log_receiver = None;
                self.stop_handle = None;
                // keep execution times for fade out
                self.logs.push("[System] Execution Completed".to_string());
            }

            // Prune old execution times (keep for 1s for fade)
            self.editor.node_execution_times.retain(|_, time| time.elapsed().as_secs_f32() < 1.0);
            
            // Request repaint for smooth fade animation
            if !self.editor.node_execution_times.is_empty() {
                ctx.request_repaint();
            }
        }

        // Debug/Performance Window
        if self.show_debug_window {
            // Update frame time tracking
            let now = std::time::Instant::now();
            let frame_time = now.duration_since(self.last_frame_time).as_secs_f32();
            self.last_frame_time = now;
            self.frame_times.push(frame_time);
            if self.frame_times.len() > 120 {
                self.frame_times.remove(0);
            }

            // Update system info periodically (every 60 frames)
            if self.frame_times.len() % 60 == 0 {
                let pid = Pid::from_u32(std::process::id());
                self.system
                    .refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
            }

            egui::Window::new("Debug / Performance")
                .open(&mut self.show_debug_window)
                .resizable(true)
                .default_width(280.0)
                .show(ctx, |ui| {
                    // FPS
                    let avg_frame_time: f32 = if self.frame_times.is_empty() {
                        0.0
                    } else {
                        self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32
                    };
                    let fps = if avg_frame_time > 0.0 {
                        1.0 / avg_frame_time
                    } else {
                        0.0
                    };
                    ui.label(format!("FPS: {:.1}", fps));
                    ui.label(format!("Frame Time: {:.2} ms", avg_frame_time * 1000.0));
                    ui.separator();

                    // App Memory & CPU
                    if let Some(process) = self.system.process(Pid::from_u32(std::process::id())) {
                        let used_memory = process.memory(); // Bytes
                        ui.label(format!(
                            "Memory: {:.1} MB",
                            used_memory as f64 / 1_048_576.0
                        ));

                        let cpu_usage = process.cpu_usage();
                        ui.label(format!("CPU Usage: {:.1}%", cpu_usage));
                        ui.add(egui::ProgressBar::new(cpu_usage / 100.0));
                    } else {
                        ui.label("Could not retrieve process info");
                    }
                    ui.separator();

                    // Graph Stats
                    ui.label(format!("Nodes: {}", self.graph.nodes.len()));
                    ui.label(format!("Connections: {}", self.graph.connections.len()));
                    ui.label(format!("Groups: {}", self.graph.groups.len()));
                });
        }

        // Style Settings Window
        let mut show_style_window = self.show_style_window;
        if show_style_window {
            let mut should_save = false;
            let mut should_reset = false;
            
            egui::Window::new("🎨 Style Settings")
                .open(&mut show_style_window)
                .resizable(true)
                .default_width(300.0)
                .show(ctx, |ui| {
                    ui.heading("Display Settings");
                    ui.horizontal(|ui| {
                        ui.label("Font Size:");
                        ui.add(egui::Slider::new(&mut self.editor.style.font_size, 8.0..=24.0).suffix("px"));
                    });
                    ui.checkbox(&mut self.editor.style.use_gradient_connections, "Gradient Connections");
                    ui.separator();
                    
                    ui.heading("Node Header Colors");
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            let categories: Vec<String> = self.editor.style.header_colors.keys().cloned().collect();
                            for category in categories {
                                if let Some(color) = self.editor.style.header_colors.get_mut(&category) {
                                    ui.horizontal(|ui| {
                                        let mut rgb = [color.r(), color.g(), color.b()];
                                        if ui.color_edit_button_srgb(&mut rgb).changed() {
                                            *color = egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                                        }
                                        ui.label(&category);
                                    });
                                }
                            }
                        });
                    
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("💾 Save").clicked() {
                            should_save = true;
                        }
                        if ui.button("🔄 Reset to Defaults").clicked() {
                            should_reset = true;
                        }
                    });
                });
            
            // Handle actions outside the closure to avoid borrow issues
            if should_save {
                self.save_settings();
                self.logs.push("[System] Style settings saved".to_string());
            }
            if should_reset {
                self.editor.style = editor::EditorStyle::default();
                self.save_settings();
                self.logs.push("[System] Style reset to defaults".to_string());
            }
        }
        self.show_style_window = show_style_window;

        let mut show_load_window = self.show_load_window;
        let mut loaded_script = None;

        if show_load_window {
            egui::Window::new("Load Script")
                .open(&mut show_load_window)
                .show(ctx, |ui| {
                    if let Ok(entries) = std::fs::read_dir("scripts") {
                        for entry in entries.flatten() {
                            if let Ok(name) = entry.file_name().into_string() {
                                if name.ends_with(".json") {
                                    ui.horizontal(|ui| {
                                        if ui.button("🗑").on_hover_text("Delete Script").clicked()
                                        {
                                            if let Err(e) = std::fs::remove_file(entry.path()) {
                                                log::error!("Failed to delete script: {}", e);
                                            }
                                            // Also try to delete history
                                            let _ = std::fs::remove_file(
                                                entry.path().with_extension("history"),
                                            );
                                        }
                                        if ui.button(&name).clicked() {
                                            if let Ok(json) = std::fs::read_to_string(entry.path())
                                            {
                                                loaded_script = Some((
                                                    json,
                                                    name.trim_end_matches(".json").to_string(),
                                                ));
                                            }
                                        }
                                    });
                                }
                            }
                        }
                    }
                });
        }
        self.show_load_window = show_load_window;

        if let Some((json, name)) = loaded_script {
            if let Ok(graph) = serde_json::from_str(&json) {
                self.graph = graph;
                self.script_name = name.clone();
                // Load History
                let history_path = format!("scripts/{}.history", name);
                if let Ok(history_json) = std::fs::read_to_string(history_path) {
                    if let Ok(stack) = serde_json::from_str(&history_json) {
                        self.undo_stack = stack;
                    } else {
                        self.undo_stack = UndoStack::default();
                    }
                } else {
                    self.undo_stack = UndoStack::default();
                    self.undo_stack.push(&self.graph); // Initial state
                }

                self.logs.push(format!("[System] Loaded {}", name));
                self.show_load_window = false;
            }
        }

        // Output Log Window (resizable and movable)
        egui::Window::new("Output Log")
            .open(&mut true) // Always open, no close button needed
            .resizable(true)
            .collapsible(true)
            .default_width(600.0)
            .default_height(200.0)
            .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(10.0, -10.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Clear").clicked() {
                        self.logs.clear();
                    }
                    ui.separator();
                    
                    // Export to scripts/logs/
                    if ui.button("📁 Export").on_hover_text("Export to scripts/logs/").clicked() {
                        let _ = std::fs::create_dir_all("scripts/logs");
                        let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
                        let filename = format!("scripts/logs/log_{}.txt", timestamp);
                        let content = self.logs.join("\n");
                        match std::fs::write(&filename, &content) {
                            Ok(_) => self.logs.push(format!("[System] Exported to {}", filename)),
                            Err(e) => self.logs.push(format!("[Error] Export failed: {}", e)),
                        }
                    }
                    
                    // Quick export to Desktop
                    if ui.button("🖥 Desktop").on_hover_text("Export to Desktop").clicked() {
                        if let Some(home) = dirs::home_dir() {
                            let desktop = home.join("Desktop");
                            let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
                            let filename = desktop.join(format!("egui_blueprint_log_{}.txt", timestamp));
                            let content = self.logs.join("\n");
                            match std::fs::write(&filename, &content) {
                                Ok(_) => self.logs.push(format!("[System] Exported to {:?}", filename)),
                                Err(e) => self.logs.push(format!("[Error] Export failed: {}", e)),
                            }
                        } else {
                            self.logs.push("[Error] Could not find home directory".to_string());
                        }
                    }
                    
                    ui.separator();
                    ui.label(format!("Count: {}", self.logs.len()));
                });
                ui.separator();
                egui::ScrollArea::both()
                    .stick_to_bottom(true)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        // Use theme-aware text colors
                        let text_color = ui.visuals().strong_text_color();
                        let highlight_color = if ui.visuals().dark_mode {
                            egui::Color32::from_rgb(100, 255, 100)
                        } else {
                            egui::Color32::from_rgb(0, 150, 0)
                        };

                        for log in &self.logs {
                            let mut job = egui::text::LayoutJob::default();
                            let mut current_segment = String::new();
                            let mut in_var = false;

                            for c in log.chars() {
                                if c == '{' && !in_var {
                                    if !current_segment.is_empty() {
                                        job.append(
                                            &current_segment,
                                            0.0,
                                            egui::TextFormat {
                                                color: text_color,
                                                ..Default::default()
                                            },
                                        );
                                        current_segment.clear();
                                    }
                                    in_var = true;
                                    current_segment.push(c);
                                } else if c == '}' && in_var {
                                    current_segment.push(c);
                                    job.append(
                                        &current_segment,
                                        0.0,
                                        egui::TextFormat {
                                            color: highlight_color,
                                            ..Default::default()
                                        },
                                    );
                                    current_segment.clear();
                                    in_var = false;
                                } else {
                                    current_segment.push(c);
                                }
                            }
                            if !current_segment.is_empty() {
                                let color = if in_var { highlight_color } else { text_color };
                                job.append(
                                    &current_segment,
                                    0.0,
                                    egui::TextFormat {
                                        color,
                                        ..Default::default()
                                    },
                                );
                            }
                            ui.label(job);
                        }
                    });
            });

        // Nodes Window (collapsible)
        egui::Window::new("Nodes")
            .open(&mut self.show_nodes_window)
            .resizable(true)
            .collapsible(true)
            .default_width(200.0)
            .default_height(400.0)
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-10.0, 40.0))
            .show(ctx, |ui| {
                // Search bar
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    ui.text_edit_singleline(&mut self.nodes_search_filter);
                });

                // Sort buttons
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(self.nodes_sort_mode == 0, "Default")
                        .clicked()
                    {
                        self.nodes_sort_mode = 0;
                    }
                    if ui
                        .selectable_label(self.nodes_sort_mode == 1, "A-Z")
                        .clicked()
                    {
                        self.nodes_sort_mode = 1;
                    }
                    if ui
                        .selectable_label(self.nodes_sort_mode == 2, "Type")
                        .clicked()
                    {
                        self.nodes_sort_mode = 2;
                    }
                });
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Collect node info first to avoid borrow conflicts
                    let mut node_info: Vec<_> = self
                        .graph
                        .nodes
                        .values()
                        .map(|node| {
                            // Get node type name (clean format)
                            let type_name = match &node.node_type {
                                NodeType::BlueprintFunction { name } => name.clone(),
                                NodeType::GetVariable { name } => format!("Get: {}", name),
                                NodeType::SetVariable { name } => format!("Set: {}", name),
                                NodeType::Add => "Add".into(),
                                NodeType::Subtract => "Subtract".into(),
                                NodeType::Multiply => "Multiply".into(),
                                NodeType::Divide => "Divide".into(),
                                NodeType::Modulo => "Modulo".into(),
                                NodeType::Power => "Power".into(),
                                NodeType::Abs => "Abs".into(),
                                NodeType::Min => "Min".into(),
                                NodeType::Max => "Max".into(),
                                NodeType::Clamp => "Clamp".into(),
                                NodeType::Random => "Random".into(),
                                NodeType::Constant => "Constant".into(),
                                NodeType::ToInteger => "To Integer".into(),
                                NodeType::ToFloat => "To Float".into(),
                                NodeType::ToString => "To String".into(),
                                NodeType::Branch => "Branch".into(),
                                NodeType::Entry => "Entry".into(),
                                NodeType::ForLoop => "For Loop".into(),
                                NodeType::WhileLoop => "While Loop".into(),
                                NodeType::Delay => "Delay".into(),
                                NodeType::GetTimestamp => "Get Timestamp".into(),
                                NodeType::Sequence => "Sequence".into(),
                                NodeType::Gate => "Gate".into(),
                                NodeType::WaitForCondition => "Wait For Condition".into(),
                                NodeType::ForLoopAsync => "For Loop Async".into(),
                                NodeType::Equals => "Equals".into(),
                                NodeType::NotEquals => "Not Equals".into(),
                                NodeType::GreaterThan => "Greater Than".into(),
                                NodeType::GreaterThanOrEqual => "Greater or Equal".into(),
                                NodeType::LessThan => "Less Than".into(),
                                NodeType::LessThanOrEqual => "Less or Equal".into(),
                                NodeType::And => "And".into(),
                                NodeType::Or => "Or".into(),
                                NodeType::Not => "Not".into(),
                                NodeType::Xor => "Xor".into(),
                                NodeType::Concat => "Concat".into(),
                                NodeType::Split => "Split".into(),
                                NodeType::Length => "Length".into(),
                                NodeType::Contains => "Contains".into(),
                                NodeType::Replace => "Replace".into(),
                                NodeType::Format => "Format".into(),
                                NodeType::StringJoin => "String Join".into(),
                                NodeType::StringBetween => "String Between".into(),
                                NodeType::StringTrim => "String Trim".into(),
                                NodeType::ReadInput => "Read Input".into(),
                                NodeType::FileRead => "File Read".into(),
                                NodeType::FileWrite => "File Write".into(),
                                // Utility
                                NodeType::Notes => "Notes".into(),
                                // System Control
                                NodeType::RunCommand => "Run Command".into(),
                                NodeType::LaunchApp => "Launch App".into(),
                                NodeType::CloseApp => "Close App".into(),
                                NodeType::FocusWindow => "Focus Window".into(),
                                NodeType::GetWindowPosition => "Get Window Pos".into(),
                                NodeType::SetWindowPosition => "Set Window Pos".into(),
                                NodeType::ScreenCapture => "Screen Capture".into(),
                                NodeType::SaveScreenshot => "Save Screenshot".into(),
                                NodeType::RegionCapture => "Region Capture".into(),
                                _ => format!("{:?}", node.node_type),
                            };

                            // Create display name: "Type: CustomName" or just "Type"
                            let display = if let Some(ref custom_name) = node.display_name {
                                if !custom_name.is_empty() {
                                    format!("{}: {}", type_name, custom_name)
                                } else {
                                    type_name.clone()
                                }
                            } else {
                                type_name.clone()
                            };

                            // Get header color based on node type (matching canvas)
                            let header_color = match &node.node_type {
                                NodeType::BlueprintFunction { name }
                                    if name.starts_with("Event") =>
                                {
                                    self.editor
                                        .style
                                        .header_colors
                                        .get("Event")
                                        .copied()
                                        .unwrap_or(egui::Color32::from_rgb(180, 50, 50))
                                }
                                NodeType::BlueprintFunction { .. }
                                | NodeType::InputParam
                                | NodeType::OutputParam => self
                                    .editor
                                    .style
                                    .header_colors
                                    .get("Function")
                                    .copied()
                                    .unwrap_or(egui::Color32::from_rgb(50, 100, 180)),
                                // Math Operations
                                NodeType::Add
                                | NodeType::Subtract
                                | NodeType::Multiply
                                | NodeType::Divide
                                | NodeType::Modulo
                                | NodeType::Power
                                | NodeType::Abs
                                | NodeType::Min
                                | NodeType::Max
                                | NodeType::Clamp
                                | NodeType::Random
                                | NodeType::Constant => self
                                    .editor
                                    .style
                                    .header_colors
                                    .get("Math")
                                    .copied()
                                    .unwrap_or(egui::Color32::from_rgb(50, 150, 50)),
                                // Variables
                                NodeType::GetVariable { .. } | NodeType::SetVariable { .. } => self
                                    .editor
                                    .style
                                    .header_colors
                                    .get("Variable")
                                    .copied()
                                    .unwrap_or(egui::Color32::from_rgb(150, 100, 50)),
                                // String Operations
                                NodeType::Concat
                                | NodeType::Split
                                | NodeType::Length
                                | NodeType::Contains
                                | NodeType::Replace
                                | NodeType::Format
                                | NodeType::StringJoin
                                | NodeType::StringBetween
                                | NodeType::ExtractAfter
                                | NodeType::ExtractUntil => self
                                    .editor
                                    .style
                                    .header_colors
                                    .get("String")
                                    .copied()
                                    .unwrap_or(egui::Color32::from_rgb(200, 100, 100)),
                                // Type Conversions
                                NodeType::ToInteger | NodeType::ToFloat | NodeType::ToString => self
                                    .editor
                                    .style
                                    .header_colors
                                    .get("Conversion")
                                    .copied()
                                    .unwrap_or(egui::Color32::from_rgb(150, 200, 100)),
                                // Comparison Operators
                                NodeType::Equals
                                | NodeType::NotEquals
                                | NodeType::GreaterThan
                                | NodeType::GreaterThanOrEqual
                                | NodeType::LessThan
                                | NodeType::LessThanOrEqual => self
                                    .editor
                                    .style
                                    .header_colors
                                    .get("Comparison")
                                    .copied()
                                    .unwrap_or(egui::Color32::from_rgb(150, 100, 200)),
                                // Logic Operators
                                NodeType::And | NodeType::Or | NodeType::Not | NodeType::Xor => self
                                    .editor
                                    .style
                                    .header_colors
                                    .get("Logic")
                                    .copied()
                                    .unwrap_or(egui::Color32::from_rgb(100, 50, 200)),
                                // Control Flow
                                NodeType::Branch
                                | NodeType::ForLoop
                                | NodeType::WhileLoop
                                | NodeType::Sequence
                                | NodeType::Gate
                                | NodeType::Entry => self
                                    .editor
                                    .style
                                    .header_colors
                                    .get("ControlFlow")
                                    .copied()
                                    .unwrap_or(egui::Color32::from_rgb(200, 150, 50)),
                                // Timing
                                NodeType::Delay => self
                                    .editor
                                    .style
                                    .header_colors
                                    .get("Time")
                                    .copied()
                                    .unwrap_or(egui::Color32::from_rgb(100, 200, 100)),
                                // Desktop Input Automation
                                NodeType::Click
                                | NodeType::DoubleClick
                                | NodeType::RightClick
                                | NodeType::MouseMove
                                | NodeType::MouseDown
                                | NodeType::MouseUp
                                | NodeType::Scroll
                                | NodeType::KeyPress
                                | NodeType::KeyDown
                                | NodeType::KeyUp
                                | NodeType::TypeText
                                | NodeType::TypeString
                                | NodeType::HotKey => self
                                    .editor
                                    .style
                                    .header_colors
                                    .get("Input")
                                    .copied()
                                    .unwrap_or(egui::Color32::from_rgb(200, 150, 50)),
                                // I/O Operations
                                NodeType::ReadInput | NodeType::FileRead | NodeType::FileWrite => {
                                    self.editor
                                        .style
                                        .header_colors
                                        .get("IO")
                                        .copied()
                                        .unwrap_or(egui::Color32::from_rgb(100, 150, 200))
                                }
                                // System Control
                                NodeType::RunCommand
                                | NodeType::LaunchApp
                                | NodeType::CloseApp
                                | NodeType::FocusWindow
                                | NodeType::GetWindowPosition
                                | NodeType::SetWindowPosition => self
                                    .editor
                                    .style
                                    .header_colors
                                    .get("System")
                                    .copied()
                                    .unwrap_or(egui::Color32::from_rgb(100, 50, 200)),
                                // Data Operations
                                NodeType::ArrayCreate
                                | NodeType::ArrayPush
                                | NodeType::ArrayPop
                                | NodeType::ArrayGet
                                | NodeType::ArraySet
                                | NodeType::ArrayLength
                                | NodeType::JSONParse
                                | NodeType::JSONStringify
                                | NodeType::HTTPRequest => self
                                    .editor
                                    .style
                                    .header_colors
                                    .get("Data")
                                    .copied()
                                    .unwrap_or(egui::Color32::from_rgb(50, 150, 150)),
                                // Screenshot & Image Tools
                                NodeType::ScreenCapture
                                | NodeType::SaveScreenshot
                                | NodeType::RegionCapture => self
                                    .editor
                                    .style
                                    .header_colors
                                    .get("Screenshot")
                                    .copied()
                                    .unwrap_or(egui::Color32::from_rgb(50, 200, 150)),
                                // Image Recognition
                                NodeType::GetPixelColor
                                | NodeType::FindColor
                                | NodeType::WaitForColor
                                | NodeType::FindImage
                                | NodeType::WaitForImage
                                | NodeType::ImageSimilarity => self
                                    .editor
                                    .style
                                    .header_colors
                                    .get("Recognition")
                                    .copied()
                                    .unwrap_or(egui::Color32::from_rgb(200, 50, 150)),
                                _ => self
                                    .editor
                                    .style
                                    .header_colors
                                    .get("Default")
                                    .copied()
                                    .unwrap_or(egui::Color32::from_rgb(100, 100, 100)),
                            };

                            // Include type_name for sorting by type
                            (node.id, display, node.position, header_color, type_name)
                        })
                        .collect();

                    // Filter by search
                    let search_lower = self.nodes_search_filter.to_lowercase();
                    if !search_lower.is_empty() {
                        node_info.retain(|(_, display, _, _, _)| {
                            display.to_lowercase().contains(&search_lower)
                        });
                    }

                    // Sort
                    match self.nodes_sort_mode {
                        1 => node_info.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase())),
                        2 => node_info.sort_by(|a, b| a.4.to_lowercase().cmp(&b.4.to_lowercase())),
                        _ => {} // Default: no sorting (insertion order)
                    }

                    for (node_id, name, position, header_color, _type_name) in node_info {
                        // Highlight if node is selected
                        let is_selected = self.editor.selected_nodes.contains(&node_id);
                        let fill_color = if is_selected {
                            header_color.gamma_multiply(1.3)
                        } else {
                            header_color
                        };
                        let button = egui::Button::new(
                            egui::RichText::new(&name).color(egui::Color32::WHITE),
                        )
                        .fill(fill_color);

                        if ui.add(button).clicked() {
                            // Select the node and pan to center it
                            self.editor.selected_nodes.clear();
                            self.editor.selected_nodes.insert(node_id);
                            let center = ui.ctx().available_rect().center().to_vec2();
                            // Account for VIRTUAL_OFFSET (5000, 5000) used in coordinate transformation
                            let virtual_offset = egui::Vec2::new(5000.0, 5000.0);
                            let node_pos = egui::Vec2::new(position.0, position.1) + virtual_offset;
                            self.editor.pan = center - node_pos * self.editor.zoom;
                            // Bring to front by updating z_order
                            if let Some(n) = self.graph.nodes.get_mut(&node_id) {
                                n.z_order = self.editor.next_z_order;
                                self.editor.next_z_order += 1;
                            }
                        }
                    }

                    if !self.graph.groups.is_empty() {
                        ui.separator();
                        ui.label("Groups:");
                        for (id, group) in &self.graph.groups {
                            let name = if group.name.is_empty() {
                                "Unnamed Group"
                            } else {
                                &group.name
                            };
                            let bg = egui::Color32::from_rgb(
                                group.color[0],
                                group.color[1],
                                group.color[2],
                            );

                            if ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new(name).color(egui::Color32::WHITE),
                                    )
                                    .fill(bg),
                                )
                                .clicked()
                            {
                                // Pan to group
                                let center = ui.ctx().available_rect().center().to_vec2();
                                let virtual_offset = egui::Vec2::new(5000.0, 5000.0);
                                let group_pos = egui::Vec2::new(group.position.0, group.position.1)
                                    + virtual_offset; // Approx top-left
                                self.editor.pan = center - group_pos * self.editor.zoom;
                            }
                        }
                    }
                });
            });

        // Cursor Info Overlay - floating panel at bottom-right showing cursor position and pixel color
        if self.show_cursor_info {
            // Request continuous repaint when cursor info is shown
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
            
            egui::Window::new("Cursor Info")
                .title_bar(true)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-10.0, -10.0))
                .show(ctx, |ui| {
                    // Get cursor position using enigo
                    use enigo::{Enigo, Settings, Mouse};
                    if let Ok(enigo) = Enigo::new(&Settings::default()) {
                        if let Ok((x, y)) = enigo.location() {
                            ui.horizontal(|ui| {
                                ui.label("Position:");
                                ui.monospace(format!("X: {}, Y: {}", x, y));
                            });
                            
                            // Get pixel color using xcap
                            let color_info = (|| -> Option<(u8, u8, u8, String)> {
                                use xcap::Monitor;
                                let monitors = Monitor::all().ok()?;
                                let monitor = monitors.first()?;
                                let image = monitor.capture_image().ok()?;
                                
                                // Adjust coordinates for monitor bounds
                                let mon_x = monitor.x().ok()?;
                                let mon_y = monitor.y().ok()?;
                                let monitor_x = (x - mon_x) as u32;
                                let monitor_y = (y - mon_y) as u32;

                                
                                if monitor_x < image.width() && monitor_y < image.height() {
                                    let pixel = image.get_pixel(monitor_x, monitor_y);
                                    let r = pixel[0];
                                    let g = pixel[1];
                                    let b = pixel[2];
                                    let hex = format!("#{:02X}{:02X}{:02X}", r, g, b);
                                    Some((r, g, b, hex))
                                } else {
                                    None
                                }
                            })();
                            
                            if let Some((r, g, b, hex)) = color_info {
                                ui.horizontal(|ui| {
                                    ui.label("Color:");
                                    // Show color preview
                                    let (rect, _) = ui.allocate_exact_size(
                                        egui::vec2(16.0, 16.0),
                                        egui::Sense::hover()
                                    );
                                    ui.painter().rect_filled(
                                        rect,
                                        2.0,
                                        egui::Color32::from_rgb(r, g, b)
                                    );
                                    ui.monospace(hex);
                                });
                            }
                        }
                    }
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Undo/Redo Input
            if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z)) {
                if let Some(prev) = self.undo_stack.undo() {
                    self.graph = prev;
                }
            }
            if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Y)) {
                // Windows/Linux Redo
                if let Some(next) = self.undo_stack.redo() {
                    self.graph = next;
                }
            }
            // Mac usually uses Cmd+Shift+Z for redo
            if ui.input(|i| i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z))
            {
                if let Some(next) = self.undo_stack.redo() {
                    self.graph = next;
                }
            }

            self.editor.show(ui, &mut self.graph, &mut self.undo_stack);
        });

        // Context menu
        // Since we are custom, we can check for right click anywhere
        if ctx.input(|i| i.pointer.secondary_clicked()) && !ctx.is_using_pointer() {
            // Open context menu
        }
    }
}
