use super::graph::{BlueprintGraph, Node};
use super::node_types::DataType;
use eframe::egui;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct ClipboardData {
    pub nodes: Vec<super::graph::Node>,
    pub connections: Vec<super::graph::Connection>,
}

impl Default for EditorStyle {
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert("Event".into(), Color32::from_rgb(180, 50, 50));
        map.insert("Function".into(), Color32::from_rgb(50, 100, 200));
        map.insert("Math".into(), Color32::from_rgb(50, 150, 100));
        map.insert("Variable".into(), Color32::from_rgb(150, 100, 50));
        map.insert("Default".into(), Color32::from_rgb(100, 100, 100));
        Self {
            header_colors: map,
            use_gradient_connections: true,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EditorStyle {
    pub header_colors: HashMap<String, Color32>,
    pub use_gradient_connections: bool,
}

pub struct GraphEditor {
    pub pan: Vec2,
    pub zoom: f32,
    pub dragging_node: Option<Uuid>,
    pub connection_start: Option<(Uuid, String, bool)>,
    pub node_finder: Option<Pos2>,
    pub node_finder_query: String,
    pub selected_nodes: std::collections::HashSet<Uuid>,
    pub selected_connections: std::collections::HashSet<crate::graph::Connection>,
    pub selection_box: Option<Rect>,
    pub style: EditorStyle,
    pub show_settings: bool,
    /// Counter for z-order assignment (incremented each time a node is clicked)
    pub next_z_order: u64,
    /// Node ID currently being edited for display name (double-click on header)
    pub editing_node_name: Option<Uuid>,
}

impl Default for GraphEditor {
    fn default() -> Self {
        Self {
            // Initial pan compensates for VIRTUAL_OFFSET to center view on graph origin
            pan: Vec2::new(-5000.0, -5000.0),
            zoom: 1.0,
            dragging_node: None,
            connection_start: None,
            node_finder: None,
            node_finder_query: String::new(),
            selected_nodes: std::collections::HashSet::new(),
            selected_connections: std::collections::HashSet::new(),
            selection_box: None,
            style: EditorStyle::default(),
            show_settings: false,
            next_z_order: 1,
            editing_node_name: None,
        }
    }
}

impl GraphEditor {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        graph: &mut BlueprintGraph,
        undo_stack: &mut crate::history::UndoStack,
    ) {
        let mut changed = false;
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
            // Zoom with pinch or scroll wheel
            if let Some(_hovered) = i.pointer.hover_pos() {
                if pointer_in_bounds {
                    // Pinch zoom
                    let zoom_delta = i.zoom_delta();
                    if zoom_delta != 1.0 {
                        let pointer = i.pointer.hover_pos().unwrap() - ui.max_rect().min.to_vec2();
                        self.pan = pointer - (pointer - self.pan) * zoom_delta;
                        self.zoom *= zoom_delta;
                    }

                    // Scroll wheel zoom (only when not scrolling content)
                    let scroll = i.raw_scroll_delta;
                    if scroll.y != 0.0 && !i.modifiers.shift {
                        let zoom_factor = 1.0 + scroll.y * 0.001;
                        let pointer = i.pointer.hover_pos().unwrap() - ui.max_rect().min.to_vec2();
                        self.pan = pointer - (pointer - self.pan) * zoom_factor;
                        self.zoom = (self.zoom * zoom_factor).clamp(0.1, 10.0);
                    }
                }
            }

            input_escape = i.key_pressed(egui::Key::Escape);
            input_delete = i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace);
            input_primary_down = i.pointer.primary_down();
            input_primary_pressed = i.pointer.primary_pressed();
            input_primary_released = i.pointer.primary_released();
            input_secondary_clicked = i.pointer.secondary_clicked();
            input_any_released = i.pointer.any_released();
            input_modifiers = i.modifiers;
            input_space = i.key_pressed(egui::Key::Space);

            // Zoom Shortcuts
            if i.modifiers.command {
                if i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals) {
                    let center = if let Some(p) = i.pointer.hover_pos() {
                        p - ui.max_rect().min
                    } else {
                        ui.max_rect().size() / 2.0
                    };
                    let delta = 1.1;
                    self.pan = center - (center - self.pan) * delta;
                    self.zoom *= delta;
                }
                if i.key_pressed(egui::Key::Minus) {
                    let center = if let Some(p) = i.pointer.hover_pos() {
                        p - ui.max_rect().min
                    } else {
                        ui.max_rect().size() / 2.0
                    };
                    let delta = 0.9;
                    self.pan = center - (center - self.pan) * delta;
                    self.zoom *= delta;
                }
                if i.key_pressed(egui::Key::Num0) {
                    self.zoom = 1.0;
                    self.pan = Vec2::ZERO;
                }
            }
        });

        let canvas_offset = ui.max_rect().min;

        // Pre-calculate node sizes (needed for both drawing and connection lines)
        let mut node_sizes = std::collections::HashMap::new();
        for node in graph.nodes.values() {
            node_sizes.insert(node.id, self.get_node_size(ui, node, &graph.connections));
        }

        let painter = ui.painter();
        painter.rect_filled(clip_rect, 0.0, Color32::from_gray(32)); // Background

        // TODO: Draw groups (behind nodes and connections) - Feature not yet implemented
        // self.draw_groups(ui, graph, canvas_offset, input_primary_pressed);

        // Draw connections
        self.draw_connections(ui, graph, &node_sizes);

        // Draw connection in progress
        if let Some((node_id, port_name, is_input)) = &self.connection_start {
            if let Some(pos) = pointer_pos {
                let start_pos = self.get_port_screen_pos(
                    ui,
                    &graph.nodes,
                    *node_id,
                    port_name,
                    *is_input,
                    canvas_offset,
                    &node_sizes,
                );
                self.draw_bezier(ui, start_pos, pos, Color32::WHITE, Color32::WHITE);
            }
        }

        // Draw nodes - sorted by z_order so higher values are on top
        let mut node_move_delta = Vec2::ZERO;
        let mut node_to_move = None;
        let mut connect_event = None;
        let mut interaction_consumed = false;

        let mut disconnect_node_id = None;
        let mut delete_node_id = None;
        let mut context_copy = false;
        let mut bring_to_front_id: Option<Uuid> = None;

        // Sort node IDs by z_order for rendering order (low to high, so high z_order draws last = on top)
        let mut sorted_node_ids: Vec<Uuid> = graph.nodes.keys().cloned().collect();
        sorted_node_ids.sort_by_key(|id| graph.nodes.get(id).map(|n| n.z_order).unwrap_or(0));

        for node_id in sorted_node_ids {
            let node = match graph.nodes.get_mut(&node_id) {
                Some(n) => n,
                None => continue,
            };
            let (
                drag_delta,
                start_connect,
                clicked,
                right_clicked,
                clicked_port,
                pressed,
                disconnect,
                node_changed,
                delete,
                copy,
            ) = self.draw_node(
                ui,
                node,
                input_primary_pressed,
                input_primary_released,
                &graph.connections,
                &node_sizes,
            );

            if node_changed {
                changed = true;
            }

            if copy {
                // Preserve multi-selection: only add to selection if not already selected
                // Don't clear selection when copying
                if !self.selected_nodes.contains(&node.id) {
                    self.selected_nodes.insert(node.id);
                }
                context_copy = true;
            }

            // Handle right-click: if node is already selected, preserve selection
            // If not selected, select only this node
            if right_clicked {
                if !self.selected_nodes.contains(&node.id) {
                    self.selected_nodes.clear();
                    self.selected_nodes.insert(node.id);
                }
                // Don't change selection if already selected - allows multi-select copy
            }

            if disconnect {
                disconnect_node_id = Some(node.id);
            }

            if delete {
                delete_node_id = Some(node.id);
            }

            if pressed || clicked_port {
                interaction_consumed = true;
            }

            if clicked && !clicked_port {
                // Handle Selection Click (left-click only)
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
                // Bring clicked node to front
                bring_to_front_id = Some(node.id);
            }

            if drag_delta != Vec2::ZERO {
                // Only respond to drag if no other drag is in progress
                if self.dragging_node.is_none() || self.dragging_node == Some(node.id) {
                    // Start drag - move the node immediately even if not selected
                    if self.dragging_node.is_none() {
                        self.dragging_node = Some(node.id);
                    }

                    // Move this node regardless of selection state
                    node_move_delta = drag_delta;
                    node_to_move = Some(node.id);

                    // Also move other selected nodes if this one is selected
                    // (handled later in the move logic)
                }
            }
            if let Some(event) = start_connect {
                connect_event = Some(event);
            }
        }

        // On mouse release, finalize drag: select node and bring to front
        if input_primary_released {
            if let Some(dragged_id) = self.dragging_node {
                // Now select and bring to front
                if !self.selected_nodes.contains(&dragged_id) {
                    if !input_modifiers.shift {
                        self.selected_nodes.clear();
                    }
                    self.selected_nodes.insert(dragged_id);
                }
                bring_to_front_id = Some(dragged_id);
            }
            self.dragging_node = None;
        }

        // Update z_order for clicked/dragged node to bring it to front
        if let Some(id) = bring_to_front_id {
            if let Some(node) = graph.nodes.get_mut(&id) {
                node.z_order = self.next_z_order;
                self.next_z_order += 1;
            }
        }

        // Handle Context Menu Delete
        if let Some(id) = delete_node_id {
            graph.nodes.remove(&id);
            graph
                .connections
                .retain(|c| c.from_node != id && c.to_node != id);
            self.selected_nodes.remove(&id);
            changed = true;
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
                    // Clear selection on background click
                    if !input_modifiers.shift {
                        self.selected_nodes.clear();
                        self.selected_connections.clear();
                    }
                }
            }
        }

        // Draw Selection Box with dashed lines
        if let Some(rect) = self.selection_box {
            let r = Rect::from_min_max(rect.min.min(rect.max), rect.max.max(rect.min));
            // Draw dashed rectangle
            let dash_length = 5.0;
            let gap_length = 3.0;
            let stroke = Stroke::new(1.0, Color32::WHITE);

            // Draw dashed lines for each edge
            Self::draw_dashed_line(
                ui.painter(),
                r.left_top(),
                r.right_top(),
                dash_length,
                gap_length,
                stroke,
            );
            Self::draw_dashed_line(
                ui.painter(),
                r.right_top(),
                r.right_bottom(),
                dash_length,
                gap_length,
                stroke,
            );
            Self::draw_dashed_line(
                ui.painter(),
                r.right_bottom(),
                r.left_bottom(),
                dash_length,
                gap_length,
                stroke,
            );
            Self::draw_dashed_line(
                ui.painter(),
                r.left_bottom(),
                r.left_top(),
                dash_length,
                gap_length,
                stroke,
            );

            // Semi-transparent fill
            ui.painter()
                .rect_filled(r, 0.0, Color32::from_white_alpha(20));
        }

        if input_primary_released {
            if let Some(rect) = self.selection_box {
                let selection_rect =
                    Rect::from_min_max(rect.min.min(rect.max), rect.max.max(rect.min));
                for node in graph.nodes.values() {
                    let node_pos = Pos2::new(node.position.0, node.position.1);
                    let screen_pos = self.to_screen(node_pos, clip_rect.min);
                    let title = format!("{:?}", node.node_type);
                    let title_galley = ui.painter().layout(
                        title,
                        egui::FontId::proportional(14.0 * self.zoom),
                        Color32::WHITE,
                        f32::INFINITY,
                    );
                    let node_width =
                        (150.0 * self.zoom).max(title_galley.rect.width() + 20.0 * self.zoom);
                    let node_screen_rect =
                        Rect::from_min_size(screen_pos, Vec2::new(node_width, 100.0 * self.zoom));
                    if selection_rect.intersects(node_screen_rect) {
                        self.selected_nodes.insert(node.id);
                    }
                }
            }
            self.selection_box = None;
        }

        // Delete Selected - Only if no text input is focused
        let any_text_editing = ui.memory(|m| m.focused().is_some());
        if input_delete && !any_text_editing {
            let nodes_to_delete: Vec<Uuid> = self.selected_nodes.iter().cloned().collect();
            if !nodes_to_delete.is_empty() {
                for id in &nodes_to_delete {
                    graph.nodes.remove(id);
                    graph
                        .connections
                        .retain(|c| c.from_node != *id && c.to_node != *id);
                }
                self.selected_nodes.clear();
                changed = true;
            }

            if !self.selected_connections.is_empty() {
                let initial_len = graph.connections.len();
                // Delete Selected Connections
                for conn in &self.selected_connections {
                    graph.connections.retain(|c| c != conn);
                }
                if graph.connections.len() != initial_len {
                    changed = true;
                }
                self.selected_connections.clear();
            }
        }

        if let Some(id) = disconnect_node_id {
            graph
                .connections
                .retain(|c| c.from_node != id && c.to_node != id);
            changed = true;
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
                    graph.connections.push(crate::graph::Connection {
                        from_node: from,
                        from_port,
                        to_node: to,
                        to_port,
                    });
                    self.connection_start = None;
                    changed = true;
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

        // Type to Search
        if self.node_finder.is_none() && !ui.memory(|m| m.focused().is_some()) {
            ui.input(|i| {
                for event in &i.events {
                    if let egui::Event::Text(text) = event {
                        if !text.trim().is_empty() {
                            if let Some(pos) = pointer_pos {
                                // We need to mutate self, so we can't be in immutable borrow if we want to set node_finder immediately?
                                // Actually this closure borrows i (input), but we need to set self.node_finder.
                                // Rust might complain if we borrow self mutably outside.
                                // Let's capture the text and do it outside.
                            }
                        }
                    }
                }
            });
        }

        // Better approach: collect text first
        let mut typed_search = None;
        if self.node_finder.is_none() && !ui.memory(|m| m.focused().is_some()) {
            ui.input(|i| {
                for event in &i.events {
                    if let egui::Event::Text(text) = event {
                        if !text.trim().is_empty() {
                            typed_search = Some(text.clone());
                        }
                    }
                }
            });
        }

        if let Some(text) = typed_search {
            if let Some(pos) = pointer_pos {
                self.node_finder = Some(pos);
                self.node_finder_query = text;
            }
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
                    ui.text_edit_singleline(&mut self.node_finder_query)
                        .request_focus();
                    let options = vec![
                        // Events
                        (
                            "Event Tick",
                            crate::node_types::NodeType::BlueprintFunction {
                                name: "Event Tick".into(),
                            },
                        ),
                        (
                            "Print String",
                            crate::node_types::NodeType::BlueprintFunction {
                                name: "Print String".into(),
                            },
                        ),
                        // Control Flow
                        ("Branch", crate::node_types::NodeType::Branch),
                        ("For Loop", crate::node_types::NodeType::ForLoop),
                        ("While Loop", crate::node_types::NodeType::WhileLoop),
                        ("Delay", crate::node_types::NodeType::Delay),
                        // Math
                        ("Add", crate::node_types::NodeType::Add),
                        ("Subtract", crate::node_types::NodeType::Subtract),
                        ("Multiply", crate::node_types::NodeType::Multiply),
                        ("Divide", crate::node_types::NodeType::Divide),
                        // Comparison
                        ("Equals (==)", crate::node_types::NodeType::Equals),
                        ("Not Equals (!=)", crate::node_types::NodeType::NotEquals),
                        ("Greater Than (>)", crate::node_types::NodeType::GreaterThan),
                        (
                            "Greater or Equal (>=)",
                            crate::node_types::NodeType::GreaterThanOrEqual,
                        ),
                        ("Less Than (<)", crate::node_types::NodeType::LessThan),
                        (
                            "Less or Equal (<=)",
                            crate::node_types::NodeType::LessThanOrEqual,
                        ),
                        // Logic
                        ("And (&&)", crate::node_types::NodeType::And),
                        ("Or (||)", crate::node_types::NodeType::Or),
                        ("Not (!)", crate::node_types::NodeType::Not),
                        // Conversions
                        ("To Integer", crate::node_types::NodeType::ToInteger),
                        ("To Float", crate::node_types::NodeType::ToFloat),
                        ("To String", crate::node_types::NodeType::ToString),
                        // Variables
                        (
                            "Get Variable",
                            crate::node_types::NodeType::GetVariable {
                                name: "MyVar".into(),
                            },
                        ),
                        (
                            "Set Variable",
                            crate::node_types::NodeType::SetVariable {
                                name: "MyVar".into(),
                            },
                        ),
                    ];

                    let filtered_options: Vec<_> = options
                        .into_iter()
                        .filter(|(label, _)| {
                            self.node_finder_query.is_empty()
                                || label
                                    .to_lowercase()
                                    .contains(&self.node_finder_query.to_lowercase())
                        })
                        .collect();

                    let mut activate_first = ui.input(|i| {
                        i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Tab)
                    });

                    for (label, node_type) in filtered_options {
                        if ui.button(label).clicked() || activate_first {
                            activate_first = false;

                            let id = Uuid::new_v4();
                            let (inputs, outputs) = Self::get_ports_for_type(&node_type);
                            let graph_pos = self.from_screen(pos, clip_rect.min);
                            let z_order = self.next_z_order;
                            self.next_z_order += 1;
                            graph.nodes.insert(
                                id,
                                Node {
                                    id,
                                    node_type,
                                    position: (graph_pos.x, graph_pos.y),
                                    inputs,
                                    outputs,
                                    z_order,
                                    display_name: None,
                                },
                            );
                            self.node_finder = None;
                            changed = true;
                            break;
                        }
                    }
                });

            if let Some(inner) = window_response {
                let clicked_outside = input_primary_pressed
                    && !inner
                        .response
                        .rect
                        .contains(pointer_pos.unwrap_or(Pos2::ZERO));
                if !open || input_escape || clicked_outside {
                    self.node_finder = None;
                }
            } else if !open || input_escape {
                self.node_finder = None;
            }
        }

        if self.show_settings {
            egui::Window::new("Style Settings")
                .open(&mut self.show_settings)
                .show(ui.ctx(), |ui| {
                    ui.checkbox(
                        &mut self.style.use_gradient_connections,
                        "Gradient Connections",
                    );
                    ui.separator();
                    ui.label("Header Colors");
                    for (key, color) in &mut self.style.header_colors {
                        ui.horizontal(|ui| {
                            ui.color_edit_button_srgba(color);
                            ui.label(key.clone());
                        });
                    }
                });
        }

        if input_primary_released && self.dragging_node.is_some() {
            changed = true;
            self.dragging_node = None;
        }

        // --- Clipboard Logic ---
        let mut copy_action = context_copy;
        let _paste_action = false;

        // Check for Command+C / Ctrl+C keyboard shortcut
        ui.input(|i| {
            if (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(egui::Key::C) {
                copy_action = true;
            }
            // Also check for Copy event (more reliable on some platforms)
            for event in &i.events {
                if matches!(event, egui::Event::Copy) {
                    copy_action = true;
                }
            }
        });
        // Paste is handled via events usually

        // Handle Copy
        if copy_action && !self.selected_nodes.is_empty() {
            let nodes: Vec<Node> = graph
                .nodes
                .values()
                .filter(|n| self.selected_nodes.contains(&n.id))
                .cloned()
                .collect();

            // Copy connections where both endpoints are selected
            let connections: Vec<crate::graph::Connection> = graph
                .connections
                .iter()
                .filter(|c| {
                    self.selected_nodes.contains(&c.from_node)
                        && self.selected_nodes.contains(&c.to_node)
                })
                .cloned()
                .collect();

            let clipboard_data = ClipboardData { nodes, connections };
            if let Ok(json) = serde_json::to_string(&clipboard_data) {
                ui.ctx().copy_text(json);
            }
        }

        // Handle Paste
        let mut paste_content = None;
        // Check events for Paste
        ui.input(|i| {
            for event in &i.events {
                if let egui::Event::Paste(text) = event {
                    paste_content = Some(text.clone());
                }
            }
        });
        // Also support Cmd+V manual check if event was missed (e.g. consumed) - but better rely on event.

        if let Some(json) = paste_content {
            if let Ok(data) = serde_json::from_str::<ClipboardData>(&json) {
                if !data.nodes.is_empty() {
                    // Calculate offset
                    // If mouse is in canvas, paste at mouse (centering the group)
                    // Else paste near original with offset

                    // Calculate bounding box of copied nodes
                    let min_x = data
                        .nodes
                        .iter()
                        .map(|n| n.position.0)
                        .fold(f32::INFINITY, f32::min);
                    let min_y = data
                        .nodes
                        .iter()
                        .map(|n| n.position.1)
                        .fold(f32::INFINITY, f32::min);
                    let _max_x = data
                        .nodes
                        .iter()
                        .map(|n| n.position.0)
                        .fold(f32::NEG_INFINITY, f32::max);
                    let _max_y = data
                        .nodes
                        .iter()
                        .map(|n| n.position.1)
                        .fold(f32::NEG_INFINITY, f32::max);

                    let canvas_offset = ui.max_rect().min; // Re-declare canvas_offset for this scope
                    let offset_vec = if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                        // Mouse Screen Pos -> Graph Pos
                        // We want the Top-Left of the group to be at Mouse Pos? Or Center?
                        // Let's align Min-Min to Mouse Pos.
                        let mouse_graph_pos = self.from_screen(mouse_pos, canvas_offset); // using canvas_offset from `show` scope?
                        // I need to ensure `canvas_offset` is available here. It is `ui.max_rect().min`.
                        // Wait, `from_screen` uses self.pan/zoom.

                        // Note: `i` here is `ui.input`. `canvas_offset` was defined earlier in `show`.
                        // I need to recalculate or pass it.
                        // But `show` is one big function. `canvas_offset` is defined at top of `show`.
                        // Yes, `canvas_offset` is available.

                        (mouse_graph_pos.x - min_x, mouse_graph_pos.y - min_y)
                    } else {
                        (20.0, 20.0) // fallback offset
                    };

                    let mut id_map = std::collections::HashMap::new();
                    let mut new_nodes = Vec::new();

                    // 1. Create new nodes
                    for node in &data.nodes {
                        let new_id = Uuid::new_v4();
                        id_map.insert(node.id, new_id);
                        let mut new_node = node.clone();
                        new_node.id = new_id;
                        new_node.position.0 += offset_vec.0;
                        new_node.position.1 += offset_vec.1;
                        // Assign new z_order so pasted nodes appear on top
                        new_node.z_order = self.next_z_order;
                        self.next_z_order += 1;
                        new_nodes.push(new_node);
                    }

                    // 2. Insert nodes
                    for node in new_nodes {
                        let id = node.id;
                        graph.nodes.insert(id, node);
                        self.selected_nodes.insert(id); // Select new nodes
                    }

                    // 3. Create new connections
                    for conn in &data.connections {
                        if let (Some(new_from), Some(new_to)) =
                            (id_map.get(&conn.from_node), id_map.get(&conn.to_node))
                        {
                            let new_conn = crate::graph::Connection {
                                from_node: *new_from,
                                from_port: conn.from_port.clone(),
                                to_node: *new_to,
                                to_port: conn.to_port.clone(),
                            };
                            graph.connections.push(new_conn);
                        }
                    }

                    self.selected_nodes.clear(); // Select ONLY pasted
                    for val in id_map.values() {
                        self.selected_nodes.insert(*val);
                    }
                    changed = true;
                    undo_stack.push(graph);
                }
            }
        }

        if changed {
            undo_stack.push(graph);
        }
    }

    /// Virtual offset to ensure all coordinates stay positive during rendering.
    /// This prevents issues when panning left/up causes negative screen coordinates.
    const VIRTUAL_OFFSET: Vec2 = Vec2::new(5000.0, 5000.0);

    fn to_screen(&self, pos: Pos2, canvas_offset: Pos2) -> Pos2 {
        // Add virtual offset before zoom to keep coordinates positive
        let virtual_pos = pos.to_vec2() + Self::VIRTUAL_OFFSET;
        (virtual_pos * self.zoom + self.pan + canvas_offset.to_vec2()).to_pos2()
    }

    fn from_screen(&self, screen_pos: Pos2, canvas_offset: Pos2) -> Pos2 {
        // Remove virtual offset after reverse transform
        let pos = (screen_pos.to_vec2() - self.pan - canvas_offset.to_vec2()) / self.zoom;
        (pos - Self::VIRTUAL_OFFSET).to_pos2()
    }

    fn get_ports_for_type(
        node_type: &crate::node_types::NodeType,
    ) -> (Vec<super::graph::Port>, Vec<super::graph::Port>) {
        use crate::graph::{Port, VariableValue};
        use crate::node_types::{DataType, NodeType};
        match node_type {
            NodeType::BlueprintFunction { name } if name == "Event Tick" => (
                vec![],
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),
            NodeType::Branch => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Condition".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
                vec![
                    Port {
                        name: "True".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "False".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                ],
            ),
            NodeType::BlueprintFunction { name } if name == "Print String" => (
                vec![
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
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),
            NodeType::Add => (
                vec![
                    Port {
                        name: "A".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                    Port {
                        name: "B".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Float,
                    default_value: VariableValue::Float(0.0),
                }],
            ),
            NodeType::Multiply => (
                vec![
                    Port {
                        name: "A".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                    Port {
                        name: "B".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Float,
                    default_value: VariableValue::Float(0.0),
                }],
            ),
            NodeType::ToInteger => (
                vec![Port {
                    name: "In".into(),
                    data_type: DataType::Custom("Any".into()),
                    default_value: VariableValue::Integer(0),
                }],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Integer,
                    default_value: VariableValue::Integer(0),
                }],
            ),
            NodeType::ToFloat => (
                vec![Port {
                    name: "In".into(),
                    data_type: DataType::Custom("Any".into()),
                    default_value: VariableValue::Float(0.0),
                }],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Float,
                    default_value: VariableValue::Float(0.0),
                }],
            ),
            NodeType::ToString => (
                vec![Port {
                    name: "In".into(),
                    data_type: DataType::Custom("Any".into()),
                    default_value: VariableValue::String("".into()),
                }],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::String,
                    default_value: VariableValue::String("".into()),
                }],
            ),
            NodeType::Divide => (
                vec![
                    Port {
                        name: "A".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                    Port {
                        name: "B".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(1.0),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Float,
                    default_value: VariableValue::Float(0.0),
                }],
            ),
            NodeType::GetVariable { .. } => (
                vec![],
                vec![Port {
                    name: "Value".into(),
                    data_type: DataType::Float,
                    default_value: VariableValue::Float(0.0),
                }],
            ),
            NodeType::SetVariable { .. } => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Value".into(),
                        data_type: DataType::Custom("Any".into()),
                        default_value: VariableValue::Float(0.0),
                    },
                ],
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),
            NodeType::Subtract => (
                vec![
                    Port {
                        name: "A".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                    Port {
                        name: "B".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Float,
                    default_value: VariableValue::Float(0.0),
                }],
            ),
            // Comparison nodes - output Boolean
            NodeType::Equals
            | NodeType::NotEquals
            | NodeType::GreaterThan
            | NodeType::GreaterThanOrEqual
            | NodeType::LessThan
            | NodeType::LessThanOrEqual => (
                vec![
                    Port {
                        name: "A".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                    Port {
                        name: "B".into(),
                        data_type: DataType::Float,
                        default_value: VariableValue::Float(0.0),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Boolean,
                    default_value: VariableValue::Boolean(false),
                }],
            ),
            // Logic nodes
            NodeType::And | NodeType::Or => (
                vec![
                    Port {
                        name: "A".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                    Port {
                        name: "B".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(false),
                    },
                ],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Boolean,
                    default_value: VariableValue::Boolean(false),
                }],
            ),
            NodeType::Not => (
                vec![Port {
                    name: "In".into(),
                    data_type: DataType::Boolean,
                    default_value: VariableValue::Boolean(false),
                }],
                vec![Port {
                    name: "Out".into(),
                    data_type: DataType::Boolean,
                    default_value: VariableValue::Boolean(true),
                }],
            ),
            // For Loop
            NodeType::ForLoop => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Start".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "End".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(10),
                    },
                ],
                vec![
                    Port {
                        name: "Loop".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Index".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(0),
                    },
                    Port {
                        name: "Done".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                ],
            ),
            // While Loop
            NodeType::WhileLoop => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Condition".into(),
                        data_type: DataType::Boolean,
                        default_value: VariableValue::Boolean(true),
                    },
                ],
                vec![
                    Port {
                        name: "Loop".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Done".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                ],
            ),
            // Delay
            NodeType::Delay => (
                vec![
                    Port {
                        name: "In".into(),
                        data_type: DataType::ExecutionFlow,
                        default_value: VariableValue::None,
                    },
                    Port {
                        name: "Duration (ms)".into(),
                        data_type: DataType::Integer,
                        default_value: VariableValue::Integer(1000),
                    },
                ],
                vec![Port {
                    name: "Next".into(),
                    data_type: DataType::ExecutionFlow,
                    default_value: VariableValue::None,
                }],
            ),
            _ => (vec![], vec![]),
        }
    }

    fn draw_node(
        &mut self,
        ui: &mut egui::Ui,
        node: &mut Node,
        input_primary_pressed: bool,
        input_primary_released: bool,
        connections: &[crate::graph::Connection],
        node_sizes: &std::collections::HashMap<Uuid, Vec2>,
    ) -> (
        Vec2,
        Option<(Uuid, String, bool)>,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
        bool,
    ) {
        let node_pos = Pos2::new(node.position.0, node.position.1);
        let screen_pos = self.to_screen(node_pos, ui.max_rect().min);

        let mut drag_delta = Vec2::ZERO;
        let mut connect_event = None;
        let mut clicked_port = false;
        let mut pressed_any_port = false;
        let mut content_changed = false;

        // Format title as "Type: CustomName" if display_name is set, otherwise just "Type"
        let type_name = match &node.node_type {
            crate::node_types::NodeType::BlueprintFunction { name } => name.clone(),
            other => format!("{:?}", other),
        };
        let title = if let Some(ref custom_name) = node.display_name {
            if custom_name.is_empty() {
                type_name
            } else {
                format!("{}: {}", type_name, custom_name)
            }
        } else {
            type_name
        };
        let title_galley = ui.painter().layout(
            title,
            egui::FontId::proportional(14.0 * self.zoom),
            Color32::WHITE,
            f32::INFINITY,
        );
        let _min_width = 150.0 * self.zoom;

        let size = *node_sizes
            .get(&node.id)
            .unwrap_or(&self.get_node_size(ui, node, connections));
        let node_rect = Rect::from_min_size(screen_pos, size);

        // --- Interaction Logic ---
        // 1. Check Ports FIRST
        let mut y_offset = 30.0 * self.zoom;

        // Fix set variable layout overlap
        if matches!(
            node.node_type,
            crate::node_types::NodeType::GetVariable { .. }
                | crate::node_types::NodeType::SetVariable { .. }
        ) {
            y_offset += 20.0 * self.zoom;
        }

        for input in &node.inputs {
            let port_pos = screen_pos + Vec2::new(0.0, y_offset);
            let port_rect = Rect::from_center_size(port_pos, Vec2::splat(16.0 * self.zoom));
            let port_response = ui.interact(
                port_rect,
                ui.id().with(node.id).with(&input.name).with("in"),
                Sense::click_and_drag(),
            );

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

        if matches!(
            node.node_type,
            crate::node_types::NodeType::GetVariable { .. }
                | crate::node_types::NodeType::SetVariable { .. }
        ) {
            y_offset += 20.0 * self.zoom;
        }

        for output in &node.outputs {
            let port_pos = screen_pos + Vec2::new(node_rect.width(), y_offset);
            let port_rect = Rect::from_center_size(port_pos, Vec2::splat(16.0 * self.zoom));
            let port_response = ui.interact(
                port_rect,
                ui.id().with(node.id).with(&output.name).with("out"),
                Sense::click_and_drag(),
            );

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
        // Only left-click (primary) should trigger selection changes
        let clicked = response.clicked_by(egui::PointerButton::Primary);
        // Right-click detection for context menu - should not change selection if already selected
        let right_clicked = response.clicked_by(egui::PointerButton::Secondary);
        let pressed_node =
            response.drag_started() || (response.contains_pointer() && input_primary_pressed);

        // --- Drawing Logic ---
        if self.selected_nodes.contains(&node.id) {
            ui.painter().rect_stroke(
                node_rect.expand(2.0),
                3.0,
                Stroke::new(2.0, Color32::YELLOW),
                egui::StrokeKind::Middle,
            );
        }

        ui.painter()
            .rect_filled(node_rect, 5.0, Color32::from_gray(64));
        ui.painter().rect_stroke(
            node_rect,
            5.0,
            Stroke::new(1.0, Color32::BLACK),
            egui::StrokeKind::Middle,
        );

        let category = match &node.node_type {
            crate::node_types::NodeType::BlueprintFunction { name } => {
                if name.starts_with("Event") {
                    "Event"
                } else {
                    "Function"
                }
            }
            crate::node_types::NodeType::Add
            | crate::node_types::NodeType::Subtract
            | crate::node_types::NodeType::Multiply
            | crate::node_types::NodeType::Divide => "Math",
            crate::node_types::NodeType::GetVariable { .. }
            | crate::node_types::NodeType::SetVariable { .. } => "Variable",
            _ => "Default",
        };

        let header_color = self
            .style
            .header_colors
            .get(category)
            .unwrap_or(&Color32::GRAY);

        let header_rect = Rect::from_min_max(
            node_rect.min,
            Pos2::new(node_rect.max.x, node_rect.min.y + 20.0 * self.zoom),
        );

        // Check for double-click on header to start editing name
        let header_response = ui.allocate_rect(header_rect, Sense::click());
        if header_response.double_clicked() {
            self.editing_node_name = Some(node.id);
            // Initialize display_name if not set
            if node.display_name.is_none() {
                node.display_name = Some(String::new());
            }
        }

        ui.painter().rect_filled(header_rect, 5.0, *header_color);

        // Show text edit if this node is being edited, otherwise show title
        if self.editing_node_name == Some(node.id) {
            if let Some(ref mut name) = node.display_name {
                let edit_rect = Rect::from_center_size(
                    header_rect.center(),
                    Vec2::new(header_rect.width() - 10.0, 16.0 * self.zoom),
                );
                let response = ui
                    .scope_builder(egui::UiBuilder::new().max_rect(edit_rect), |ui| {
                        let r = ui.add(
                            egui::TextEdit::singleline(name)
                                .hint_text("Enter name...")
                                .desired_width(edit_rect.width()),
                        );
                        r
                    })
                    .inner;

                // Stop editing on Enter or lose focus
                if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.editing_node_name = None;
                    // Remove empty names
                    if name.is_empty() {
                        node.display_name = None;
                    }
                    content_changed = true;
                }
            }
        } else {
            ui.painter().galley(
                header_rect.center() - title_galley.rect.size() * 0.5,
                title_galley,
                Color32::WHITE,
            );
        }

        // Custom UI for Variable Node Name
        if let crate::node_types::NodeType::GetVariable { name }
        | crate::node_types::NodeType::SetVariable { name } = &mut node.node_type
        {
            let edit_rect = Rect::from_min_size(
                screen_pos + Vec2::new(10.0 * self.zoom, 25.0 * self.zoom),
                Vec2::new(100.0 * self.zoom, 18.0 * self.zoom),
            );
            ui.scope_builder(egui::UiBuilder::new().max_rect(edit_rect), |ui| {
                ui.text_edit_singleline(name);
            });
        }

        // Draw Ports & Inline Editors
        let mut y_offset = 30.0 * self.zoom;

        if matches!(
            node.node_type,
            crate::node_types::NodeType::GetVariable { .. }
                | crate::node_types::NodeType::SetVariable { .. }
        ) {
            y_offset += 20.0 * self.zoom;
        }

        for input in &mut node.inputs {
            let port_pos = screen_pos + Vec2::new(0.0, y_offset);
            ui.painter().circle_filled(
                port_pos,
                5.0 * self.zoom,
                self.get_type_color(&input.data_type),
            );

            let name_pos = port_pos + Vec2::new(12.0 * self.zoom, 0.0);
            ui.painter().text(
                name_pos,
                egui::Align2::LEFT_CENTER,
                &input.name,
                egui::FontId::proportional(12.0 * self.zoom),
                Color32::WHITE,
            );

            // Inline Editor
            let is_connected = connections
                .iter()
                .any(|c| c.to_node == node.id && c.to_port == input.name);
            if !is_connected && input.data_type != DataType::ExecutionFlow {
                let edit_rect = Rect::from_min_size(
                    name_pos + Vec2::new(60.0 * self.zoom, -10.0 * self.zoom),
                    Vec2::new(80.0 * self.zoom, 20.0 * self.zoom),
                );

                let inner_changed = ui
                    .scope_builder(egui::UiBuilder::new().max_rect(edit_rect), |ui| {
                        use crate::graph::VariableValue;
                        let mut c = false;
                        match &mut input.default_value {
                            VariableValue::String(s) => {
                                if ui
                                    .add(
                                        egui::TextEdit::singleline(s)
                                            .desired_width(70.0 * self.zoom),
                                    )
                                    .lost_focus()
                                {
                                    c = true;
                                }
                            }
                            VariableValue::Float(f) => {
                                if ui.add(egui::DragValue::new(f).speed(0.1)).changed() {
                                    c = true;
                                }
                            }
                            VariableValue::Integer(i) => {
                                if ui.add(egui::DragValue::new(i)).changed() {
                                    c = true;
                                }
                            }
                            VariableValue::Boolean(b) => {
                                if ui.checkbox(b, "").changed() {
                                    c = true;
                                }
                            }
                            _ => {}
                        }
                        c
                    })
                    .inner;

                if inner_changed {
                    content_changed = true;
                }
            }

            y_offset += 25.0 * self.zoom;
        }

        let mut y_offset = 30.0 * self.zoom;
        if matches!(
            node.node_type,
            crate::node_types::NodeType::GetVariable { .. }
                | crate::node_types::NodeType::SetVariable { .. }
        ) {
            y_offset += 20.0 * self.zoom;
        }

        for output in &node.outputs {
            let port_pos = screen_pos + Vec2::new(node_rect.width(), y_offset);
            ui.painter().circle_filled(
                port_pos,
                5.0 * self.zoom,
                self.get_type_color(&output.data_type),
            );
            ui.painter().text(
                port_pos - Vec2::new(12.0 * self.zoom, 0.0),
                egui::Align2::RIGHT_CENTER,
                &output.name,
                egui::FontId::proportional(12.0 * self.zoom),
                Color32::WHITE,
            );
            y_offset += 25.0 * self.zoom;
        }

        let mut disconnect_all = false;
        let mut delete_node = false;
        let mut copy_node = false;
        response.context_menu(|ui| {
            if matches!(
                node.node_type,
                crate::node_types::NodeType::Add
                    | crate::node_types::NodeType::Subtract
                    | crate::node_types::NodeType::Multiply
                    | crate::node_types::NodeType::Divide
            ) {
                ui.menu_button("Data Type", |ui| {
                    if ui.button("Float").clicked() {
                        content_changed = true;
                        for p in &mut node.inputs {
                            p.data_type = DataType::Float;
                            p.default_value = crate::graph::VariableValue::Float(0.0);
                        }
                        for p in &mut node.outputs {
                            p.data_type = DataType::Float;
                            p.default_value = crate::graph::VariableValue::Float(0.0);
                        }
                        ui.close();
                    }
                    if ui.button("Integer").clicked() {
                        content_changed = true;
                        for p in &mut node.inputs {
                            p.data_type = DataType::Integer;
                            p.default_value = crate::graph::VariableValue::Integer(0);
                        }
                        for p in &mut node.outputs {
                            p.data_type = DataType::Integer;
                            p.default_value = crate::graph::VariableValue::Integer(0);
                        }
                        ui.close();
                    }
                });
                ui.separator();
            }

            if matches!(
                node.node_type,
                crate::node_types::NodeType::SetVariable { .. }
            ) {
                ui.menu_button("Variable Type", |ui| {
                    if ui.button("Float").clicked() {
                        content_changed = true;
                        for p in &mut node.inputs {
                            if p.name == "Value" {
                                p.data_type = DataType::Float;
                                p.default_value = crate::graph::VariableValue::Float(0.0);
                            }
                        }
                        ui.close();
                    }
                    if ui.button("Integer").clicked() {
                        content_changed = true;
                        for p in &mut node.inputs {
                            if p.name == "Value" {
                                p.data_type = DataType::Integer;
                                p.default_value = crate::graph::VariableValue::Integer(0);
                            }
                        }
                        ui.close();
                    }
                    if ui.button("String").clicked() {
                        content_changed = true;
                        for p in &mut node.inputs {
                            if p.name == "Value" {
                                p.data_type = DataType::String;
                                p.default_value = crate::graph::VariableValue::String("".into());
                            }
                        }
                        ui.close();
                    }
                    if ui.button("Boolean").clicked() {
                        content_changed = true;
                        for p in &mut node.inputs {
                            if p.name == "Value" {
                                p.data_type = DataType::Boolean;
                                p.default_value = crate::graph::VariableValue::Boolean(false);
                            }
                        }
                        ui.close();
                    }
                });
                ui.separator();
            }

            if ui.button("Copy").clicked() {
                copy_node = true;
                ui.close();
            }
            if ui.button("Disconnect All").clicked() {
                disconnect_all = true;
                ui.close();
            }
            if ui.button("Delete").clicked() {
                delete_node = true;
                ui.close();
            }
        });

        (
            drag_delta,
            connect_event,
            clicked,
            right_clicked,
            clicked_port,
            pressed_node || pressed_any_port,
            disconnect_all,
            content_changed,
            delete_node,
            copy_node,
        )
    }

    fn draw_connections(
        &mut self,
        ui: &egui::Ui,
        graph: &mut BlueprintGraph,
        node_sizes: &std::collections::HashMap<Uuid, Vec2>,
    ) {
        let offset = ui.max_rect().min;
        let pointer_pos = ui.ctx().pointer_latest_pos();
        let mut action = None;

        let primary_clicked = ui.input(|i| i.pointer.primary_clicked());
        let secondary_clicked = ui.input(|i| i.pointer.secondary_clicked());
        let shift_down = ui.input(|i| i.modifiers.shift);

        for conn in &graph.connections {
            let p1 = self.get_port_screen_pos(
                ui,
                &graph.nodes,
                conn.from_node,
                &conn.from_port,
                false,
                offset,
                node_sizes,
            );
            let p2 = self.get_port_screen_pos(
                ui,
                &graph.nodes,
                conn.to_node,
                &conn.to_port,
                true,
                offset,
                node_sizes,
            );

            let color = if self.selected_connections.contains(conn) {
                (Color32::YELLOW, Color32::YELLOW)
            } else {
                if self.style.use_gradient_connections {
                    let c1 = self.get_node_color(graph, conn.from_node);
                    let c2 = self.get_node_color(graph, conn.to_node);
                    (c1, c2)
                } else {
                    (Color32::WHITE, Color32::WHITE)
                }
            };

            self.draw_bezier(ui, p1, p2, color.0, color.1);

            if let Some(pos) = pointer_pos {
                // Hit test
                if self.hit_test_bezier(pos, p1, p2, 10.0) {
                    if primary_clicked {
                        action = Some((conn.clone(), "select"));
                    } else if secondary_clicked {
                        action = Some((conn.clone(), "delete"));
                    }
                }
            }
        }

        if let Some((conn, act)) = action {
            if act == "delete" {
                graph.connections.retain(|c| c != &conn);
                self.selected_connections.remove(&conn);
            } else if act == "select" {
                if shift_down {
                    if self.selected_connections.contains(&conn) {
                        self.selected_connections.remove(&conn);
                    } else {
                        self.selected_connections.insert(conn);
                    }
                } else {
                    self.selected_connections.clear();
                    self.selected_connections.insert(conn);
                }
            }
        }
    }

    fn draw_bezier(&self, ui: &egui::Ui, p1: Pos2, p2: Pos2, c1_color: Color32, c2_color: Color32) {
        let p1_vec = p1.to_vec2();
        let p2_vec = p2.to_vec2();
        let control_scale = (p2_vec.x - p1_vec.x).abs().max(50.0) * 0.5;
        let c1 = Pos2::new(p1.x + control_scale, p1.y);
        let c2 = Pos2::new(p2.x - control_scale, p2.y);

        if c1_color == c2_color {
            let curve = egui::epaint::CubicBezierShape::from_points_stroke(
                [p1, c1, c2, p2],
                false,
                Color32::TRANSPARENT,
                Stroke::new(2.0 * self.zoom, c1_color),
            );
            ui.painter().add(curve);
        } else {
            // Gradient Approximation
            let steps = 40;
            let mut prev_p = p1;

            for i in 1..=steps {
                let t = i as f32 / steps as f32;
                let t_inv = 1.0 - t;
                // Cubic Bezier formula
                let p = (t_inv.powi(3) * p1.to_vec2()
                    + 3.0 * t_inv.powi(2) * t * c1.to_vec2()
                    + 3.0 * t_inv * t.powi(2) * c2.to_vec2()
                    + t.powi(3) * p2.to_vec2())
                .to_pos2();

                // Interpolate color
                // Simple linear interpolation of RGBA
                let r = (c1_color.r() as f32 * (1.0 - t) + c2_color.r() as f32 * t) as u8;
                let g = (c1_color.g() as f32 * (1.0 - t) + c2_color.g() as f32 * t) as u8;
                let b = (c1_color.b() as f32 * (1.0 - t) + c2_color.b() as f32 * t) as u8;
                let a = (c1_color.a() as f32 * (1.0 - t) + c2_color.a() as f32 * t) as u8;
                let color = Color32::from_rgba_premultiplied(r, g, b, a); // or from_rgb if alpha is full

                ui.painter()
                    .line_segment([prev_p, p], Stroke::new(2.0 * self.zoom, color));

                prev_p = p;
            }
        }
    }

    fn get_node_color(&self, graph: &BlueprintGraph, node_id: Uuid) -> Color32 {
        if let Some(node) = graph.nodes.get(&node_id) {
            let category = match &node.node_type {
                crate::node_types::NodeType::BlueprintFunction { name } => {
                    if name.starts_with("Event") {
                        "Event"
                    } else {
                        "Function"
                    }
                }
                crate::node_types::NodeType::Add
                | crate::node_types::NodeType::Subtract
                | crate::node_types::NodeType::Multiply
                | crate::node_types::NodeType::Divide => "Math",
                crate::node_types::NodeType::GetVariable { .. }
                | crate::node_types::NodeType::SetVariable { .. } => "Variable",
                _ => "Default",
            };
            *self
                .style
                .header_colors
                .get(category)
                .unwrap_or(&Color32::WHITE)
        } else {
            Color32::WHITE
        }
    }

    fn get_port_screen_pos(
        &self,
        _ui: &egui::Ui,
        nodes: &std::collections::HashMap<Uuid, Node>,
        node_id: Uuid,
        port_name: &str,
        is_input: bool,
        offset: Pos2,
        node_sizes: &std::collections::HashMap<Uuid, Vec2>,
    ) -> Pos2 {
        if let Some(node) = nodes.get(&node_id) {
            let node_pos = Pos2::new(node.position.0, node.position.1);
            let screen_pos = self.to_screen(node_pos, offset);

            let node_size = *node_sizes.get(&node_id).unwrap_or(&Vec2::ZERO);

            let index = if is_input {
                node.inputs
                    .iter()
                    .position(|p| p.name == port_name)
                    .unwrap_or(0)
            } else {
                node.outputs
                    .iter()
                    .position(|p| p.name == port_name)
                    .unwrap_or(0)
            };

            // Match the draw_node calculation: 30.0 base + 20.0 for GetVariable/SetVariable offset
            let mut base_y = 30.0;
            if matches!(
                node.node_type,
                crate::node_types::NodeType::GetVariable { .. }
                    | crate::node_types::NodeType::SetVariable { .. }
            ) {
                base_y += 20.0;
            }

            let y = (base_y + (index as f32 * 25.0)) * self.zoom;
            let x = if is_input { 0.0 } else { node_size.x };

            return screen_pos + Vec2::new(x, y);
        }
        Pos2::ZERO
    }

    fn get_node_size(
        &self,
        ui: &egui::Ui,
        node: &Node,
        connections: &[crate::graph::Connection],
    ) -> Vec2 {
        let title = format!("{:?}", node.node_type);
        let title_width = ui
            .painter()
            .layout(
                title,
                egui::FontId::proportional(14.0 * self.zoom),
                Color32::WHITE,
                f32::INFINITY,
            )
            .rect
            .width();

        // Dynamic height based on number of ports
        let port_count = node.inputs.len().max(node.outputs.len());
        let height = (40.0 + port_count as f32 * 25.0) * self.zoom;

        let mut max_inline_width: f32 = 0.0;
        for input in &node.inputs {
            let is_connected = connections
                .iter()
                .any(|c| c.to_node == node.id && c.to_port == input.name);
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

    fn hit_test_bezier(&self, pos: Pos2, p1: Pos2, p2: Pos2, threshold: f32) -> bool {
        let p1_vec = p1.to_vec2();
        let p2_vec = p2.to_vec2();
        let control_scale = (p2_vec.x - p1_vec.x).abs().max(50.0) * 0.5;
        let c1 = Pos2::new(p1.x + control_scale, p1.y);
        let c2 = Pos2::new(p2.x - control_scale, p2.y);

        let steps = 20;
        let mut prev = p1;
        for i in 1..=steps {
            let t = i as f32 / steps as f32;
            let t_inv = 1.0 - t;
            let current = (t_inv.powi(3) * p1.to_vec2()
                + 3.0 * t_inv.powi(2) * t * c1.to_vec2()
                + 3.0 * t_inv * t.powi(2) * c2.to_vec2()
                + t.powi(3) * p2.to_vec2())
            .to_pos2();

            if self.distance_to_segment(pos, prev, current) < threshold {
                return true;
            }
            prev = current;
        }
        false
    }

    fn distance_to_segment(&self, p: Pos2, a: Pos2, b: Pos2) -> f32 {
        let ab = b - a;
        if ab.length_sq() < 1e-6 {
            return p.distance(a);
        }
        let ap = p - a;
        let t = (ap.dot(ab) / ab.length_sq()).clamp(0.0, 1.0);
        let closest = a + ab * t;
        p.distance(closest)
    }

    fn draw_dashed_line(
        painter: &egui::Painter,
        start: Pos2,
        end: Pos2,
        dash_length: f32,
        gap_length: f32,
        stroke: Stroke,
    ) {
        let dir = end - start;
        let total_length = dir.length();
        if total_length < 0.001 {
            return;
        }

        let unit = dir / total_length;
        let mut pos = 0.0;
        let mut drawing = true;

        while pos < total_length {
            let segment_length = if drawing { dash_length } else { gap_length };
            let segment_end = (pos + segment_length).min(total_length);

            if drawing {
                let p1 = start + unit * pos;
                let p2 = start + unit * segment_end;
                painter.line_segment([p1, p2], stroke);
            }

            pos = segment_end;
            drawing = !drawing;
        }
    }
}
