use eframe::egui;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};
use uuid::Uuid;
use super::graph::{BlueprintGraph, Node};
use super::node_types::DataType;

pub struct GraphEditor {
    pub pan: Vec2,
    pub zoom: f32,
    pub dragging_node: Option<Uuid>,
    pub connection_start: Option<(Uuid, String, bool)>,
    pub node_finder: Option<Pos2>,
    pub node_finder_query: String,
    pub selected_nodes: std::collections::HashSet<Uuid>,
    pub selection_box: Option<Rect>,
}

impl Default for GraphEditor {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.0,
            dragging_node: None,
            connection_start: None,
            node_finder: None,
            node_finder_query: String::new(),
            selected_nodes: std::collections::HashSet::new(),
            selection_box: None,
        }
    }
}

impl GraphEditor {
    pub fn show(&mut self, ui: &mut egui::Ui, graph: &mut BlueprintGraph) {
        let clip_rect = ui.max_rect();
        let pointer_in_bounds = ui.rect_contains_pointer(clip_rect);

        ui.input(|i| {
            // Pan with Middle Mouse or Alt + Left Mouse
            if i.pointer.middle_down() || (i.modifiers.alt && i.pointer.primary_down()) {
                 self.pan += i.pointer.delta();
            }
            // Zoom
            if let Some(_hovered) = i.pointer.hover_pos() {
                if pointer_in_bounds {
                     let zoom_delta = i.zoom_delta();
                     if zoom_delta != 1.0 {
                         // TODO: Zoom towards pointer
                         self.zoom *= zoom_delta;
                     }
                }
            }
        });

        let painter = ui.painter();
        painter.rect_filled(clip_rect, 0.0, Color32::from_gray(32)); // Background

        // Draw connections
        self.draw_connections(ui, graph);
        
        // Draw connection in progress
        if let Some((node_id, port_name, is_input)) = &self.connection_start {
            if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                 let start_pos = self.get_port_screen_pos(ui, &graph.nodes, *node_id, port_name, *is_input, ui.max_rect().min);
                 self.draw_bezier(ui, start_pos, pointer_pos, Color32::WHITE);
            }
        }

        // Draw nodes
        let mut node_move_delta = Vec2::ZERO;
        let mut node_to_move = None; // Primary node being dragged
        let mut connect_event = None;
        let mut click_on_node = false;

        for node in graph.nodes.values() {
            let (drag_delta, start_connect, clicked) = self.draw_node(ui, node);
            if clicked {
                click_on_node = true;
                // Handle Selection Click
                if ui.input(|i| i.modifiers.shift) {
                    if self.selected_nodes.contains(&node.id) {
                        self.selected_nodes.remove(&node.id);
                    } else {
                        self.selected_nodes.insert(node.id);
                    }
                } else if !self.selected_nodes.contains(&node.id) {
                     // If clicking a new node without shift, select only it
                     self.selected_nodes.clear();
                     self.selected_nodes.insert(node.id);
                }
            }
            
            if drag_delta != Vec2::ZERO {
                node_move_delta = drag_delta;
                node_to_move = Some(node.id);
                
                // If dragging a node that isn't selected, select it (and clear others if not shift)
                if !self.selected_nodes.contains(&node.id) {
                     if !ui.input(|i| i.modifiers.shift) {
                        self.selected_nodes.clear();
                     }
                     self.selected_nodes.insert(node.id);
                }
            }
            if let Some(event) = start_connect {
                connect_event = Some(event);
            }
        }

        // Apply deferred updates (Batch Move)
        if node_move_delta != Vec2::ZERO {
            for id in &self.selected_nodes {
                if let Some(node) = graph.nodes.get_mut(id) {
                    node.position.0 += node_move_delta.x;
                    node.position.1 += node_move_delta.y;
                }
            }
        }
        
        // Background Selection Box Logic
        // If clicking on background (not node, not connection start) and dragging
        if ui.input(|i| i.pointer.primary_down()) 
            && node_to_move.is_none() 
            && connect_event.is_none()
            && self.connection_start.is_none() 
            && !ui.input(|i| i.modifiers.alt) // Alt is for Pan
            && !click_on_node
        {
             // Start or update selection box
             if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                 if let Some(mut rect) = self.selection_box {
                     rect.max = pointer_pos;
                     self.selection_box = Some(rect);
                 } else if ui.input(|i| i.pointer.primary_pressed()) {
                     self.selection_box = Some(Rect::from_min_max(pointer_pos, pointer_pos));
                     if !ui.input(|i| i.modifiers.shift) {
                         self.selected_nodes.clear();
                     }
                 }
             }
        }
        
        if ui.input(|i| i.pointer.primary_released()) {
            if let Some(rect) = self.selection_box {
                // Finalize selection
                // Check intersection with nodes
                 // We need to check if node screen rect intersects with selection box (screen rect)
                 // Wait, selection box is in screen coords? Yes (pointer_pos).
                 // Node drawing uses screen coords.
                 
                 // We need to iterate again or check positions.
                 // Since we don't store screen rects, we calculate them again.
                 let selection_rect = Rect::from_min_max(rect.min.min(rect.max), rect.max.max(rect.min)); // Normalize
                 
                 for node in graph.nodes.values() {
                     let node_pos = Pos2::new(node.position.0, node.position.1);
                     let screen_pos = self.to_screen(node_pos, clip_rect.min);
                     
                     // Approximate Size
                     let title = format!("{:?}", node.node_type);
                     let title_galley = ui.painter().layout(title, egui::FontId::proportional(14.0 * self.zoom), Color32::WHITE, f32::INFINITY);
                     let min_width = 150.0 * self.zoom;
                     let node_width = min_width.max(title_galley.rect.width() + 20.0 * self.zoom);
                     let node_screen_rect = Rect::from_min_size(screen_pos, Vec2::new(node_width, 100.0 * self.zoom));
                     
                     if selection_rect.intersects(node_screen_rect) {
                         self.selected_nodes.insert(node.id);
                     }
                 }
            }
            self.selection_box = None;
        }

        // Draw Selection Box
        if let Some(rect) = self.selection_box {
             let r = Rect::from_min_max(rect.min.min(rect.max), rect.max.max(rect.min));
             ui.painter().rect_stroke(r, 0.0, Stroke::new(1.0, Color32::WHITE), egui::StrokeKind::Middle);
             ui.painter().rect_filled(r, 0.0, Color32::from_white_alpha(32));
        }

        // Delete Selected
        if ui.input(|i| i.key_pressed(egui::Key::Delete)) {
            for id in &self.selected_nodes {
                graph.nodes.remove(id);
                // Also remove connections
                graph.connections.retain(|c| c.from_node != *id && c.to_node != *id);
            }
            self.selected_nodes.clear();
        }
        
        // Right click wire deletion
        if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
             if ui.input(|i| i.pointer.secondary_clicked()) {
                 graph.connections.retain(|c| {
                      let p1 = self.get_port_screen_pos(ui, &graph.nodes, c.from_node, &c.from_port, false, clip_rect.min);
                      let p2 = self.get_port_screen_pos(ui, &graph.nodes, c.to_node, &c.to_port, true, clip_rect.min);
                      // Simple distance check to line segment or bezier? 
                      // Benzier is hard. Let's approx with distance to p1 or p2 for now for simplicity, or middle.
                      // Proper hit testing on bezier is mathy.
                      // Let's use a simple distance to the midpoint.
                      let mid = p1 + (p2 - p1) * 0.5;
                      if mid.distance(pointer_pos) < 20.0 {
                          return false; // Delete
                      }
                      true
                 });
             }
        }

        if let Some((id, port, is_input)) = connect_event {
             // If we are already dragging a connection, try to complete it
             if let Some((start_id, start_port, start_is_input)) = &self.connection_start {
                 if *start_is_input != is_input && *start_id != id {
                     // Valid(ish) connection
                     // TODO: Check types
                     let (from, from_port, to, to_port) = if *start_is_input {
                         (id, port, *start_id, start_port.clone())
                     } else {
                         (*start_id, start_port.clone(), id, port)
                     };
                     
                     graph.connections.push(crate::graph::Connection {
                         from_node: from,
                         from_port: from_port,
                         to_node: to,
                         to_port: to_port,
                     });
                     self.connection_start = None;
                 } else {
                     // Cancel or restart
                     self.connection_start = Some((id, port, is_input));
                 }
             } else {
                 self.connection_start = Some((id, port, is_input));
             }
        }
        
        // Stop connection drag if mouse released (and not handled) OR Escape pressed OR Right Click
         if ui.input(|i| i.pointer.any_released() || i.key_pressed(egui::Key::Escape) || i.pointer.secondary_clicked()) && self.connection_start.is_some() {
             // Cancel connection
             self.connection_start = None;
         }
         
         if ui.input(|i| i.pointer.primary_released()) {
             // If we clicked on nothing, clear connection
         }
         
         // Quick Add Menu (Spacebar)
         if ui.input(|i| i.key_pressed(egui::Key::Space)) && self.node_finder.is_none() {
             if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                 self.node_finder = Some(pointer_pos);
                 self.node_finder_query.clear();
             }
         }
         
         if let Some(pos) = self.node_finder {
             let mut open = true;
             egui::Window::new("Add Node")
                 .id(egui::Id::new("node_finder"))
                 .fixed_pos(pos)
                 .pivot(egui::Align2::LEFT_TOP)
                 .collapsible(false)
                 .resizable(false)
                 .title_bar(false)
                 .open(&mut open)
                 .show(ui.ctx(), |ui| {
                     ui.text_edit_singleline(&mut self.node_finder_query).request_focus();
                     
                     let options = vec![
                         ("Event Tick", crate::node_types::NodeType::BlueprintFunction { name: "Event Tick".into() }),
                         ("Print String", crate::node_types::NodeType::BlueprintFunction { name: "Print String".into() }),
                         ("Add", crate::node_types::NodeType::Add),
                         ("Subtract", crate::node_types::NodeType::Subtract),
                         // Add more options
                     ];
                     
                     for (label, node_type) in options {
                         if self.node_finder_query.is_empty() || label.to_lowercase().contains(&self.node_finder_query.to_lowercase()) {
                             if ui.button(label).clicked() {
                                 // Add Node
                                 let id = Uuid::new_v4();
                                 // We need to define ports for these.
                                 // This is where a factory/template system is needed.
                                 // For now, hardcode logic or use a helper.
                                 let (inputs, outputs) = Self::get_ports_for_type(&node_type);
                                 
                                 // Convert screen pos back to graph pos
                                 let graph_pos = self.from_screen(pos, clip_rect.min);

                                 graph.nodes.insert(id, Node {
                                     id,
                                     node_type,
                                     position: (graph_pos.x, graph_pos.y),
                                     inputs,
                                     outputs,
                                 });
                                 self.node_finder = None;
                             }
                         }
                     }
                 });
             
             if !open || ui.input(|i| i.key_pressed(egui::Key::Escape)) || ui.input(|i| i.pointer.primary_clicked() && !ui.rect_contains_pointer(ui.max_rect())) {
                 // Close if clicked outside or pressed escape (Window built-in closing handles outer clicks usually? No, egui windows are non-modal)
                 // Just check open flag or escape
                 self.node_finder = None;
             }
         }
    }

    fn to_screen(&self, pos: Pos2, offset: Pos2) -> Pos2 {
        (pos.to_vec2() * self.zoom + self.pan + offset.to_vec2()).to_pos2()
    }
    
    fn from_screen(&self, pos: Pos2, offset: Pos2) -> Pos2 {
        ((pos.to_vec2() - self.pan - offset.to_vec2()) / self.zoom).to_pos2()
    }
    
    fn get_ports_for_type(node_type: &crate::node_types::NodeType) -> (Vec<super::graph::Port>, Vec<super::graph::Port>) {
        use crate::graph::Port;
        use crate::node_types::{NodeType, DataType};
        match node_type {
            NodeType::BlueprintFunction { name } if name == "Event Tick" => (
                vec![], 
                vec![Port { name: "Next".into(), data_type: DataType::ExecutionFlow }]
            ),
            NodeType::BlueprintFunction { name } if name == "Print String" => (
                vec![
                    Port { name: "In".into(), data_type: DataType::ExecutionFlow },
                    Port { name: "String".into(), data_type: DataType::String }
                ],
                vec![Port { name: "Next".into(), data_type: DataType::ExecutionFlow }]
            ),
            NodeType::Add => (
                 vec![
                    Port { name: "A".into(), data_type: DataType::Float }, // Simplifying to Float
                    Port { name: "B".into(), data_type: DataType::Float }
                ],
                vec![Port { name: "Out".into(), data_type: DataType::Float }]
            ),
            NodeType::Subtract => (
                 vec![
                    Port { name: "A".into(), data_type: DataType::Float },
                    Port { name: "B".into(), data_type: DataType::Float }
                ],
                vec![Port { name: "Out".into(), data_type: DataType::Float }]
            ),
            _ => (vec![], vec![])
        }
    }

    fn draw_node(&mut self, ui: &mut egui::Ui, node: &Node) -> (Vec2, Option<(Uuid, String, bool)>, bool) {
        let node_pos = Pos2::new(node.position.0, node.position.1);
        let screen_pos = self.to_screen(node_pos, ui.max_rect().min);
        
        let mut drag_delta = Vec2::ZERO;
        let mut connect_event = None; // (NodeId, PortName, is_input)

        // Calculate size based on title
        let title = format!("{:?}", node.node_type);
        let title_galley = ui.painter().layout(title, egui::FontId::proportional(14.0 * self.zoom), Color32::WHITE, f32::INFINITY);
        let min_width = 150.0 * self.zoom;
        let node_width = min_width.max(title_galley.rect.width() + 20.0 * self.zoom); // Padding

        let node_rect = Rect::from_min_size(screen_pos, Vec2::new(node_width, 100.0 * self.zoom)); // Approx height
        
        // Node Window/Area
        let response = ui.allocate_rect(node_rect, Sense::click_and_drag());
        if response.dragged() {
            drag_delta = response.drag_delta() / self.zoom;
        }
        let clicked = response.clicked();

        // Selection Highlight
        if self.selected_nodes.contains(&node.id) {
             ui.painter().rect_stroke(node_rect.expand(2.0), 3.0, Stroke::new(2.0, Color32::YELLOW), egui::StrokeKind::Middle);
        }

        // Draw Node Background
        ui.painter().rect_filled(node_rect, 5.0, Color32::from_gray(64));
        ui.painter().rect_stroke(node_rect, 5.0, Stroke::new(1.0, Color32::BLACK), egui::StrokeKind::Middle);

        // Header
        let header_rect = Rect::from_min_max(node_rect.min, Pos2::new(node_rect.max.x, node_rect.min.y + 20.0 * self.zoom));
        ui.painter().rect_filled(header_rect, 5.0, Color32::from_rgb(100, 100, 200));
        ui.painter().galley(header_rect.center() - title_galley.rect.size() * 0.5, title_galley, Color32::WHITE);

        // Ports
        // Inputs
        let mut y_offset = 30.0 * self.zoom;
        for input in &node.inputs {
            let port_pos = screen_pos + Vec2::new(0.0, y_offset);
            let port_rect = Rect::from_center_size(port_pos, Vec2::splat(10.0 * self.zoom));
            
            ui.painter().circle_filled(port_pos, 5.0 * self.zoom, self.get_type_color(&input.data_type));
            
            // Name
            ui.painter().text(
                 port_pos + Vec2::new(12.0 * self.zoom, 0.0),
                 egui::Align2::LEFT_CENTER,
                 &input.name,
                 egui::FontId::proportional(12.0 * self.zoom),
                 Color32::WHITE
            );

            // Interaction
            let port_response = ui.interact(port_rect, ui.id().with(node.id).with(&input.name).with("in"), Sense::click());
            if port_response.clicked() {
                connect_event = Some((node.id, input.name.clone(), true));
            }

            y_offset += 20.0 * self.zoom;
        }
        
        // Outputs
        let mut y_offset = 30.0 * self.zoom;
        for output in &node.outputs {
            let port_pos = screen_pos + Vec2::new(node_rect.width(), y_offset);
            let port_rect = Rect::from_center_size(port_pos, Vec2::splat(10.0 * self.zoom));
            
            ui.painter().circle_filled(port_pos, 5.0 * self.zoom, self.get_type_color(&output.data_type));
             // Name
            ui.painter().text(
                 port_pos - Vec2::new(12.0 * self.zoom, 0.0),
                 egui::Align2::RIGHT_CENTER,
                 &output.name,
                 egui::FontId::proportional(12.0 * self.zoom),
                 Color32::WHITE
            );
            
             // Interaction
            let port_response = ui.interact(port_rect, ui.id().with(node.id).with(&output.name).with("out"), Sense::click());
            if port_response.clicked() {
                connect_event = Some((node.id, output.name.clone(), false));
            }

            y_offset += 20.0 * self.zoom;
        }

        (drag_delta, connect_event, clicked)
    }
    
    fn draw_connections(&self, ui: &egui::Ui, graph: &BlueprintGraph) {
        let offset = ui.max_rect().min;
        for conn in &graph.connections {
            let p1 = self.get_port_screen_pos(ui, &graph.nodes, conn.from_node, &conn.from_port, false, offset);
            let p2 = self.get_port_screen_pos(ui, &graph.nodes, conn.to_node, &conn.to_port, true, offset);
            self.draw_bezier(ui, p1, p2, Color32::WHITE); 
        }
    }
    
    fn draw_bezier(&self, ui: &egui::Ui, p1: Pos2, p2: Pos2, color: Color32) {
        let p1_vec = p1.to_vec2();
        let p2_vec = p2.to_vec2();
        let control_scale = (p2_vec.x - p1_vec.x).abs().max(50.0) * 0.5;
        let c1 = Pos2::new(p1.x + control_scale, p1.y);
        let c2 = Pos2::new(p2.x - control_scale, p2.y);
        
        let curve = egui::epaint::CubicBezierShape::from_points_stroke(
            [p1, c1, c2, p2],
            false,
            Color32::TRANSPARENT,
            Stroke::new(2.0 * self.zoom, color),
        );
        ui.painter().add(curve);
    }

    fn get_port_screen_pos(&self, ui: &egui::Ui, nodes: &std::collections::HashMap<Uuid, Node>, node_id: Uuid, port_name: &str, is_input: bool, offset: Pos2) -> Pos2 {
         if let Some(node) = nodes.get(&node_id) {
             let node_pos = Pos2::new(node.position.0, node.position.1);
             let screen_pos = self.to_screen(node_pos, offset);
             
             // Recalculate Width (Duplicated logic, should extract to method)
             let title = format!("{:?}", node.node_type);
             let title_width = ui.painter().layout(title, egui::FontId::proportional(14.0 * self.zoom), Color32::WHITE, f32::INFINITY).rect.width();
             let min_width = 150.0 * self.zoom;
             let node_width = min_width.max(title_width + 20.0 * self.zoom);

             let index = if is_input {
                 node.inputs.iter().position(|p| p.name == port_name).unwrap_or(0)
             } else {
                 node.outputs.iter().position(|p| p.name == port_name).unwrap_or(0)
             };
             
             let y = 30.0 + (index as f32 * 20.0);
             let x = if is_input { 0.0 } else { node_width / self.zoom }; // Need unzoomed width? No, node_width IS zoomed.
             // Wait, node_width IS zoomed because it uses * self.zoom.
             // screen_pos is zoomed.
             // So we add node_width directly? 
             // x=0 is left. x=width is right.
             // But screen_pos is the top-left of the node.
             // yes.
             
             return screen_pos + Vec2::new(x, y * self.zoom);
         }
         Pos2::ZERO
    }
    
    fn get_type_color(&self, dt: &DataType) -> Color32 {
        match dt {
            DataType::ExecutionFlow => Color32::WHITE,
            DataType::Boolean => Color32::RED,
            DataType::Float => Color32::GREEN,
            DataType::Integer => Color32::LIGHT_BLUE,
            DataType::String => Color32::KHAKI,
            DataType::Vector3 => Color32::YELLOW,
            DataType::Custom(_) => Color32::GRAY,
        }
    }
}
