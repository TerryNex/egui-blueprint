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
        let pointer_pos = ui.ctx().pointer_latest_pos();

        let mut input_escape = false;
        let mut input_delete = false;
        let mut input_primary_down = false;
        let mut input_primary_pressed = false;
        let mut input_primary_released = false;
        let mut input_secondary_clicked = false;
        let mut input_any_released = false;
        let mut input_modifiers = egui::Modifiers::default();
        let mut input_space = false;

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
                         self.zoom *= zoom_delta;
                     }
                }
            }
            
            input_escape = i.key_pressed(egui::Key::Escape);
            input_delete = i.key_pressed(egui::Key::Delete);
            input_primary_down = i.pointer.primary_down();
            input_primary_pressed = i.pointer.primary_pressed();
            input_primary_released = i.pointer.primary_released();
            input_secondary_clicked = i.pointer.secondary_clicked();
            input_any_released = i.pointer.any_released();
            input_modifiers = i.modifiers;
            input_space = i.key_pressed(egui::Key::Space);
        });

        // Pre-calculate node sizes (needed for both drawing and connection lines)
        let mut node_sizes = std::collections::HashMap::new();
        for node in graph.nodes.values() {
             node_sizes.insert(node.id, self.get_node_size(ui, node, &graph.connections));
        }

        let painter = ui.painter();
        painter.rect_filled(clip_rect, 0.0, Color32::from_gray(32)); // Background

        // Draw connections
        self.draw_connections(ui, graph, &node_sizes);
        
        // Draw connection in progress
        if let Some((node_id, port_name, is_input)) = &self.connection_start {
            if let Some(pos) = pointer_pos {
                 let start_pos = self.get_port_screen_pos(ui, &graph.nodes, *node_id, port_name, *is_input, clip_rect.min, &node_sizes);
                 self.draw_bezier(ui, start_pos, pos, Color32::WHITE);
            }
        }

        // Draw nodes
        let mut node_move_delta = Vec2::ZERO;
        let mut node_to_move = None;
        let mut connect_event = None;
        let mut interaction_consumed = false;

        for node in graph.nodes.values_mut() {
            let (drag_delta, start_connect, clicked, clicked_port, pressed) = self.draw_node(ui, node, input_primary_pressed, input_primary_released, &graph.connections, &node_sizes);
            
            if pressed || clicked_port {
                interaction_consumed = true;
            }

            if clicked && !clicked_port {
                // Handle Selection Click
                if input_modifiers.shift {
                    if self.selected_nodes.contains(&node.id) {
                        self.selected_nodes.remove(&node.id);
                    } else {
                        self.selected_nodes.insert(node.id);
                    }
                } else if !self.selected_nodes.contains(&node.id) {
                     self.selected_nodes.clear();
                     self.selected_nodes.insert(node.id);
                }
            }
            
            if drag_delta != Vec2::ZERO {
                node_move_delta = drag_delta;
                node_to_move = Some(node.id);
                
                if !self.selected_nodes.contains(&node.id) {
                     if !input_modifiers.shift {
                        self.selected_nodes.clear();
                     }
                     self.selected_nodes.insert(node.id);
                }
            }
            if let Some(event) = start_connect {
                connect_event = Some(event);
            }
        }

        // Apply Batch Move
        if node_move_delta != Vec2::ZERO {
            for id in &self.selected_nodes {
                if let Some(node) = graph.nodes.get_mut(id) {
                    node.position.0 += node_move_delta.x;
                    node.position.1 += node_move_delta.y;
                }
            }
        }
        
        // Background Selection Box Logic
        if input_primary_down 
            && !interaction_consumed
            && node_to_move.is_none() 
            && connect_event.is_none()
            && self.connection_start.is_none() 
            && !input_modifiers.alt
        {
             if let Some(pos) = pointer_pos {
                 if let Some(mut rect) = self.selection_box {
                     rect.max = pos;
                     self.selection_box = Some(rect);
                 } else if input_primary_pressed {
                     self.selection_box = Some(Rect::from_min_max(pos, pos));
                     if !input_modifiers.shift {
                         self.selected_nodes.clear();
                     }
                 }
             }
        }
        
        if input_primary_released {
            if let Some(rect) = self.selection_box {
                 let selection_rect = Rect::from_min_max(rect.min.min(rect.max), rect.max.max(rect.min));
                 for node in graph.nodes.values() {
                     let node_pos = Pos2::new(node.position.0, node.position.1);
                     let screen_pos = self.to_screen(node_pos, clip_rect.min);
                     let title = format!("{:?}", node.node_type);
                     let title_galley = ui.painter().layout(title, egui::FontId::proportional(14.0 * self.zoom), Color32::WHITE, f32::INFINITY);
                     let node_width = (150.0 * self.zoom).max(title_galley.rect.width() + 20.0 * self.zoom);
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
        if input_delete {
            for id in &self.selected_nodes {
                graph.nodes.remove(id);
                graph.connections.retain(|c| c.from_node != *id && c.to_node != *id);
            }
            self.selected_nodes.clear();
        }
        
        // Right click wire deletion
        if let Some(pos) = pointer_pos {
             if input_secondary_clicked {
                 graph.connections.retain(|c| {
                      let p1 = self.get_port_screen_pos(ui, &graph.nodes, c.from_node, &c.from_port, false, clip_rect.min, &node_sizes);
                      let p2 = self.get_port_screen_pos(ui, &graph.nodes, c.to_node, &c.to_port, true, clip_rect.min, &node_sizes);
                      let mid = p1 + (p2 - p1) * 0.5;
                      mid.distance(pos) > 20.0
                 });
             }
        }

        let has_connect_event = connect_event.is_some();
        if let Some((id, port, is_input)) = connect_event {
             if let Some((start_id, start_port, start_is_input)) = &self.connection_start {
                 if *start_is_input != is_input && *start_id != id {
                     let (from, from_port, to, to_port) = if *start_is_input {
                         (id, port, *start_id, start_port.clone())
                     } else {
                         (*start_id, start_port.clone(), id, port)
                     };
                     graph.connections.push(crate::graph::Connection { from_node: from, from_port, to_node: to, to_port });
                     self.connection_start = None;
                 } else {
                     self.connection_start = Some((id, port, is_input));
                 }
             } else {
                 self.connection_start = Some((id, port, is_input));
             }
        }
        
        // Connection Cancellation
         if (input_escape || input_secondary_clicked) && self.connection_start.is_some() {
             self.connection_start = None;
         }
         
         // Cancellation on release only if not over a port (handled by connect_event)
         if input_primary_released && self.connection_start.is_some() && !has_connect_event {
              self.connection_start = None;
         }
         
         // Quick Add Menu (Spacebar)
         if input_space && self.node_finder.is_none() {
             if let Some(pos) = pointer_pos {
                 self.node_finder = Some(pos);
                 self.node_finder_query.clear();
             }
         }
         
         if let Some(pos) = self.node_finder {
             let mut open = true;
             let window_response = egui::Window::new("Add Node")
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
                     ];
                     for (label, node_type) in options {
                         if self.node_finder_query.is_empty() || label.to_lowercase().contains(&self.node_finder_query.to_lowercase()) {
                             if ui.button(label).clicked() {
                                 let id = Uuid::new_v4();
                                 let (inputs, outputs) = Self::get_ports_for_type(&node_type);
                                 let graph_pos = self.from_screen(pos, clip_rect.min);
                                 graph.nodes.insert(id, Node { id, node_type, position: (graph_pos.x, graph_pos.y), inputs, outputs });
                                 self.node_finder = None;
                             }
                         }
                     }
                 });
             
             if let Some(inner) = window_response {
                 let clicked_outside = input_primary_pressed && !inner.response.rect.contains(pointer_pos.unwrap_or(Pos2::ZERO));
                 if !open || input_escape || clicked_outside {
                     self.node_finder = None;
                 }
             } else if !open || input_escape {
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
        use crate::graph::{Port, VariableValue};
        use crate::node_types::{NodeType, DataType};
        match node_type {
            NodeType::BlueprintFunction { name } if name == "Event Tick" => (
                vec![], 
                vec![Port { name: "Next".into(), data_type: DataType::ExecutionFlow, default_value: VariableValue::None }]
            ),
            NodeType::BlueprintFunction { name } if name == "Print String" => (
                vec![
                    Port { name: "In".into(), data_type: DataType::ExecutionFlow, default_value: VariableValue::None },
                    Port { name: "String".into(), data_type: DataType::String, default_value: VariableValue::String("Hello".into()) }
                ],
                vec![Port { name: "Next".into(), data_type: DataType::ExecutionFlow, default_value: VariableValue::None }]
            ),
            NodeType::Add => (
                 vec![
                    Port { name: "A".into(), data_type: DataType::Float, default_value: VariableValue::Float(0.0) },
                    Port { name: "B".into(), data_type: DataType::Float, default_value: VariableValue::Float(0.0) }
                ],
                vec![Port { name: "Out".into(), data_type: DataType::Float, default_value: VariableValue::Float(0.0) }]
            ),
            NodeType::Subtract => (
                 vec![
                    Port { name: "A".into(), data_type: DataType::Float, default_value: VariableValue::Float(0.0) },
                    Port { name: "B".into(), data_type: DataType::Float, default_value: VariableValue::Float(0.0) }
                ],
                vec![Port { name: "Out".into(), data_type: DataType::Float, default_value: VariableValue::Float(0.0) }]
            ),
            _ => (vec![], vec![])
        }
    }

    fn draw_node(&mut self, ui: &mut egui::Ui, node: &mut Node, input_primary_pressed: bool, input_primary_released: bool, connections: &[crate::graph::Connection], node_sizes: &std::collections::HashMap<Uuid, Vec2>) -> (Vec2, Option<(Uuid, String, bool)>, bool, bool, bool) {
        let node_pos = Pos2::new(node.position.0, node.position.1);
        let screen_pos = self.to_screen(node_pos, ui.max_rect().min);
        
        let mut drag_delta = Vec2::ZERO;
        let mut connect_event = None;
        let mut clicked_port = false;
        let mut pressed_any_port = false;

        let title = format!("{:?}", node.node_type);
        let title_galley = ui.painter().layout(title, egui::FontId::proportional(14.0 * self.zoom), Color32::WHITE, f32::INFINITY);
        let _min_width = 150.0 * self.zoom;
        
        let size = *node_sizes.get(&node.id).unwrap_or(&self.get_node_size(ui, node, connections));
        let node_rect = Rect::from_min_size(screen_pos, size);
        
        // --- Interaction Logic ---
        // 1. Check Ports FIRST
        let mut y_offset = 30.0 * self.zoom;
        for input in &node.inputs {
            let port_pos = screen_pos + Vec2::new(0.0, y_offset);
            let port_rect = Rect::from_center_size(port_pos, Vec2::splat(16.0 * self.zoom));
            let port_response = ui.interact(port_rect, ui.id().with(node.id).with(&input.name).with("in"), Sense::click_and_drag());
            
            if port_response.drag_started() || port_response.clicked() {
                connect_event = Some((node.id, input.name.clone(), true));
                clicked_port = true;
            }
            if port_response.hovered() && input_primary_released {
                connect_event = Some((node.id, input.name.clone(), true));
                clicked_port = true;
            }
            if port_response.contains_pointer() && input_primary_pressed {
                pressed_any_port = true;
            }
            y_offset += 25.0 * self.zoom;
        }

        let mut y_offset = 30.0 * self.zoom;
        for output in &node.outputs {
            let port_pos = screen_pos + Vec2::new(node_rect.width(), y_offset);
            let port_rect = Rect::from_center_size(port_pos, Vec2::splat(16.0 * self.zoom));
            let port_response = ui.interact(port_rect, ui.id().with(node.id).with(&output.name).with("out"), Sense::click_and_drag());
            
            if port_response.drag_started() || port_response.clicked() {
                connect_event = Some((node.id, output.name.clone(), false));
                clicked_port = true;
            }
             if port_response.hovered() && input_primary_released {
                connect_event = Some((node.id, output.name.clone(), false));
                clicked_port = true;
            }
            if port_response.contains_pointer() && input_primary_pressed {
                pressed_any_port = true;
            }
            y_offset += 25.0 * self.zoom;
        }

        // 2. Allocate Node Background
        let response = ui.allocate_rect(node_rect, Sense::click_and_drag());
        if response.dragged() && !pressed_any_port {
            drag_delta = response.drag_delta() / self.zoom;
        }
        let clicked = response.clicked();
        let pressed_node = response.drag_started() || (response.contains_pointer() && input_primary_pressed);

        // --- Drawing Logic ---
        if self.selected_nodes.contains(&node.id) {
             ui.painter().rect_stroke(node_rect.expand(2.0), 3.0, Stroke::new(2.0, Color32::YELLOW), egui::StrokeKind::Middle);
        }

        ui.painter().rect_filled(node_rect, 5.0, Color32::from_gray(64));
        ui.painter().rect_stroke(node_rect, 5.0, Stroke::new(1.0, Color32::BLACK), egui::StrokeKind::Middle);

        let header_rect = Rect::from_min_max(node_rect.min, Pos2::new(node_rect.max.x, node_rect.min.y + 20.0 * self.zoom));
        ui.painter().rect_filled(header_rect, 5.0, Color32::from_rgb(100, 100, 200));
        ui.painter().galley(header_rect.center() - title_galley.rect.size() * 0.5, title_galley, Color32::WHITE);

        // Draw Ports & Inline Editors
        let mut y_offset = 30.0 * self.zoom;
        for input in &mut node.inputs {
            let port_pos = screen_pos + Vec2::new(0.0, y_offset);
            ui.painter().circle_filled(port_pos, 5.0 * self.zoom, self.get_type_color(&input.data_type));
            
            let name_pos = port_pos + Vec2::new(12.0 * self.zoom, 0.0);
            ui.painter().text(name_pos, egui::Align2::LEFT_CENTER, &input.name, egui::FontId::proportional(12.0 * self.zoom), Color32::WHITE);
            
            // Inline Editor
            let is_connected = connections.iter().any(|c| c.to_node == node.id && c.to_port == input.name);
            if !is_connected && input.data_type != DataType::ExecutionFlow {
                let edit_rect = Rect::from_min_size(name_pos + Vec2::new(60.0 * self.zoom, -10.0 * self.zoom), Vec2::new(80.0 * self.zoom, 20.0 * self.zoom));
                
                ui.allocate_ui_at_rect(edit_rect, |ui| {
                    use crate::graph::VariableValue;
                    match &mut input.default_value {
                        VariableValue::String(s) => {
                             ui.add(egui::TextEdit::singleline(s).desired_width(70.0 * self.zoom));
                        }
                        VariableValue::Float(f) => {
                             ui.add(egui::DragValue::new(f).speed(0.1));
                        }
                        VariableValue::Integer(i) => {
                             ui.add(egui::DragValue::new(i));
                        }
                        VariableValue::Boolean(b) => {
                             ui.checkbox(b, "");
                        }
                        _ => {}
                    }
                });
            }

            y_offset += 25.0 * self.zoom;
        }
        
        let mut y_offset = 30.0 * self.zoom;
        for output in &node.outputs {
            let port_pos = screen_pos + Vec2::new(node_rect.width(), y_offset);
            ui.painter().circle_filled(port_pos, 5.0 * self.zoom, self.get_type_color(&output.data_type));
            ui.painter().text(port_pos - Vec2::new(12.0 * self.zoom, 0.0), egui::Align2::RIGHT_CENTER, &output.name, egui::FontId::proportional(12.0 * self.zoom), Color32::WHITE);
            y_offset += 25.0 * self.zoom;
        }

        (drag_delta, connect_event, clicked, clicked_port, pressed_node || pressed_any_port)
    }
    
    fn draw_connections(&self, ui: &egui::Ui, graph: &BlueprintGraph, node_sizes: &std::collections::HashMap<Uuid, Vec2>) {
        let offset = ui.max_rect().min;
        for conn in &graph.connections {
            let p1 = self.get_port_screen_pos(ui, &graph.nodes, conn.from_node, &conn.from_port, false, offset, node_sizes);
            let p2 = self.get_port_screen_pos(ui, &graph.nodes, conn.to_node, &conn.to_port, true, offset, node_sizes);
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

    fn get_port_screen_pos(&self, _ui: &egui::Ui, nodes: &std::collections::HashMap<Uuid, Node>, node_id: Uuid, port_name: &str, is_input: bool, offset: Pos2, node_sizes: &std::collections::HashMap<Uuid, Vec2>) -> Pos2 {
         if let Some(node) = nodes.get(&node_id) {
             let node_pos = Pos2::new(node.position.0, node.position.1);
             let screen_pos = self.to_screen(node_pos, offset);
             
             let node_size = *node_sizes.get(&node_id).unwrap_or(&Vec2::ZERO);

             let index = if is_input {
                 node.inputs.iter().position(|p| p.name == port_name).unwrap_or(0)
             } else {
                 node.outputs.iter().position(|p| p.name == port_name).unwrap_or(0)
             };
             
             let y = 30.0 + (index as f32 * 25.0);
             let x = if is_input { 0.0 } else { node_size.x };
             
             return screen_pos + Vec2::new(x, y * self.zoom);
         }
         Pos2::ZERO
    }

    fn get_node_size(&self, ui: &egui::Ui, node: &Node, connections: &[crate::graph::Connection]) -> Vec2 {
        let title = format!("{:?}", node.node_type);
        let title_width = ui.painter().layout(title, egui::FontId::proportional(14.0 * self.zoom), Color32::WHITE, f32::INFINITY).rect.width();
        
        // Dynamic height based on number of ports
        let port_count = node.inputs.len().max(node.outputs.len());
        let height = (40.0 + port_count as f32 * 25.0) * self.zoom;
        
        let mut max_inline_width: f32 = 0.0;
        for input in &node.inputs {
            let is_connected = connections.iter().any(|c| c.to_node == node.id && c.to_port == input.name);
            if !is_connected && input.data_type != DataType::ExecutionFlow {
                max_inline_width = max_inline_width.max(100.0 * self.zoom);
            }
        }
        
        let min_width = 150.0 * self.zoom;
        let width = min_width.max(title_width + 40.0 * self.zoom + max_inline_width);
        
        Vec2::new(width, height)
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
