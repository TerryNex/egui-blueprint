//! Editor styling and constants.
//!
//! Contains EditorStyle, color definitions, and input validation constants.

use egui::Color32;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Valid key names for keyboard automation nodes.
pub const VALID_KEYS: &[&str] = &[
    "return",
    "space",
    "backspace",
    "tab",
    "escape",
    "up",
    "down",
    "left",
    "right",
    "shift",
    "ctrl",
    "alt",
    "command",
    "option",
    "meta",
    "f1",
    "f2",
    "f3",
    "f4",
    "f5",
    "f6",
    "f7",
    "f8",
    "f9",
    "f10",
    "f11",
    "f12",
    "home",
    "end",
    "pageup",
    "pagedown",
    "insert",
    "delete",
    "capslock",
    "numlock",
    "scrolllock",
    "printscreen",
    "pause",
    "a",
    "b",
    "c",
    "d",
    "e",
    "f",
    "g",
    "h",
    "i",
    "j",
    "k",
    "l",
    "m",
    "n",
    "o",
    "p",
    "q",
    "r",
    "s",
    "t",
    "u",
    "v",
    "w",
    "x",
    "y",
    "z",
    "0",
    "1",
    "2",
    "3",
    "4",
    "5",
    "6",
    "7",
    "8",
    "9",
];

/// Valid mouse button names.
pub const VALID_BUTTONS: &[&str] = &["left", "right", "middle"];

/// Valid HTTP methods for HTTPRequest nodes.
pub const HTTP_METHODS: &[&str] = &["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];

/// Clipboard data for copy/paste operations.
#[derive(Clone, Serialize, Deserialize)]
pub struct ClipboardData {
    pub nodes: Vec<crate::graph::Node>,
    pub connections: Vec<crate::graph::Connection>,
}

/// Visual styling configuration for the graph editor.
#[derive(Clone, Serialize, Deserialize)]
pub struct EditorStyle {
    pub header_colors: HashMap<String, Color32>,
    pub use_gradient_connections: bool,
    #[serde(default = "default_font_size")]
    pub font_size: f32,
}

fn default_font_size() -> f32 {
    14.0
}

impl Default for EditorStyle {
    fn default() -> Self {
        let mut map = HashMap::new();
        // Events and Functions
        map.insert("Event".into(), Color32::from_rgb(180, 50, 50));       // Red
        map.insert("Function".into(), Color32::from_rgb(50, 100, 200));   // Blue
        
        // Math and Variables
        map.insert("Math".into(), Color32::from_rgb(50, 150, 100));       // Green
        map.insert("Variable".into(), Color32::from_rgb(150, 100, 50));   // Brown/Orange
        
        // Logic and Comparison
        map.insert("Logic".into(), Color32::from_rgb(100, 50, 200));      // Purple
        map.insert("Comparison".into(), Color32::from_rgb(150, 100, 200)); // Light Purple
        
        // Control Flow
        map.insert("ControlFlow".into(), Color32::from_rgb(200, 150, 50)); // Yellow/Orange
        
        // Input/Output
        map.insert("Input".into(), Color32::from_rgb(200, 150, 50));      // Yellow/Orange
        map.insert("IO".into(), Color32::from_rgb(100, 150, 200));        // Light Blue
        
        // Type Conversion
        map.insert("Conversion".into(), Color32::from_rgb(150, 200, 100)); // Lime
        
        // String Operations
        map.insert("String".into(), Color32::from_rgb(200, 100, 100));    // Coral
        
        // System and Data
        map.insert("System".into(), Color32::from_rgb(100, 50, 200));     // Purple
        map.insert("Data".into(), Color32::from_rgb(50, 150, 150));       // Cyan
        
        // Image/Screenshot
        map.insert("Screenshot".into(), Color32::from_rgb(50, 200, 150)); // Teal
        map.insert("Recognition".into(), Color32::from_rgb(200, 50, 150)); // Magenta
        map.insert("Image".into(), Color32::from_rgb(200, 50, 150));      // Magenta (alias)
        
        // Time
        map.insert("Time".into(), Color32::from_rgb(100, 200, 100));      // Light Green
        
        // Default fallback
        map.insert("Default".into(), Color32::from_rgb(100, 100, 100));   // Gray
        
        Self {
            header_colors: map,
            use_gradient_connections: true,
            font_size: 14.0,
        }
    }
}
