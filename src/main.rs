mod node_types;
mod graph;
mod executor;
mod editor;

use eframe::egui;
use graph::{BlueprintGraph, Node, Port};
use node_types::{NodeType, DataType};
use editor::GraphEditor;
use uuid::Uuid;

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
}

impl Default for MyApp {
    fn default() -> Self {
        let mut app = Self {
            graph: BlueprintGraph::default(),
            editor: GraphEditor::default(),
            logs: Vec::new(),
        };
        // Add some test nodes
        app.add_test_nodes();
        app
    }
}

impl MyApp {
    fn add_test_nodes(&mut self) {
        use crate::graph::VariableValue;
        let id1 = Uuid::new_v4();
        self.graph.nodes.insert(id1, Node {
            id: id1,
            node_type: NodeType::BlueprintFunction { name: "Event Tick".into() },
            position: (100.0, 100.0),
            inputs: vec![],
            outputs: vec![Port { name: "Next".into(), data_type: DataType::ExecutionFlow, default_value: VariableValue::None }],
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
                
                 if ui.button("Save").clicked() {
                    if let Ok(json) = serde_json::to_string(&self.graph) {
                        let _ = std::fs::write("blueprint.json", json);
                    }
                }
                if ui.button("Load").clicked() {
                    if let Ok(json) = std::fs::read_to_string("blueprint.json") {
                        if let Ok(graph) = serde_json::from_str(&json) {
                            self.graph = graph;
                        }
                    }
                }
                 if ui.button("Run").clicked() {
                   log::info!("Running graph...");
                   self.logs.clear();
                   // We need to move logs into closure, or use RefCell if we needed interior mutability,
                   // but here we can just collect them? No, Interpreter is synchronous.
                   // We can pass a closure that pushes to a local vec, then extend self.logs.
                   let mut current_logs = Vec::new();
                   if let Err(e) = executor::Interpreter::run(&self.graph, |msg| current_logs.push(msg)) {
                       log::error!("Execution error: {}", e);
                       current_logs.push(format!("Error: {}", e));
                   }
                   self.logs.extend(current_logs);
                }
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").resizable(true).min_height(100.0).show(ctx, |ui| {
            ui.heading("Output Log");
            egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                for log in &self.logs {
                    ui.label(log);
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.editor.show(ui, &mut self.graph);
        });
        
        // Context menu
        // Since we are custom, we can check for right click anywhere
         if ctx.input(|i| i.pointer.secondary_clicked()) && !ctx.is_using_pointer() {
              // Open context menu
         }
    }
}
