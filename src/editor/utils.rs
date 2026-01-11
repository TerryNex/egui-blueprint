//! # Editor Utility Functions
//!
//! This module contains utility functions for the graph editor,
//! including geometry calculations, color utilities, and rendering helpers.
//!
//! ## Key Functions
//!
//! ### Geometry
//! - [`hit_test_bezier`]: Test if a point is near a bezier curve
//! - [`distance_to_segment`]: Calculate distance from point to line segment
//! - [`draw_dashed_line`]: Draw a dashed line on a painter
//!
//! ### Colors
//! - [`get_type_color`]: Get color for a data type
//!
//! ## Usage
//! These functions are used by [`GraphEditor`] for connection rendering
//! and hit testing.

use crate::node_types::DataType;
use egui::{Color32, Pos2, Stroke};

/// Get the display color for a data type.
///
/// # Colors
/// - ExecutionFlow: White (flow connections)
/// - Boolean: Red
/// - Float: Green
/// - Integer: Light Blue
/// - String: Khaki
/// - Vector3: Yellow
/// - Array: Orange
/// - Custom: Gray
pub fn get_type_color(dt: &DataType) -> Color32 {
    match dt {
        DataType::ExecutionFlow => Color32::WHITE,
        DataType::Boolean => Color32::RED,
        DataType::Float => Color32::GREEN,
        DataType::Integer => Color32::LIGHT_BLUE,
        DataType::String => Color32::KHAKI,
        DataType::Vector3 => Color32::YELLOW,
        DataType::Array => Color32::from_rgb(255, 165, 0), // Orange
        DataType::Custom(_) => Color32::GRAY,
    }
}

/// Test if a point is near a bezier connection curve.
///
/// # Arguments
/// * `pos` - Point to test
/// * `p1` - Start point of bezier
/// * `p2` - End point of bezier
/// * `threshold` - Maximum distance to be considered a hit
/// * `zoom` - Current zoom level (unused but kept for API consistency)
///
/// # Returns
/// `true` if the point is within `threshold` pixels of the curve
pub fn hit_test_bezier(pos: Pos2, p1: Pos2, p2: Pos2, threshold: f32) -> bool {
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

        if distance_to_segment(pos, prev, current) < threshold {
            return true;
        }
        prev = current;
    }
    false
}

/// Calculate the distance from a point to a line segment.
///
/// # Arguments
/// * `p` - The point
/// * `a` - Start of segment
/// * `b` - End of segment
///
/// # Returns
/// The shortest distance from `p` to the segment `a-b`
pub fn distance_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    if ab.length_sq() < 1e-6 {
        return p.distance(a);
    }
    let ap = p - a;
    let t = (ap.dot(ab) / ab.length_sq()).clamp(0.0, 1.0);
    let closest = a + ab * t;
    p.distance(closest)
}

/// Draw a dashed line on a painter.
///
/// # Arguments
/// * `painter` - The egui painter to draw on
/// * `start` - Start point
/// * `end` - End point
/// * `dash_length` - Length of each dash
/// * `gap_length` - Length of gaps between dashes
/// * `stroke` - Stroke style to use
pub fn draw_dashed_line(
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

/// Calculate bezier control points for a connection curve.
///
/// Used for consistent bezier curve generation across drawing and hit testing.
///
/// # Returns
/// `(control1, control2)` - The two control points for the cubic bezier
pub fn bezier_control_points(p1: Pos2, p2: Pos2) -> (Pos2, Pos2) {
    let p1_vec = p1.to_vec2();
    let p2_vec = p2.to_vec2();
    let control_scale = (p2_vec.x - p1_vec.x).abs().max(50.0) * 0.5;
    let c1 = Pos2::new(p1.x + control_scale, p1.y);
    let c2 = Pos2::new(p2.x - control_scale, p2.y);
    (c1, c2)
}

/// Interpolate between two colors.
///
/// # Arguments
/// * `c1` - Start color
/// * `c2` - End color
/// * `t` - Interpolation factor (0.0 = c1, 1.0 = c2)
///
/// # Returns
/// Interpolated color
pub fn lerp_color(c1: Color32, c2: Color32, t: f32) -> Color32 {
    let r = (c1.r() as f32 * (1.0 - t) + c2.r() as f32 * t) as u8;
    let g = (c1.g() as f32 * (1.0 - t) + c2.g() as f32 * t) as u8;
    let b = (c1.b() as f32 * (1.0 - t) + c2.b() as f32 * t) as u8;
    let a = (c1.a() as f32 * (1.0 - t) + c2.a() as f32 * t) as u8;
    Color32::from_rgba_premultiplied(r, g, b, a)
}
