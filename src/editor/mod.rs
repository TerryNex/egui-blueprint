//! # Graph Editor
//!
//! This module provides the visual node-based graph editor.
//!
//! ## Submodules
//! - [`node_ports`]: Port definitions for all node types
//! - [`style`]: Editor styling and clipboard data
//! - [`utils`]: Geometry, color, and rendering utilities
//!
//! ## Main Type
//! [`GraphEditor`] - The main graph editor widget

// Submodules
pub mod node_ports;
pub mod style;
pub mod utils;

// Re-exports for backwards compatibility
pub use style::{ClipboardData, EditorStyle, HTTP_METHODS, VALID_BUTTONS, VALID_KEYS};

use super::graph::{BlueprintGraph, Node};
use super::node_types::DataType;
use eframe::egui;
use egui::{Color32, CornerRadius, Pos2, Rect, Sense, Stroke, Vec2};
use uuid::Uuid;

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
    /// Currently hovered port (node_id, port_name, is_input) for visual feedback
    pub hovered_port: Option<(Uuid, String, bool)>,
    /// Group ID currently being edited for name (double-click on header)
    pub editing_group_name: Option<Uuid>,
    pub recorder: crate::recorder::Recorder,
    /// Map of node ID to the time it was last executed (for fade-out effect)
    pub node_execution_times: std::collections::HashMap<Uuid, std::time::Instant>,
    /// Cache for image template thumbnails (path -> TextureHandle)
    pub image_thumbnail_cache: std::collections::HashMap<String, egui::TextureHandle>,
    /// List of available template images (cached on first access)
    pub available_templates: Option<Vec<String>>,
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
            hovered_port: None,
            editing_group_name: None,
            recorder: crate::recorder::Recorder::new(),
            node_execution_times: std::collections::HashMap::new(),
            image_thumbnail_cache: std::collections::HashMap::new(),
            available_templates: None,
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
                        let old_zoom = self.zoom;
                        let new_zoom = (old_zoom * zoom_delta).clamp(0.1, 1.0);
                        let actual_zoom_ratio = new_zoom / old_zoom;
                        
                        let pointer = i.pointer.hover_pos().unwrap() - ui.max_rect().min.to_vec2();
                        self.pan = pointer - (pointer - self.pan) * actual_zoom_ratio;
                        self.zoom = new_zoom;
                    }

                    // Scroll wheel zoom (only when not scrolling content)
                    let scroll = i.raw_scroll_delta;
                    if scroll.y != 0.0 && !i.modifiers.shift {
                        let old_zoom = self.zoom;
                        let zoom_factor = 1.0 + scroll.y * 0.001;
                        let new_zoom = (old_zoom * zoom_factor).clamp(0.1, 1.0);
                        
                        // Calculate the actual zoom change ratio
                        let actual_zoom_ratio = new_zoom / old_zoom;
                        
                        // Get pointer position relative to canvas
                        let pointer = i.pointer.hover_pos().unwrap() - ui.max_rect().min.to_vec2();
                        
                        // Adjust pan to keep the point under the cursor stationary
                        // Formula: new_pan = pointer - (pointer - old_pan) * (new_zoom / old_zoom)
                        self.pan = pointer - (pointer - self.pan) * actual_zoom_ratio;
                        self.zoom = new_zoom;
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
                    let old_zoom = self.zoom;
                    let new_zoom = (old_zoom * 1.1).clamp(0.1, 1.0);
                    let actual_ratio = new_zoom / old_zoom;
                    self.pan = center - (center - self.pan) * actual_ratio;
                    self.zoom = new_zoom;
                }
                if i.key_pressed(egui::Key::Minus) {
                    let center = if let Some(p) = i.pointer.hover_pos() {
                        p - ui.max_rect().min
                    } else {
                        ui.max_rect().size() / 2.0
                    };
                    let old_zoom = self.zoom;
                    let new_zoom = (old_zoom * 0.9).clamp(0.1, 1.0);
                    let actual_ratio = new_zoom / old_zoom;
                    self.pan = center - (center - self.pan) * actual_ratio;
                    self.zoom = new_zoom;
                }
                if i.key_pressed(egui::Key::Num0) {
                    self.zoom = 1.0;
                    self.pan = Vec2::new(-5000.0, -5000.0); // Reset to initial pan with VIRTUAL_OFFSET
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

        // Context Menu on Background - Create Group
        // Moved to top to avoid conflict with node context menus (nodes are drawn on top)
        if !self.selected_nodes.is_empty() {
            // Check if any selected node is already in a group
            let already_grouped: std::collections::HashSet<Uuid> = graph
                .groups
                .values()
                .flat_map(|g| g.contained_nodes.iter().cloned())
                .collect();

            let selected_in_group: Vec<_> = self
                .selected_nodes
                .iter()
                .filter(|id| already_grouped.contains(id))
                .cloned()
                .collect();

            let can_create_group = selected_in_group.is_empty();

            // Find group name if selected node is in a group
            let group_info: Option<(Uuid, String)> = if !selected_in_group.is_empty() {
                graph
                    .groups
                    .values()
                    .find(|g| {
                        g.contained_nodes
                            .iter()
                            .any(|id| selected_in_group.contains(id))
                    })
                    .map(|g| (g.id, g.name.clone()))
            } else {
                None
            };

            ui.interact(clip_rect, ui.id().with("bg_context"), Sense::click())
                .context_menu(|ui| {
                    if can_create_group {
                        if ui.button("Create Group from Selection").clicked() {
                            // Calculate bounds of selected nodes
                            let mut min_x = f32::INFINITY;
                            let mut min_y = f32::INFINITY;
                            let mut max_x = f32::NEG_INFINITY;
                            let mut max_y = f32::NEG_INFINITY;

                            let nodes: Vec<Uuid> = self.selected_nodes.iter().cloned().collect();

                            for id in &nodes {
                                if let Some(node) = graph.nodes.get(id) {
                                    min_x = min_x.min(node.position.0);
                                    min_y = min_y.min(node.position.1);
                                    let size = node_sizes
                                        .get(id)
                                        .cloned()
                                        .unwrap_or(Vec2::new(150.0, 100.0));
                                    let node_width = size.x / self.zoom;
                                    let node_height = size.y / self.zoom;

                                    max_x = max_x.max(node.position.0 + node_width);
                                    max_y = max_y.max(node.position.1 + node_height);
                                }
                            }

                            if min_x != f32::INFINITY {
                                let padding = 20.0;
                                let group_id = Uuid::new_v4();
                                let group = crate::graph::NodeGroup {
                                    id: group_id,
                                    name: "New Group".into(),
                                    position: (min_x - padding, min_y - padding - 30.0),
                                    size: (
                                        max_x - min_x + padding * 2.0,
                                        max_y - min_y + padding * 2.0 + 30.0,
                                    ),
                                    color: [100, 100, 100, 255],
                                    contained_nodes: nodes,
                                };
                                graph.groups.insert(group_id, group);
                                ui.close();
                            }
                        }
                    } else if let Some((group_id, group_name)) = group_info {
                        // Node is already in a group - show group name and allow navigation
                        let label = format!(
                            "📁 In Group: {}",
                            if group_name.is_empty() {
                                "Unnamed"
                            } else {
                                &group_name
                            }
                        );
                        if ui.button(&label).clicked() {
                            // Pan to group
                            if let Some(group) = graph.groups.get(&group_id) {
                                let center = ui.ctx().available_rect().center().to_vec2();
                                let virtual_offset = Vec2::new(5000.0, 5000.0);
                                let group_pos =
                                    Vec2::new(group.position.0, group.position.1) + virtual_offset;
                                self.pan = center - group_pos * self.zoom;
                            }
                            ui.close();
                        }
                    }
                });
        }

        // Draw groups (behind nodes and connections)
        self.draw_groups(ui, graph, canvas_offset, input_primary_pressed);

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

            // Compute overlapping groups for this node (≥50% overlap)
            let node_pos = Pos2::new(node.position.0, node.position.1);
            let node_screen_pos = self.to_screen(node_pos, canvas_offset);
            let node_size = node_sizes
                .get(&node_id)
                .cloned()
                .unwrap_or(Vec2::new(150.0, 100.0));
            let node_screen_rect = Rect::from_min_size(node_screen_pos, node_size);
            let node_area = node_size.x * node_size.y;

            // Find groups this node is NOT already in and overlaps ≥50%
            let overlapping_groups: Vec<(Uuid, String)> = graph
                .groups
                .values()
                .filter(|g| !g.contained_nodes.contains(&node_id))
                .filter_map(|g| {
                    let group_pos = Pos2::new(g.position.0, g.position.1);
                    let group_screen_pos = self.to_screen(group_pos, canvas_offset);
                    let group_screen_size = Vec2::new(g.size.0, g.size.1) * self.zoom;
                    let group_screen_rect =
                        Rect::from_min_size(group_screen_pos, group_screen_size);

                    let intersection = node_screen_rect.intersect(group_screen_rect);
                    if intersection.is_positive() {
                        let intersection_area = intersection.width() * intersection.height();
                        let overlap_ratio = intersection_area / node_area;
                        if overlap_ratio >= 0.5 {
                            return Some((g.id, g.name.clone()));
                        }
                    }
                    None
                })
                .collect();

            // Use child_ui with clip_rect to prevent layout cursor accumulation between nodes
            // This ensures each node's ui.interact() and ui.scope_builder() don't affect other nodes
            let (
                drag_delta,
                start_connect,
                _clicked,
                right_clicked,
                clicked_port,
                pressed,
                disconnect,
                node_changed,
                delete,
                copy,
                add_to_group,
            ) = {
                let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(clip_rect));

                self.draw_node(
                    &mut child_ui,
                    node,
                    input_primary_pressed,
                    input_primary_released,
                    &graph.connections,
                    &node_sizes,
                    &overlapping_groups,
                )
            };

            // Handle add to group request
            if let Some(group_id) = add_to_group {
                if let Some(group) = graph.groups.get_mut(&group_id) {
                    group.contained_nodes.push(node_id);
                    changed = true;
                }
            }

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

            if pressed || clicked_port || right_clicked {
                interaction_consumed = true;
            }

            // Handle Selection on Press (mouse down) - allows immediate drag
            if pressed && !clicked_port {
                if input_modifiers.shift {
                    // Shift+click toggles selection
                    if self.selected_nodes.contains(&node.id) {
                        self.selected_nodes.remove(&node.id);
                    } else {
                        self.selected_nodes.insert(node.id);
                    }
                } else if !self.selected_nodes.contains(&node.id) {
                    // Not selected - select only this node
                    self.selected_nodes.clear();
                    self.selected_nodes.insert(node.id);
                }
                // Else: already selected, keep selection for multi-node drag

                // Bring pressed node to front
                bring_to_front_id = Some(node.id);
            }

            if drag_delta != Vec2::ZERO {
                // Issue 6: Prevent Middle Mouse from moving nodes
                // drag_delta comes from child_ui which usually respects Sense.
                // However, to be absolutely sure, we ensure primary button is down.
                if input_primary_down {
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
        // Allow starting box selection on background even if a node was previously interacted with
        let on_background = if let Some(pos) = pointer_pos {
            // Check if pointer is over any node
            let over_node = graph.nodes.values().any(|node| {
                let node_pos = Pos2::new(node.position.0, node.position.1);
                let screen_pos = self.to_screen(node_pos, clip_rect.min);
                let estimated_size = egui::vec2(200.0 * self.zoom, 100.0 * self.zoom);
                let node_rect = Rect::from_min_size(screen_pos, estimated_size);
                node_rect.contains(pos)
            });
            !over_node
        } else {
            false
        };

        if input_primary_down
            && on_background
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
                        to_port: to_port.clone(),
                    });
                    self.connection_start = None;
                    changed = true;

                    // Dynamic port expansion for StringJoin nodes
                    // When connecting to a StringJoin input, check if we need to add more ports
                    if let Some(target_node) = graph.nodes.get_mut(&to) {
                        if matches!(
                            target_node.node_type,
                            crate::node_types::NodeType::StringJoin
                        ) {
                            // Check if the last input port is now connected
                            if let Some(last_input) = target_node.inputs.last() {
                                let last_name = last_input.name.clone();
                                let last_is_connected = graph
                                    .connections
                                    .iter()
                                    .any(|c| c.to_node == to && c.to_port == last_name);

                                if last_is_connected {
                                    // Add a new input port
                                    let new_idx = target_node.inputs.len();
                                    target_node.inputs.push(super::graph::Port {
                                        name: format!("Input {}", new_idx),
                                        data_type: super::node_types::DataType::Custom(
                                            "Any".into(),
                                        ),
                                        default_value: super::graph::VariableValue::String(
                                            "".into(),
                                        ),
                                    });
                                }
                            }
                        }
                    }
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
                            if let Some(_pos) = pointer_pos {
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
        if self.node_finder.is_none()
            && !ui.memory(|m| m.focused().is_some())
            && !self.recorder.is_recording()
        {
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

        // Quick Add Menu (Spacebar) - only if not editing text
        let any_text_edit_has_focus = ui.ctx().memory(|m| m.focused().is_some());
        if input_space
            && self.node_finder.is_none()
            && self.editing_node_name.is_none()
            && !any_text_edit_has_focus
        {
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
                .max_height(300.0)
                .open(&mut open)
                .show(ui.ctx(), |ui| {
                    ui.text_edit_singleline(&mut self.node_finder_query)
                        .request_focus();

                    egui::ScrollArea::vertical()
                        .max_height(250.0)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.set_min_width(200.0);

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
                                ("For Loop Async", crate::node_types::NodeType::ForLoopAsync),
                                ("For Each Line", crate::node_types::NodeType::ForEachLine),
                                ("While Loop", crate::node_types::NodeType::WhileLoop),
                                ("Delay", crate::node_types::NodeType::Delay),
                                ("Get Timestamp", crate::node_types::NodeType::GetTimestamp),
                                ("Sequence", crate::node_types::NodeType::Sequence),
                                ("Gate", crate::node_types::NodeType::Gate),
                                ("Wait For Condition", crate::node_types::NodeType::WaitForCondition),
                                // Math
                                ("Add", crate::node_types::NodeType::Add),
                                ("Subtract", crate::node_types::NodeType::Subtract),
                                ("Multiply", crate::node_types::NodeType::Multiply),
                                ("Divide", crate::node_types::NodeType::Divide),
                                ("Modulo (%)", crate::node_types::NodeType::Modulo),
                                ("Power (^)", crate::node_types::NodeType::Power),
                                ("Abs", crate::node_types::NodeType::Abs),
                                ("Min", crate::node_types::NodeType::Min),
                                ("Max", crate::node_types::NodeType::Max),
                                ("Clamp", crate::node_types::NodeType::Clamp),
                                ("Random", crate::node_types::NodeType::Random),
                                ("Constant", crate::node_types::NodeType::Constant),
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
                                ("Xor (^)", crate::node_types::NodeType::Xor),
                                // String Operations
                                ("Concat", crate::node_types::NodeType::Concat),
                                ("Split", crate::node_types::NodeType::Split),
                                ("Length", crate::node_types::NodeType::Length),
                                ("Contains", crate::node_types::NodeType::Contains),
                                ("Replace", crate::node_types::NodeType::Replace),
                                ("Format", crate::node_types::NodeType::Format),
                                ("String Join", crate::node_types::NodeType::StringJoin),
                                ("String Between", crate::node_types::NodeType::StringBetween),
                                ("String Trim", crate::node_types::NodeType::StringTrim),
                                // Conversions
                                ("To Integer", crate::node_types::NodeType::ToInteger),
                                ("To Float", crate::node_types::NodeType::ToFloat),
                                ("To String", crate::node_types::NodeType::ToString),
                                // I/O
                                ("Read Input", crate::node_types::NodeType::ReadInput),
                                ("File Read", crate::node_types::NodeType::FileRead),
                                ("File Write", crate::node_types::NodeType::FileWrite),
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
                                // Utility
                                // ("Notes", crate::node_types::NodeType::Notes), // Removed due to bugs
                                // System Control
                                ("Run Command", crate::node_types::NodeType::RunCommand),
                                ("Launch App", crate::node_types::NodeType::LaunchApp),
                                ("Close App", crate::node_types::NodeType::CloseApp),
                                ("Focus Window", crate::node_types::NodeType::FocusWindow),
                                (
                                    "Get Window Position",
                                    crate::node_types::NodeType::GetWindowPosition,
                                ),
                                (
                                    "Set Window Position",
                                    crate::node_types::NodeType::SetWindowPosition,
                                ),
                                // Desktop Input Automation (Module A)
                                ("Click", crate::node_types::NodeType::Click),
                                ("Double Click", crate::node_types::NodeType::DoubleClick),
                                ("Right Click", crate::node_types::NodeType::RightClick),
                                ("Mouse Move", crate::node_types::NodeType::MouseMove),
                                ("Mouse Down", crate::node_types::NodeType::MouseDown),
                                ("Mouse Up", crate::node_types::NodeType::MouseUp),
                                ("Scroll", crate::node_types::NodeType::Scroll),
                                ("Key Press", crate::node_types::NodeType::KeyPress),
                                ("Key Down", crate::node_types::NodeType::KeyDown),
                                ("Key Up", crate::node_types::NodeType::KeyUp),
                                ("Type Text", crate::node_types::NodeType::TypeText),
                                ("Type String", crate::node_types::NodeType::TypeString),
                                ("Hot Key", crate::node_types::NodeType::HotKey),
                                // Data Operations (Module H)
                                ("Array Create", crate::node_types::NodeType::ArrayCreate),
                                ("Array Push", crate::node_types::NodeType::ArrayPush),
                                ("Array Pop", crate::node_types::NodeType::ArrayPop),
                                ("Array Get", crate::node_types::NodeType::ArrayGet),
                                ("Array Set", crate::node_types::NodeType::ArraySet),
                                ("Array Length", crate::node_types::NodeType::ArrayLength),
                                ("JSON Parse", crate::node_types::NodeType::JSONParse),
                                ("JSON Stringify", crate::node_types::NodeType::JSONStringify),
                                ("HTTP Request", crate::node_types::NodeType::HTTPRequest),
                                // Screenshot & Image Tools (Module C)
                                ("Screen Capture", crate::node_types::NodeType::ScreenCapture),
                                (
                                    "Save Screenshot",
                                    crate::node_types::NodeType::SaveScreenshot,
                                ),
                                (
                                    "Region Capture",
                                    crate::node_types::NodeType::RegionCapture,
                                ),
                                // Image Recognition (Module D)
                                (
                                    "Get Pixel Color",
                                    crate::node_types::NodeType::GetPixelColor,
                                ),
                                ("Find Color", crate::node_types::NodeType::FindColor),
                                ("Wait For Color", crate::node_types::NodeType::WaitForColor),
                                ("Find Image", crate::node_types::NodeType::FindImage),
                                ("Wait For Image", crate::node_types::NodeType::WaitForImage),
                                (
                                    "Image Similarity",
                                    crate::node_types::NodeType::ImageSimilarity,
                                ),
                                // String Extraction
                                ("Extract After", crate::node_types::NodeType::ExtractAfter),
                                ("Extract Until", crate::node_types::NodeType::ExtractUntil),
                            ];

                            // Fuzzy search: remove whitespace and support abbreviation matching
                            let query_normalized: String = self
                                .node_finder_query
                                .to_lowercase()
                                .chars()
                                .filter(|c| !c.is_whitespace())
                                .collect();

                            let filtered_options: Vec<_> = options
                                .into_iter()
                                .filter(|(label, _)| {
                                    if query_normalized.is_empty() {
                                        return true;
                                    }
                                    let label_lower = label.to_lowercase();
                                    let label_no_space: String = label_lower
                                        .chars()
                                        .filter(|c| !c.is_whitespace())
                                        .collect();

                                    // Check if label contains query (ignoring spaces)
                                    if label_no_space.contains(&query_normalized) {
                                        return true;
                                    }

                                    // Check abbreviation match (first letter of each word)
                                    let abbreviation: String = label
                                        .split_whitespace()
                                        .filter_map(|word| word.chars().next())
                                        .map(|c| c.to_lowercase().next().unwrap_or(c))
                                        .collect();
                                    if abbreviation.contains(&query_normalized) {
                                        return true;
                                    }

                                    false
                                })
                                .collect();

                            let activate_first = ui.input(|i| {
                                i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Tab)
                            });

                            for (label, node_type) in filtered_options {
                                if ui.button(label).clicked() || activate_first {
                                    // activate_first = false; // logic flow breaks anyway

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
                                            enabled: true,
                                            group_id: None,
                                            note_text: String::new(),
                                            note_size: (200.0, 100.0),
                                        },
                                    );
                                    self.node_finder = None;
                                    changed = true;
                                    break;
                                }
                            }
                        }); // Close ScrollArea
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
                    ui.horizontal(|ui| {
                        ui.label("Font Size:");
                        ui.add(
                            egui::Slider::new(&mut self.style.font_size, 8.0..=24.0).suffix("px"),
                        );
                    });
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
        // Only trigger if no text widget has focus (to allow copying text from input fields)
        let has_text_focus = ui.ctx().memory(|m| m.focused().is_some()) 
            && ui.ctx().wants_keyboard_input();
        
        if !has_text_focus {
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
        }
        // Paste is handled via events usually

        // --- Grouping Logic (Cmd+G) ---
        let mut create_group_action = false;
        ui.input(|i| {
            if (i.modifiers.command || i.modifiers.ctrl) && i.key_pressed(egui::Key::G) {
                create_group_action = true;
            }
        });

        if create_group_action && !self.selected_nodes.is_empty() {
            // Calculate bounds of selected nodes
            let mut min_x = f32::INFINITY;
            let mut min_y = f32::INFINITY;
            let mut max_x = f32::NEG_INFINITY;
            let mut max_y = f32::NEG_INFINITY;

            let mut nodes: Vec<Uuid> = self.selected_nodes.iter().cloned().collect();

            // Issue 2: Prevent Double Grouping
            // Filter out nodes that are already in a group?
            // Actually, we can check if any of these nodes are in any existing group's contained_nodes list.
            // But checking all groups is O(N*M). Since N and M are small, it's fine.
            let already_grouped: std::collections::HashSet<Uuid> = graph
                .groups
                .values()
                .flat_map(|g| g.contained_nodes.clone())
                .collect();

            nodes.retain(|id| !already_grouped.contains(id));

            if nodes.is_empty() {
                // If all selected were already grouped, maybe we should just return or notify?
                // For now, just don't create an empty group.
            } else {
                for id in &nodes {
                    if let Some(node) = graph.nodes.get(id) {
                        min_x = min_x.min(node.position.0);
                        min_y = min_y.min(node.position.1);
                        // Approximate size since we don't store it globally yet
                        // Issue 4: Fix Bounds (Rightmost node cutoff)
                        // This approximation was 150x100.
                        // We should try to use the actual size from the previous frame's node_sizes if available.
                        // We passed node_sizes to draw_connections, but we don't have it here easily unless we store it in self.
                        // Let's assume a safer default or try to get it.
                        // Actually, we can just use a slightly larger default or update this logic later.
                        let node_width = 200.0; // Increased width safety
                        let node_height = 150.0;

                        max_x = max_x.max(node.position.0 + node_width);
                        max_y = max_y.max(node.position.1 + node_height);
                    }
                }

                if min_x != f32::INFINITY {
                    let padding = 30.0;
                    let group_id = Uuid::new_v4();
                    let group = crate::graph::NodeGroup {
                        id: group_id,
                        name: "New Group".into(),
                        position: (min_x - padding, min_y - padding - 30.0), // Extra top padding for header
                        size: (
                            max_x - min_x + padding * 2.0,
                            max_y - min_y + padding * 2.0 + 30.0,
                        ),
                        color: [100, 100, 100, 255],
                        contained_nodes: nodes,
                    };
                    graph.groups.insert(group_id, group);
                    changed = true;
                }
            }
        }

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


    /// Returns the input and output port definitions for a given node type.
    ///
    /// This function delegates to `node_ports::get_ports_for_type` which contains
    /// the complete port definitions for all node types.
    pub fn get_ports_for_type(
        node_type: &crate::node_types::NodeType,
    ) -> (Vec<super::graph::Port>, Vec<super::graph::Port>) {
        node_ports::get_ports_for_type(node_type)
    }


    fn draw_node(
        &mut self,
        ui: &mut egui::Ui,
        node: &mut Node,
        input_primary_pressed: bool,
        input_primary_released: bool,
        connections: &[crate::graph::Connection],
        node_sizes: &std::collections::HashMap<Uuid, Vec2>,
        overlapping_groups: &[(Uuid, String)], // Groups this node overlaps by >=50%
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
        Option<Uuid>, // add_to_group_id - if user clicked "Add to Group"
    ) {
        let node_pos = Pos2::new(node.position.0, node.position.1);
        let screen_pos = self.to_screen(node_pos, ui.max_rect().min);

        let mut drag_delta = Vec2::ZERO;
        let mut connect_event = None;
        let mut clicked_port = false;
        let mut pressed_any_port = false;
        let mut content_changed = false;

        // ===== SPECIAL HANDLING FOR NOTES NODE =====
        // REMOVED at user request due to stability issues (global selection/drift).
        // if matches!(node.node_type, crate::node_types::NodeType::Notes) { ... }
        // ===========================================

        // ===== END NOTES NODE HANDLING =====

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
            // Enlarged hitbox when dragging a connection for easier targeting
            let hitbox_size = if self.connection_start.is_some() { 36.0 } else { 24.0 };
            let port_rect = Rect::from_center_size(port_pos, Vec2::splat(hitbox_size * self.zoom));
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
            // Store hover state for visual feedback during drawing
            if port_response.hovered() {
                self.hovered_port = Some((node.id, input.name.clone(), true));
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
            // Enlarged hitbox when dragging a connection for easier targeting
            let hitbox_size = if self.connection_start.is_some() { 36.0 } else { 24.0 };
            let port_rect = Rect::from_center_size(port_pos, Vec2::splat(hitbox_size * self.zoom));
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
            // Store hover state for visual feedback during drawing
            if port_response.hovered() {
                self.hovered_port = Some((node.id, output.name.clone(), false));
            }
            y_offset += 25.0 * self.zoom;
        }

        // 2. Interact with Node Background
        // Shrink the interaction rect horizontally to avoid capturing port click areas
        let port_zone = 15.0 * self.zoom;
        let node_interact_rect = Rect::from_min_max(
            node_rect.min + Vec2::new(port_zone, 0.0),
            node_rect.max - Vec2::new(port_zone, 0.0),
        );
        let response = ui.interact(
            node_interact_rect,
            ui.id().with(node.id).with("node_bg"),
            Sense::click_and_drag(),
        );
        if response.dragged() && !pressed_any_port {
            drag_delta = response.drag_delta() / self.zoom;
        }

        // Click detection
        let clicked = response.clicked_by(egui::PointerButton::Primary);
        let right_clicked = response.clicked_by(egui::PointerButton::Secondary);
        let pressed_node = response.drag_started() || response.hovered() && input_primary_pressed;

        if self.selected_nodes.contains(&node.id) {
            ui.painter().rect_stroke(
                node_rect.expand(2.0),
                3.0,
                Stroke::new(2.0, Color32::YELLOW),
                egui::StrokeKind::Middle,
            );
        }

        // Visual Feedback for Execution (Fade-Out)
        let mut active_alpha = 0.0;
        if let Some(time) = self.node_execution_times.get(&node.id) {
             let elapsed = time.elapsed().as_secs_f32();
             let fade_duration = 0.5; // 0.5s tail
             if elapsed < fade_duration {
                 active_alpha = (1.0 - elapsed / fade_duration).powf(2.0); // Simple quadratic fade
             }
        }
        
        // Background
        let bg_color = if active_alpha > 0.0 {
            let base_col = Color32::from_rgb(50, 150, 50); // Brighter green start
            let r = crate::editor::utils::lerp(64.0, base_col.r() as f32, active_alpha) as u8;
            let g = crate::editor::utils::lerp(64.0, base_col.g() as f32, active_alpha) as u8;
            let b = crate::editor::utils::lerp(64.0, base_col.b() as f32, active_alpha) as u8;
            Color32::from_rgb(r, g, b)
        } else {
            Color32::from_gray(64)
        };
        
        ui.painter()
            .rect_filled(node_rect, 5.0, bg_color);

        if active_alpha > 0.0 {
            // Add a glowing border effect
             let alpha_u8 = (255.0 * active_alpha) as u8;
             ui.painter().rect_stroke(
                node_rect.expand(2.0),
                5.0,
                Stroke::new(3.0, Color32::GREEN.gamma_multiply(active_alpha)),
                egui::StrokeKind::Middle,
            );
        }
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
            // Math Operations
            crate::node_types::NodeType::Add
            | crate::node_types::NodeType::Subtract
            | crate::node_types::NodeType::Multiply
            | crate::node_types::NodeType::Divide
            | crate::node_types::NodeType::Modulo
            | crate::node_types::NodeType::Power
            | crate::node_types::NodeType::Abs
            | crate::node_types::NodeType::Min
            | crate::node_types::NodeType::Max
            | crate::node_types::NodeType::Clamp
            | crate::node_types::NodeType::Random
            | crate::node_types::NodeType::Constant => "Math",

            // Variables
            crate::node_types::NodeType::GetVariable { .. }
            | crate::node_types::NodeType::SetVariable { .. } => "Variable",

            // String Operations
            crate::node_types::NodeType::StringJoin
            | crate::node_types::NodeType::StringBetween
            | crate::node_types::NodeType::Concat
            | crate::node_types::NodeType::Split
            | crate::node_types::NodeType::Length
            | crate::node_types::NodeType::Contains
            | crate::node_types::NodeType::Replace
            | crate::node_types::NodeType::Format
            | crate::node_types::NodeType::ExtractAfter
            | crate::node_types::NodeType::ExtractUntil => "String",

            // Type Conversions
            crate::node_types::NodeType::ToString
            | crate::node_types::NodeType::ToInteger
            | crate::node_types::NodeType::ToFloat => "Conversion",

            // Comparison Operators
            crate::node_types::NodeType::Equals
            | crate::node_types::NodeType::NotEquals
            | crate::node_types::NodeType::GreaterThan
            | crate::node_types::NodeType::GreaterThanOrEqual
            | crate::node_types::NodeType::LessThan
            | crate::node_types::NodeType::LessThanOrEqual => "Comparison",

            // Logic Operators
            crate::node_types::NodeType::And
            | crate::node_types::NodeType::Or
            | crate::node_types::NodeType::Not
            | crate::node_types::NodeType::Xor => "Logic",

            // Control Flow
            crate::node_types::NodeType::Branch
            | crate::node_types::NodeType::ForLoop
            | crate::node_types::NodeType::WhileLoop
            | crate::node_types::NodeType::Sequence
            | crate::node_types::NodeType::Gate
            | crate::node_types::NodeType::Entry => "ControlFlow",

            // Timing
            crate::node_types::NodeType::Delay => "Time",

            // Desktop Input Automation
            crate::node_types::NodeType::Click
            | crate::node_types::NodeType::DoubleClick
            | crate::node_types::NodeType::RightClick
            | crate::node_types::NodeType::MouseMove
            | crate::node_types::NodeType::MouseDown
            | crate::node_types::NodeType::MouseUp
            | crate::node_types::NodeType::Scroll
            | crate::node_types::NodeType::KeyPress
            | crate::node_types::NodeType::KeyDown
            | crate::node_types::NodeType::KeyUp
            | crate::node_types::NodeType::TypeText
            | crate::node_types::NodeType::TypeString
            | crate::node_types::NodeType::HotKey => "Input",

            // I/O Operations
            crate::node_types::NodeType::ReadInput
            | crate::node_types::NodeType::FileRead
            | crate::node_types::NodeType::FileWrite => "IO",

            // System Control
            crate::node_types::NodeType::RunCommand
            | crate::node_types::NodeType::LaunchApp
            | crate::node_types::NodeType::CloseApp
            | crate::node_types::NodeType::FocusWindow
            | crate::node_types::NodeType::GetWindowPosition
            | crate::node_types::NodeType::SetWindowPosition => "System",

            // Data Operations
            crate::node_types::NodeType::ArrayCreate
            | crate::node_types::NodeType::ArrayPush
            | crate::node_types::NodeType::ArrayPop
            | crate::node_types::NodeType::ArrayGet
            | crate::node_types::NodeType::ArraySet
            | crate::node_types::NodeType::ArrayLength
            | crate::node_types::NodeType::JSONParse
            | crate::node_types::NodeType::JSONStringify
            | crate::node_types::NodeType::HTTPRequest => "Data",

            // Screenshot & Image Tools
            crate::node_types::NodeType::ScreenCapture
            | crate::node_types::NodeType::SaveScreenshot
            | crate::node_types::NodeType::RegionCapture => "Screenshot",

            // Image Recognition
            crate::node_types::NodeType::GetPixelColor
            | crate::node_types::NodeType::FindColor
            | crate::node_types::NodeType::WaitForColor
            | crate::node_types::NodeType::FindImage
            | crate::node_types::NodeType::WaitForImage
            | crate::node_types::NodeType::ImageSimilarity => "Recognition",

            // Input/Output Parameters
            crate::node_types::NodeType::InputParam
            | crate::node_types::NodeType::OutputParam => "Function",

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
            if node.display_name.is_none() {
                node.display_name = Some(String::new());
            }
        }

        ui.painter().rect_filled(header_rect, 5.0, *header_color);
        
        // Show enable/disable checkbox for Event Tick nodes
        if matches!(&node.node_type, crate::node_types::NodeType::BlueprintFunction { name } if name == "Event Tick") {
            let checkbox_size = 14.0 * self.zoom;
            let checkbox_pos = header_rect.left_top() + Vec2::new(4.0 * self.zoom, (header_rect.height() - checkbox_size) / 2.0);
            let checkbox_rect = Rect::from_min_size(checkbox_pos, Vec2::splat(checkbox_size));
            
            let checkbox_response = ui.allocate_rect(checkbox_rect, Sense::click());
            
            // Draw checkbox background
            let bg_color = if node.enabled { Color32::from_rgb(50, 180, 50) } else { Color32::from_rgb(100, 100, 100) };
            ui.painter().rect_filled(checkbox_rect, 2.0, bg_color);
            
            // Draw checkmark if enabled
            if node.enabled {
                let stroke = Stroke::new(2.0 * self.zoom, Color32::WHITE);
                let check_points = [
                    checkbox_rect.left_top() + Vec2::new(checkbox_size * 0.2, checkbox_size * 0.5),
                    checkbox_rect.left_top() + Vec2::new(checkbox_size * 0.4, checkbox_size * 0.75),
                    checkbox_rect.left_top() + Vec2::new(checkbox_size * 0.85, checkbox_size * 0.2),
                ];
                ui.painter().line_segment([check_points[0], check_points[1]], stroke);
                ui.painter().line_segment([check_points[1], check_points[2]], stroke);
            }
            
            // Handle click to toggle
            if checkbox_response.clicked() {
                node.enabled = !node.enabled;
                content_changed = true;
            }
            
            // Tooltip
            checkbox_response.on_hover_text(if node.enabled { "Enabled - Click to disable" } else { "Disabled - Click to enable" });
        }

        // Show text edit if this node is being edited, otherwise show title
        // Cancel editing if node is no longer selected
        if self.editing_node_name == Some(node.id) && !self.selected_nodes.contains(&node.id) {
            self.editing_node_name = None;
        }
        if self.editing_node_name == Some(node.id) {
            if let Some(ref mut name) = node.display_name {
                let edit_rect = Rect::from_center_size(
                    header_rect.center(),
                    Vec2::new(header_rect.width() - 10.0, 16.0 * self.zoom),
                );
                let response = ui
                    .scope_builder(egui::UiBuilder::new().max_rect(edit_rect), |ui| {
                        ui.style_mut().text_styles.insert(
                            egui::TextStyle::Body,
                            egui::FontId::proportional(14.0 * self.zoom),
                        );
                        let r = ui.add(
                            egui::TextEdit::singleline(name)
                                .hint_text("Enter name...")
                                .desired_width(edit_rect.width()),
                        );
                        r.request_focus();
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
                ui.style_mut().text_styles.insert(
                    egui::TextStyle::Body,
                    egui::FontId::proportional(14.0 * self.zoom),
                );
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

        // Show thumbnail preview for FindImage/WaitForImage nodes
        if matches!(
            node.node_type,
            crate::node_types::NodeType::FindImage | crate::node_types::NodeType::WaitForImage
        ) {
            if let Some(path_input) = node.inputs.iter().find(|p| p.name == "ImagePath") {
                if let crate::graph::VariableValue::String(path) = &path_input.default_value {
                    if !path.is_empty() {
                        let thumb_size = 48.0 * self.zoom;
                        // Fixed scaling: all offsets multiplied by zoom
                        let thumb_pos = screen_pos + Vec2::new(
                            node_rect.width() - thumb_size - 5.0 * self.zoom,
                            y_offset + 100.0 * self.zoom
                        );
                        let thumb_rect = Rect::from_min_size(thumb_pos, Vec2::splat(thumb_size));
                        
                        // Draw thumbnail background
                        ui.painter().rect_filled(thumb_rect, 4.0, Color32::from_gray(40));
                        
                        // Load and display thumbnail
                        if let Some(tex) = self.image_thumbnail_cache.get(path) {
                            ui.painter().image(
                                tex.id(),
                                thumb_rect,
                                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                                Color32::WHITE
                            );
                        } else {
                            // Try to load if not cached
                            if let Ok(img) = image::open(path) {
                                let thumb = img.thumbnail(48, 48);
                                let rgba = thumb.to_rgba8();
                                let size = [rgba.width() as _, rgba.height() as _];
                                let pixels = rgba.into_raw();
                                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                                let handle = ui.ctx().load_texture(
                                    path.clone(),
                                    color_image,
                                    egui::TextureOptions::default()
                                );
                                self.image_thumbnail_cache.insert(path.clone(), handle);
                            } else {
                                // Show placeholder for missing image
                                ui.painter().text(
                                    thumb_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "?",
                                    egui::FontId::proportional(20.0 * self.zoom),
                                    Color32::GRAY,
                                );
                            }
                        }
                        
                        // Draw border
                        ui.painter().rect_stroke(
                            thumb_rect,
                            4.0,
                            Stroke::new(1.0, Color32::DARK_GRAY),
                            egui::StrokeKind::Outside,
                        );
                    }
                }
            }
        }

        for input in &mut node.inputs {
            let port_pos = screen_pos + Vec2::new(0.0, y_offset);
            let port_color = self.get_type_color(&input.data_type);

            // Increased port circle from 5 to 7 for better visibility
            ui.painter()
                .circle_filled(port_pos, 7.0 * self.zoom, port_color);

            // Add hover highlight effect with glowing stroke
            let is_hovered = self
                .hovered_port
                .as_ref()
                .map_or(false, |(id, name, is_input)| {
                    *id == node.id && name == &input.name && *is_input
                });
            if is_hovered {
                ui.painter().circle_stroke(
                    port_pos,
                    10.0 * self.zoom,
                    Stroke::new(2.0 * self.zoom, Color32::WHITE),
                );
            }

            let name_pos = port_pos + Vec2::new(14.0 * self.zoom, 0.0);
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
                        ui.style_mut().text_styles.insert(
                            egui::TextStyle::Body,
                            egui::FontId::proportional(12.0 * self.zoom),
                        );
                        use crate::graph::VariableValue;
                        let mut c = false;
                        match &mut input.default_value {
                            VariableValue::String(s) => {
                                let is_key = input.name == "Key"
                                    && matches!(
                                        node.node_type,
                                        crate::node_types::NodeType::KeyPress
                                            | crate::node_types::NodeType::KeyDown
                                            | crate::node_types::NodeType::KeyUp
                                            | crate::node_types::NodeType::HotKey
                                    );
                                let is_btn = input.name == "Button"
                                    && matches!(
                                        node.node_type,
                                        crate::node_types::NodeType::MouseDown
                                            | crate::node_types::NodeType::MouseUp
                                    );
                                let is_method = input.name == "Method"
                                    && matches!(
                                        node.node_type,
                                        crate::node_types::NodeType::HTTPRequest
                                    );

                                if is_key || is_btn || is_method {
                                    let popup_id = ui.make_persistent_id(format!(
                                        "popup_{}_{}",
                                        node.id, input.name
                                    ));

                                    ui.horizontal(|ui| {
                                        let text_color = if (is_key
                                            && (s.len() == 1
                                                || VALID_KEYS.contains(&s.to_lowercase().as_str())))
                                            || (is_btn
                                                && VALID_BUTTONS
                                                    .contains(&s.to_lowercase().as_str()))
                                            || (is_method
                                                && HTTP_METHODS
                                                    .contains(&s.to_uppercase().as_str()))
                                        {
                                            ui.style().visuals.text_color()
                                        } else {
                                            Color32::RED
                                        };

                                        let response = ui.add(
                                            egui::TextEdit::singleline(s)
                                                .desired_width(80.0 * self.zoom)
                                                .text_color(text_color),
                                        );

                                        if response.lost_focus() {
                                            c = true;
                                        }
                                        if response.changed() {
                                            ui.memory_mut(|m| m.open_popup(popup_id));
                                            c = true;
                                        }

                                        if ui.add(egui::Button::new("▼").small()).clicked() {
                                            ui.memory_mut(|m| m.toggle_popup(popup_id));
                                        }

                                        egui::popup_below_widget(
                                            ui,
                                            popup_id,
                                            &response,
                                            egui::PopupCloseBehavior::CloseOnClickOutside,
                                            |ui: &mut egui::Ui| {
                                                egui::Resize::default()
                                                    .id_salt(popup_id)
                                                    .min_size(Vec2::new(150.0, 100.0))
                                                    .max_size(Vec2::new(
                                                        400.0,
                                                        ui.ctx().viewport_rect().height() - 50.0,
                                                    ))
                                                    .with_stroke(true)
                                                    .show(ui, |ui| {
                                                        egui::ScrollArea::vertical().show(
                                                            ui,
                                                            |ui| {
                                                                let options: &[&str] = if is_key {
                                                                    VALID_KEYS
                                                                } else if is_btn {
                                                                    VALID_BUTTONS
                                                                } else {
                                                                    HTTP_METHODS
                                                                };
                                                                let search = s.to_lowercase();
                                                                for &opt in options {
                                                                    if !search.is_empty()
                                                                        && !opt.contains(&search)
                                                                    {
                                                                        continue;
                                                                    }
                                                                    if ui
                                                                        .add(
                                                                            egui::Button::new(opt)
                                                                                .frame(false),
                                                                        )
                                                                        .clicked()
                                                                    {
                                                                        *s = opt.to_string();
                                                                        c = true;
                                                                        ui.close();
                                                                    }
                                                                }

                                                                if options.iter().all(|o| {
                                                                    !search.is_empty()
                                                                        && !o.contains(&search)
                                                                }) {
                                                                    ui.label("No matches");
                                                                }
                                                            },
                                                        );
                                                    });
                                            },
                                        );
                                    });
                                } else {
                                    // Check for ImagePath input in image recognition nodes
                                    let is_image_path = input.name == "ImagePath"
                                        && matches!(
                                            node.node_type,
                                            crate::node_types::NodeType::FindImage
                                                | crate::node_types::NodeType::WaitForImage
                                        );

                                    if is_image_path {
                                        let popup_id = ui.make_persistent_id(format!(
                                            "img_popup_{}_{}",
                                            node.id, input.name
                                        ));

                                        ui.horizontal(|ui| {
                                            // Text input for manual entry
                                            let response = ui.add(
                                                egui::TextEdit::singleline(s)
                                                    .desired_width(50.0 * self.zoom),
                                            );
                                            if response.lost_focus() {
                                                c = true;
                                            }

                                            // Dropdown button
                                            if ui.add(egui::Button::new("📁").small()).clicked() {
                                                // Refresh template list when opening
                                                self.available_templates = None;
                                                ui.memory_mut(|m| m.toggle_popup(popup_id));
                                            }

                                            egui::popup_below_widget(
                                                ui,
                                                popup_id,
                                                &response,
                                                egui::PopupCloseBehavior::CloseOnClickOutside,
                                                |ui: &mut egui::Ui| {
                                                    // Scan templates directory if not cached
                                                    if self.available_templates.is_none() {
                                                        let mut templates = Vec::new();
                                                        if let Ok(entries) = std::fs::read_dir("scripts/templates") {
                                                            for entry in entries.flatten() {
                                                                let path = entry.path();
                                                                if let Some(ext) = path.extension() {
                                                                    if ext == "png" || ext == "jpg" || ext == "jpeg" {
                                                                        templates.push(
                                                                            path.to_string_lossy().to_string()
                                                                        );
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        templates.sort();
                                                        self.available_templates = Some(templates);
                                                    }

                                                    egui::Resize::default()
                                                        .id_salt(popup_id)
                                                        .min_size(Vec2::new(200.0, 280.0))
                                                        .max_size(Vec2::new(
                                                            350.0,
                                                            ui.ctx().viewport_rect().height() - 100.0,
                                                        ))
                                                        .with_stroke(true)
                                                        .show(ui, |ui| {
                                                            ui.label("Select Image Template:");
                                                            
                                                            // Separate search box instead of filtering by input value
                                                            let search_id = ui.make_persistent_id("template_search");
                                                            let mut search_text = ui.data_mut(|d| {
                                                                d.get_temp::<String>(search_id).unwrap_or_default()
                                                            });
                                                            ui.horizontal(|ui| {
                                                                ui.label("🔍");
                                                                ui.text_edit_singleline(&mut search_text);
                                                            });
                                                            ui.data_mut(|d| d.insert_temp(search_id, search_text.clone()));
                                                            
                                                            ui.separator();

                                                            let search = search_text.to_lowercase();
                                                            let templates = self.available_templates.clone().unwrap_or_default();

                                                            if templates.is_empty() {
                                                                ui.label("No images in scripts/templates/");
                                                            } else {
                                                                egui::ScrollArea::vertical()
                                                                    // .max_height(300.0)
                                                                    .max_height(ui.ctx().viewport_rect().height() - 100.0)
                                                                    .show_rows(ui, 52.0, templates.len(), |ui, row_range| {
                                                                        for idx in row_range {
                                                                            if let Some(path) = templates.get(idx) {
                                                                                let filename = std::path::Path::new(path)
                                                                                    .file_name()
                                                                                    .map(|f| f.to_string_lossy().to_string())
                                                                                    .unwrap_or_else(|| path.clone());

                                                                                // Filter by search
                                                                                if !search.is_empty() && !filename.to_lowercase().contains(&search) {
                                                                                    continue;
                                                                                }
                                                                                // each row use uniqe id
                                                                                ui.push_id(path, |ui| {

                                                                                    // start horizontal layout
                                                                                    let inner_resp = ui.horizontal(|ui| {
                                                                                            ui.set_min_width(ui.available_width());
                                                                                        
                                                                                            // Load thumbnail if not cached
                                                                                            if let Some(tex) = self.image_thumbnail_cache.get(path) {
                                                                                                ui.image(egui::load::SizedTexture::new(tex.id(), egui::vec2(48.0, 48.0)));
                                                                                            } else {
                                                                                                // Try to load image
                                                                                                if let Ok(img) = image::open(path) {
                                                                                                    let thumb = img.thumbnail(48, 48);
                                                                                                    let rgba = thumb.to_rgba8();
                                                                                                    let size = [rgba.width() as _, rgba.height() as _];
                                                                                                    let pixels = rgba.into_raw();
                                                                                                    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                                                                                                    let handle = ui.ctx().load_texture(
                                                                                                        path.clone(),
                                                                                                        color_image,
                                                                                                        egui::TextureOptions::default()
                                                                                                    );
                                                                                                    ui.image(egui::load::SizedTexture::new(handle.id(), egui::vec2(48.0, 48.0)));
                                                                                                    self.image_thumbnail_cache.insert(path.clone(), handle);
                                                                                                } else {
                                                                                                    ui.label("⚠️");
                                                                                                }
                                                                                            }
                                                                                            // if ui.add(egui::Button::new(&filename).frame(false)).clicked() {
                                                                                            //     *s = path.clone();
                                                                                            //     c = true;
                                                                                            //     ui.close();
                                                                                            // }
                                                                                            ui.label(&filename); 
                                                                                    });
                                                                                    // get range
                                                                                    let rect = inner_resp.response.rect;
                                                                                    // set rectangle area response
                                                                                    let response = ui.interact(rect, ui.id(), egui::Sense::click());
                                                                                    // hover effect
                                                                                    // if response.hovered() {
                                                                                        
                                                                                    //     // highlight row
                                                                                    //     ui.ctx().layer_painter(egui::LayerId::background()).rect_filled(
                                                                                    //         rect.expand(2.0),           
                                                                                    //         egui::Rounding::same(4),  
                                                                                    //         ui.visuals().widgets.hovered.bg_fill 
                                                                                    //     );
                                                                                    // }
                                                                                    
                                                                                
                                                                                    // process click event
                                                                                    if response.clicked() {
                                                                                        *s = path.clone();
                                                                                        c = true;
                                                                                        ui.close(); // ui.close() usually use in Window or Menu, depending on context
                                                                                    }
                                                                                });
                                                                            }
                                                                        }
                                                                    });
                                                            }
                                                        });
                                                },
                                            );
                                        });
                                    } else {
                                        // Check for Algorithm input in FindImage
                                        let is_algorithm = input.name == "Algorithm"
                                            && matches!(
                                                node.node_type,
                                                crate::node_types::NodeType::FindImage
                                            );
                                        
                                        if is_algorithm {
                                            const ALGORITHMS: [&str; 3] = ["NCC", "SSD", "SSDNorm"];
                                            let current = s.clone();
                                            egui::ComboBox::from_id_salt(format!("algo_{}", node.id))
                                                .selected_text(&current)
                                                .width(80.0 * self.zoom)
                                                .show_ui(ui, |ui| {
                                                    for algo in ALGORITHMS {
                                                        if ui.selectable_label(current == algo, algo).clicked() {
                                                            *s = algo.to_string();
                                                            c = true;
                                                        }
                                                    }
                                                });
                                        } else {
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
                                    }
                                }
                            }
                            VariableValue::Float(f) => {
                                if ui.add(egui::DragValue::new(f).speed(0.1)).changed() {
                                    c = true;
                                }
                            }
                            VariableValue::Integer(i) => {
                                // Tolerance inputs should be clamped to 0-255 range
                                let is_tolerance = input.name == "Tolerance";
                                let drag = if is_tolerance {
                                    // Tolerance: 1-100 where 100 = exact match, 1 = very loose
                                    egui::DragValue::new(i).range(1..=100)
                                } else {
                                    egui::DragValue::new(i)
                                };
                                if ui.add(drag).changed() {
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

        // StringJoin dynamic port expansion based on text content
        // When the last input has content, add a new empty input
        if matches!(node.node_type, crate::node_types::NodeType::StringJoin) {
            if let Some(last_input) = node.inputs.last() {
                let has_content = match &last_input.default_value {
                    crate::graph::VariableValue::String(s) => !s.is_empty(),
                    _ => false,
                };
                if has_content {
                    let new_idx = node.inputs.len();
                    node.inputs.push(super::graph::Port {
                        name: format!("Input {}", new_idx),
                        data_type: super::node_types::DataType::Custom("Any".into()),
                        default_value: super::graph::VariableValue::String("".into()),
                    });
                    content_changed = true;
                }
            }
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
            let port_color = self.get_type_color(&output.data_type);

            // Increased port circle from 5 to 7 for better visibility
            ui.painter()
                .circle_filled(port_pos, 7.0 * self.zoom, port_color);

            // Add hover highlight effect with glowing stroke
            let is_hovered = self
                .hovered_port
                .as_ref()
                .map_or(false, |(id, name, is_input)| {
                    *id == node.id && name == &output.name && !*is_input
                });
            if is_hovered {
                ui.painter().circle_stroke(
                    port_pos,
                    10.0 * self.zoom,
                    Stroke::new(2.0 * self.zoom, Color32::WHITE),
                );
            }

            ui.painter().text(
                port_pos - Vec2::new(14.0 * self.zoom, 0.0),
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
        let mut add_to_group_id: Option<Uuid> = None;
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

            // Add to Group submenu - only show if there are overlapping groups
            if !overlapping_groups.is_empty() {
                ui.menu_button("Add to Group", |ui| {
                    for (group_id, group_name) in overlapping_groups {
                        let label = if group_name.is_empty() {
                            "Unnamed Group".to_string()
                        } else {
                            group_name.clone()
                        };
                        if ui.button(&label).clicked() {
                            add_to_group_id = Some(*group_id);
                            ui.close();
                        }
                    }
                });
                ui.separator();
            }

            if ui.button("Rename").clicked() {
                self.editing_node_name = Some(node.id);
                if node.display_name.is_none() {
                    node.display_name = Some(String::new());
                }
                ui.close();
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
            add_to_group_id,
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
        utils::get_type_color(dt)
    }

    fn hit_test_bezier(&self, pos: Pos2, p1: Pos2, p2: Pos2, threshold: f32) -> bool {
        utils::hit_test_bezier(pos, p1, p2, threshold)
    }

    fn distance_to_segment(&self, p: Pos2, a: Pos2, b: Pos2) -> f32 {
        utils::distance_to_segment(p, a, b)
    }

    fn draw_dashed_line(
        painter: &egui::Painter,
        start: Pos2,
        end: Pos2,
        dash_length: f32,
        gap_length: f32,
        stroke: Stroke,
    ) {
        utils::draw_dashed_line(painter, start, end, dash_length, gap_length, stroke);
    }

    fn draw_groups(
        &mut self,
        ui: &mut egui::Ui,
        graph: &mut BlueprintGraph,
        canvas_offset: Pos2,
        _input_primary_pressed: bool,
    ) {
        let mut group_move_delta = Vec2::ZERO;
        let mut group_to_move: Option<Uuid> = None;
        let mut delete_group_id: Option<Uuid> = None;

        // Z-order: larger groups drawn first (behind smaller ones)
        let mut sorted_groups: Vec<Uuid> = graph.groups.keys().cloned().collect();
        sorted_groups.sort_by(|a, b| {
            let ga = &graph.groups[a];
            let gb = &graph.groups[b];
            let area_b = gb.size.0 * gb.size.1;
            let area_a = ga.size.0 * ga.size.1;
            area_b
                .partial_cmp(&area_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for group_id in sorted_groups {
            let group = match graph.groups.get_mut(&group_id) {
                Some(g) => g,
                None => continue,
            };

            let pos = Pos2::new(group.position.0, group.position.1);
            let size = Vec2::new(group.size.0, group.size.1);
            let screen_pos = self.to_screen(pos, canvas_offset);

            let rect = Rect::from_min_size(screen_pos, size * self.zoom);
            let header_height = 24.0 * self.zoom;
            let header_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), header_height));

            // Header interaction (drag to move group)
            let header_response = ui.interact(
                header_rect,
                ui.id().with(group.id).with("header"),
                Sense::click_and_drag(),
            );

            // Double-click on header to rename
            if header_response.double_clicked() {
                self.editing_group_name = Some(group.id);
            }

            // Context Menu - Ungroup only
            header_response.context_menu(|ui| {
                if ui.button("Ungroup").clicked() {
                    delete_group_id = Some(group.id);
                    ui.close();
                }
            });

            if header_response.dragged() {
                group_move_delta = header_response.drag_delta() / self.zoom;
                group_to_move = Some(group.id);
            }

            // Resize handle
            let resize_size = Vec2::splat(12.0 * self.zoom);
            let resize_rect = Rect::from_min_size(rect.max - resize_size, resize_size);
            let resize_response = ui.interact(
                resize_rect,
                ui.id().with(group.id).with("resize"),
                Sense::drag(),
            );

            if resize_response.dragged() {
                let delta = resize_response.drag_delta() / self.zoom;
                group.size.0 = (group.size.0 + delta.x).max(100.0);
                group.size.1 = (group.size.1 + delta.y).max(100.0);
            }

            // --- Rendering ---
            // Background
            let bg_color = Color32::from_rgba_unmultiplied(
                group.color[0],
                group.color[1],
                group.color[2],
                (group.color[3] as f32 * 0.25) as u8,
            );
            ui.painter().rect_filled(rect, 8.0, bg_color);

            // Border
            ui.painter().rect_stroke(
                rect,
                8.0,
                Stroke::new(
                    2.0,
                    Color32::from_rgba_unmultiplied(
                        group.color[0],
                        group.color[1],
                        group.color[2],
                        group.color[3],
                    ),
                ),
                egui::StrokeKind::Middle,
            );

            // Header
            ui.painter().rect_filled(
                header_rect,
                CornerRadius {
                    nw: 8,
                    ne: 8,
                    sw: 0,
                    se: 0,
                },
                Color32::from_rgba_unmultiplied(
                    group.color[0],
                    group.color[1],
                    group.color[2],
                    group.color[3],
                ),
            );

            // Title text or TextEdit (if editing)
            if self.editing_group_name == Some(group.id) {
                // Show TextEdit for rename
                let title_rect = Rect::from_min_size(
                    header_rect.min + Vec2::new(8.0 * self.zoom, 2.0 * self.zoom),
                    Vec2::new(
                        header_rect.width() - 16.0 * self.zoom,
                        header_height - 4.0 * self.zoom,
                    ),
                );
                let mut title_ui = ui.new_child(egui::UiBuilder::new().max_rect(title_rect));
                let font_id = egui::FontId::proportional(14.0 * self.zoom);
                title_ui
                    .style_mut()
                    .text_styles
                    .insert(egui::TextStyle::Body, font_id);

                let response = title_ui.add(
                    egui::TextEdit::singleline(&mut group.name)
                        .frame(false)
                        .text_color(Color32::WHITE)
                        .desired_width(title_rect.width()),
                );

                // Stop editing on Enter or lose focus
                if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.editing_group_name = None;
                }

                // Request focus on first frame
                if !response.has_focus() {
                    response.request_focus();
                }
            } else {
                // Static title text
                ui.painter().text(
                    header_rect.left_center() + Vec2::new(10.0 * self.zoom, 0.0),
                    egui::Align2::LEFT_CENTER,
                    &group.name,
                    egui::FontId::proportional(14.0 * self.zoom),
                    Color32::WHITE,
                );
            }

            // Resize indicator
            let resize_color = Color32::from_white_alpha(120);
            ui.painter().line_segment(
                [
                    rect.max - Vec2::new(10.0, 2.0) * self.zoom,
                    rect.max - Vec2::new(2.0, 2.0) * self.zoom,
                ],
                Stroke::new(2.0, resize_color),
            );
            ui.painter().line_segment(
                [
                    rect.max - Vec2::new(2.0, 10.0) * self.zoom,
                    rect.max - Vec2::new(2.0, 2.0) * self.zoom,
                ],
                Stroke::new(2.0, resize_color),
            );
        }

        // Apply group deletion
        if let Some(id) = delete_group_id {
            graph.groups.remove(&id);
        }

        // Apply group movement (both group position AND contained nodes)
        if let Some(group_id) = group_to_move {
            if group_move_delta != Vec2::ZERO {
                // First update the group position
                if let Some(group) = graph.groups.get_mut(&group_id) {
                    group.position.0 += group_move_delta.x;
                    group.position.1 += group_move_delta.y;
                }

                // Then move all contained nodes
                if let Some(group) = graph.groups.get(&group_id) {
                    let node_ids = group.contained_nodes.clone();
                    for node_id in node_ids {
                        if let Some(node) = graph.nodes.get_mut(&node_id) {
                            node.position.0 += group_move_delta.x;
                            node.position.1 += group_move_delta.y;
                        }
                    }
                }
            }
        }
    }
}
