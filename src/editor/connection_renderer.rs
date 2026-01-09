//! Connection rendering utilities.
//!
//! Handles drawing bezier curves between node ports and
//! hit-testing for connection selection.

use egui::{Color32, Pos2, Stroke, Vec2};

/// Draw a bezier curve between two points with gradient coloring.
pub fn draw_bezier(
    painter: &egui::Painter,
    p1: Pos2,
    p2: Pos2,
    c1_color: Color32,
    c2_color: Color32,
) {
    let dist = (p2.x - p1.x).abs();
    let control_offset = (dist * 0.5).max(50.0);

    let cp1 = Pos2::new(p1.x + control_offset, p1.y);
    let cp2 = Pos2::new(p2.x - control_offset, p2.y);

    let steps = 30;
    let mut points = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let it = 1.0 - t;
        let x = it.powi(3) * p1.x
            + 3.0 * it.powi(2) * t * cp1.x
            + 3.0 * it * t.powi(2) * cp2.x
            + t.powi(3) * p2.x;
        let y = it.powi(3) * p1.y
            + 3.0 * it.powi(2) * t * cp1.y
            + 3.0 * it * t.powi(2) * cp2.y
            + t.powi(3) * p2.y;
        points.push(Pos2::new(x, y));
    }

    // Draw with gradient
    for i in 0..points.len() - 1 {
        let t = i as f32 / (points.len() - 1) as f32;
        let r = (c1_color.r() as f32 * (1.0 - t) + c2_color.r() as f32 * t) as u8;
        let g = (c1_color.g() as f32 * (1.0 - t) + c2_color.g() as f32 * t) as u8;
        let b = (c1_color.b() as f32 * (1.0 - t) + c2_color.b() as f32 * t) as u8;
        let color = Color32::from_rgb(r, g, b);
        painter.line_segment([points[i], points[i + 1]], Stroke::new(2.0, color));
    }
}

/// Test if a point is within a threshold distance of a bezier curve.
pub fn hit_test_bezier(pos: Pos2, p1: Pos2, p2: Pos2, threshold: f32) -> bool {
    let dist = (p2.x - p1.x).abs();
    let control_offset = (dist * 0.5).max(50.0);

    let cp1 = Pos2::new(p1.x + control_offset, p1.y);
    let cp2 = Pos2::new(p2.x - control_offset, p2.y);

    // Sample points along bezier and check distance
    let steps = 20;
    for i in 0..steps {
        let t1 = i as f32 / steps as f32;
        let t2 = (i + 1) as f32 / steps as f32;

        let it1 = 1.0 - t1;
        let x1 = it1.powi(3) * p1.x
            + 3.0 * it1.powi(2) * t1 * cp1.x
            + 3.0 * it1 * t1.powi(2) * cp2.x
            + t1.powi(3) * p2.x;
        let y1 = it1.powi(3) * p1.y
            + 3.0 * it1.powi(2) * t1 * cp1.y
            + 3.0 * it1 * t1.powi(2) * cp2.y
            + t1.powi(3) * p2.y;

        let it2 = 1.0 - t2;
        let x2 = it2.powi(3) * p1.x
            + 3.0 * it2.powi(2) * t2 * cp1.x
            + 3.0 * it2 * t2.powi(2) * cp2.x
            + t2.powi(3) * p2.x;
        let y2 = it2.powi(3) * p1.y
            + 3.0 * it2.powi(2) * t2 * cp1.y
            + 3.0 * it2 * t2.powi(2) * cp2.y
            + t2.powi(3) * p2.y;

        let a = Pos2::new(x1, y1);
        let b = Pos2::new(x2, y2);

        if distance_to_segment(pos, a, b) < threshold {
            return true;
        }
    }
    false
}

/// Calculate distance from a point to a line segment.
pub fn distance_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let ap = p - a;
    let proj = ab.dot(ap) / ab.length_sq();
    let proj_clamped = proj.clamp(0.0, 1.0);
    let closest = a + ab * proj_clamped;
    (p - closest).length()
}

/// Draw a dashed line between two points.
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
    if total_length == 0.0 {
        return;
    }

    let dir_norm = dir / total_length;
    let mut current = 0.0;
    let mut drawing = true;

    while current < total_length {
        let segment_length = if drawing { dash_length } else { gap_length };
        let segment_end = (current + segment_length).min(total_length);

        if drawing {
            let p1 = start + dir_norm * current;
            let p2 = start + dir_norm * segment_end;
            painter.line_segment([p1, p2], stroke);
        }

        current = segment_end;
        drawing = !drawing;
    }
}
