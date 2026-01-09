//! Coordinate transformation utilities for the graph editor.
//!
//! Handles conversions between graph coordinates and screen coordinates,
//! accounting for pan, zoom, and virtual offset.

use egui::{Pos2, Vec2};

/// Virtual offset to ensure all coordinates stay positive during rendering.
/// This prevents issues when panning left/up causes negative screen coordinates.
pub const VIRTUAL_OFFSET: Vec2 = Vec2::new(5000.0, 5000.0);

/// Convert graph coordinates to screen coordinates.
///
/// # Arguments
/// * `pos` - Position in graph space
/// * `pan` - Current pan offset
/// * `zoom` - Current zoom level
/// * `canvas_offset` - Top-left corner of the canvas in screen space
pub fn to_screen(pos: Pos2, pan: Vec2, zoom: f32, canvas_offset: Pos2) -> Pos2 {
    let virtual_pos = pos.to_vec2() + VIRTUAL_OFFSET;
    canvas_offset + pan + virtual_pos * zoom
}

/// Convert screen coordinates to graph coordinates.
///
/// # Arguments
/// * `screen_pos` - Position in screen space
/// * `pan` - Current pan offset
/// * `zoom` - Current zoom level
/// * `canvas_offset` - Top-left corner of the canvas in screen space
pub fn from_screen(screen_pos: Pos2, pan: Vec2, zoom: f32, canvas_offset: Pos2) -> Pos2 {
    let relative = screen_pos - canvas_offset - pan;
    (relative / zoom - VIRTUAL_OFFSET).to_pos2()
}
