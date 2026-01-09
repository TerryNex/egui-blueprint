mod editor;
mod executor;
mod graph;
mod history;
mod node_types;

use chrono::Local;
use editor::GraphEditor;
use eframe::egui;
use graph::{BlueprintGraph, Node, Port};
use history::UndoStack;
use node_types::{DataType, NodeType};
use std::sync::mpsc::Receiver;
use sysinfo::{Pid, ProcessesToUpdate, System};
use uuid::Uuid;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "egui Blueprint Node Editor",
        options,
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
    log_receiver: Option<Receiver<String>>,
    // Debug window state
    system: System,
    frame_times: Vec<f32>,
    last_frame_time: std::time::Instant,
}

impl Default for MyApp {
    fn default() -> Self {
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
            system: System::new_all(),
            frame_times: Vec::with_capacity(120),
            last_frame_time: std::time::Instant::now(),
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
            },
        );
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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

                if ui.button("Style").clicked() {
                    self.editor.show_settings = !self.editor.show_settings;
                }

                ui.label("Script:");
                ui.text_edit_singleline(&mut self.script_name);

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
                if ui.button("Run").clicked() {
                    log::info!("Running graph (async)...");
                    self.start_time = std::time::Instant::now();
                    self.log_receiver = Some(executor::Interpreter::run_async(&self.graph));
                    self.logs
                        .push("[System] Async Execution Started".to_string());
                }
                ui.separator();
                if ui.button("Debug").clicked() {
                    self.show_debug_window = !self.show_debug_window;
                }
            });
        });

        // Drain logs from async threads
        if let Some(rx) = &self.log_receiver {
            while let Ok(msg) = rx.try_recv() {
                let now = Local::now();
                let time_str = now.format("%H:%M:%S").to_string();
                self.logs.push(format!("[{}] {}", time_str, msg));
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
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Collect node info first to avoid borrow conflicts
                    let node_info: Vec<_> = self
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
                                NodeType::Sequence => "Sequence".into(),
                                NodeType::Gate => "Gate".into(),
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
                                NodeType::ReadInput => "Read Input".into(),
                                NodeType::FileRead => "File Read".into(),
                                NodeType::FileWrite => "File Write".into(),
                                // System Control
                                NodeType::RunCommand => "Run Command".into(),
                                NodeType::LaunchApp => "Launch App".into(),
                                NodeType::CloseApp => "Close App".into(),
                                NodeType::FocusWindow => "Focus Window".into(),
                                NodeType::GetWindowPosition => "Get Window Pos".into(),
                                NodeType::SetWindowPosition => "Set Window Pos".into(),
                                NodeType::ScreenCapture => "Screen Capture".into(),
                                NodeType::SaveScreenshot => "Save Screenshot".into(),
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
                                NodeType::BlueprintFunction { .. } => self
                                    .editor
                                    .style
                                    .header_colors
                                    .get("Function")
                                    .copied()
                                    .unwrap_or(egui::Color32::from_rgb(50, 100, 200)),
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
                                    .unwrap_or(egui::Color32::from_rgb(50, 150, 100)),
                                NodeType::GetVariable { .. } | NodeType::SetVariable { .. } => self
                                    .editor
                                    .style
                                    .header_colors
                                    .get("Variable")
                                    .copied()
                                    .unwrap_or(egui::Color32::from_rgb(150, 100, 50)),
                                _ => self
                                    .editor
                                    .style
                                    .header_colors
                                    .get("Default")
                                    .copied()
                                    .unwrap_or(egui::Color32::from_rgb(100, 100, 100)),
                            };

                            (node.id, display, node.position, header_color)
                        })
                        .collect();

                    for (node_id, name, position, header_color) in node_info {
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
