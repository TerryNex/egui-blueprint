mod node_types;
mod graph;
mod executor;
mod editor;
mod history;

use eframe::egui;
use graph::{BlueprintGraph, Node, Port};
use node_types::{NodeType, DataType};
use editor::GraphEditor;
use uuid::Uuid;
use history::UndoStack;
use std::sync::mpsc::Receiver;
use chrono::Local;

fn main() -> eframe::Result<()> {
    env_logger::init(); 
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0]),
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
    start_time: std::time::Instant,
    undo_stack: UndoStack,
    log_receiver: Option<Receiver<String>>,
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
            start_time: std::time::Instant::now(),
            undo_stack: UndoStack::default(),
            log_receiver: None,
        };
        let _ = std::fs::create_dir_all("scripts");
        // Add some test nodes
        app.load_settings();
        app.add_test_nodes();
        app
    }
}

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
struct AppSettings {
    style: editor::EditorStyle,
    history_max_records: usize,
}

impl MyApp {
    fn load_settings(&mut self) {
        if let Ok(json) = std::fs::read_to_string("settings.json") {
            if let Ok(settings) = serde_json::from_str::<AppSettings>(&json) {
                self.editor.style = settings.style;
                self.undo_stack.max_records = settings.history_max_records;
                self.logs.push("[System] Settings loaded.".to_string());
            }
        }
    }
    
    fn save_settings(&self) {
        let settings = AppSettings {
            style: self.editor.style.clone(),
            history_max_records: self.undo_stack.max_records,
        };
        if let Ok(json) = serde_json::to_string_pretty(&settings) {
            let _ = std::fs::write("settings.json", json);
        }
    }
    
    fn add_test_nodes(&mut self) {
        use crate::graph::VariableValue;
        let id1 = Uuid::new_v4();
        self.graph.nodes.insert(id1, Node {
            id: id1,
            node_type: NodeType::BlueprintFunction { name: "Event Tick".into() },
            position: (100.0, 100.0),
            inputs: vec![],
            outputs: vec![Port { name: "Next".into(), data_type: DataType::ExecutionFlow, default_value: VariableValue::None }],
            z_order: 0,
            display_name: None,
        });
        
        let id2 = Uuid::new_v4();
        self.graph.nodes.insert(id2, Node {
            id: id2,
            node_type: NodeType::BlueprintFunction { name: "Print String".into() },
            position: (400.0, 100.0),
            inputs: vec![
                Port { name: "In".into(), data_type: DataType::ExecutionFlow, default_value: VariableValue::None },
                Port { name: "String".into(), data_type: DataType::String, default_value: VariableValue::String("Hello".into()) },
            ],
            outputs: vec![Port { name: "Next".into(), data_type: DataType::ExecutionFlow, default_value: VariableValue::None }],
            z_order: 1,
            display_name: None,
        });
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Blueprint Editor (Custom)");
                ui.separator();
                if ui.button("Zoom In").clicked() { self.editor.zoom *= 1.1; }
                if ui.button("Zoom Out").clicked() { self.editor.zoom /= 1.1; }
                if ui.button("Reset").clicked() { self.editor.pan = egui::Vec2::ZERO; self.editor.zoom = 1.0; }
                ui.separator();
                
                if ui.button("Style").clicked() {
                     self.editor.show_settings = !self.editor.show_settings;
                }
                
                ui.label("Script:");
                ui.text_edit_singleline(&mut self.script_name);
                
                if ui.button("Save").clicked() {
                    let name = if self.script_name.ends_with(".json") { self.script_name.clone() } else { format!("{}.json", self.script_name) };
                    if let Ok(json) = serde_json::to_string(&self.graph) {
                        let _ = std::fs::write(format!("scripts/{}", name), json);
                        
                        // Save History
                        if let Ok(history_json) = serde_json::to_string(&self.undo_stack) {
                            let history_name = format!("{}.history", name.trim_end_matches(".json"));
                            let _ = std::fs::write(format!("scripts/{}", history_name), history_json);
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
                   self.logs.push("[System] Async Execution Started".to_string());
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
                                          if ui.button("🗑").on_hover_text("Delete Script").clicked() {
                                              if let Err(e) = std::fs::remove_file(entry.path()) {
                                                  log::error!("Failed to delete script: {}", e);
                                              }
                                              // Also try to delete history
                                              let _ = std::fs::remove_file(entry.path().with_extension("history"));
                                          }
                                          if ui.button(&name).clicked() {
                                              if let Ok(json) = std::fs::read_to_string(entry.path()) {
                                                  loaded_script = Some((json, name.trim_end_matches(".json").to_string()));
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

        egui::TopBottomPanel::bottom("bottom_panel").resizable(true).min_height(100.0).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Output Log");
                if ui.button("Clear").clicked() {
                    self.logs.clear();
                }
                ui.label(format!("Count: {}", self.logs.len()));
            });
            egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                for log in &self.logs {
                    let mut job = egui::text::LayoutJob::default();
                    let mut current_segment = String::new();
                    let mut in_var = false;
                    
                    for c in log.chars() {
                        if c == '{' && !in_var {
                            if !current_segment.is_empty() {
                                job.append(&current_segment, 0.0, egui::TextFormat { color: egui::Color32::WHITE, ..Default::default() });
                                current_segment.clear();
                            }
                            in_var = true;
                            current_segment.push(c);
                        } else if c == '}' && in_var {
                            current_segment.push(c);
                            job.append(&current_segment, 0.0, egui::TextFormat { color: egui::Color32::from_rgb(100, 255, 100), ..Default::default() });
                            current_segment.clear();
                            in_var = false;
                        } else {
                            current_segment.push(c);
                        }
                    }
                    if !current_segment.is_empty() {
                         let color = if in_var { egui::Color32::from_rgb(100, 255, 100) } else { egui::Color32::WHITE };
                         job.append(&current_segment, 0.0, egui::TextFormat { color, ..Default::default() });
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
                    let node_info: Vec<_> = self.graph.nodes.values()
                        .map(|node| {
                            let name = if let Some(ref custom_name) = node.display_name {
                                if custom_name.is_empty() {
                                    match &node.node_type {
                                        NodeType::BlueprintFunction { name } => name.clone(),
                                        NodeType::GetVariable { name } => format!("Get {}", name),
                                        NodeType::SetVariable { name } => format!("Set {}", name),
                                        other => format!("{:?}", other),
                                    }
                                } else {
                                    format!("{:?}: {}", node.node_type, custom_name)
                                }
                            } else {
                                match &node.node_type {
                                    NodeType::BlueprintFunction { name } => name.clone(),
                                    NodeType::GetVariable { name } => format!("Get {}", name),
                                    NodeType::SetVariable { name } => format!("Set {}", name),
                                    NodeType::Add => "Add".into(),
                                    NodeType::Subtract => "Subtract".into(),
                                    NodeType::Multiply => "Multiply".into(),
                                    NodeType::Divide => "Divide".into(),
                                    NodeType::ToInteger => "To Integer".into(),
                                    NodeType::ToFloat => "To Float".into(),
                                    NodeType::ToString => "To String".into(),
                                    NodeType::Branch => "Branch".into(),
                                    NodeType::Entry => "Entry".into(),
                                    NodeType::Equals => "Equals".into(),
                                    NodeType::GreaterThan => "GreaterThan".into(),
                                    NodeType::LessThan => "LessThan".into(),
                                    _ => format!("{:?}", node.node_type),
                                }
                            };
                            (node.id, name, node.position)
                        })
                        .collect();

                    for (node_id, name, position) in node_info {
                         // Highlight if node is selected
                         let is_selected = self.editor.selected_nodes.contains(&node_id);
                         let button = egui::Button::new(&name)
                             .fill(if is_selected { egui::Color32::from_rgb(80, 80, 120) } else { egui::Color32::from_gray(48) });
                         
                         if ui.add(button).clicked() {
                             // Select the node and pan to center it
                             self.editor.selected_nodes.clear();
                             self.editor.selected_nodes.insert(node_id);
                             let center = ui.ctx().screen_rect().center().to_vec2();
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
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Undo/Redo Input
            if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z)) {
                if let Some(prev) = self.undo_stack.undo() {
                    self.graph = prev;
                }
            }
            if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Y)) { // Windows/Linux Redo
                 if let Some(next) = self.undo_stack.redo() {
                    self.graph = next;
                }
            }
             // Mac usually uses Cmd+Shift+Z for redo
            if ui.input(|i| i.modifiers.command && i.modifiers.shift && i.key_pressed(egui::Key::Z)) {
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
