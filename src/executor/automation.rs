//! Desktop input automation helpers.
//!
//! Provides keyboard and mouse automation functionality using the enigo crate.

use enigo::Key;

/// Convert a string key name to an enigo Key variant.
pub fn string_to_key(key_str: &str) -> Option<Key> {
    match key_str.to_lowercase().as_str() {
        // Modifier keys
        "shift" | "lshift" => Some(Key::Shift),
        "control" | "ctrl" | "lcontrol" => Some(Key::Control),
        "alt" | "option" | "lalt" => Some(Key::Alt),
        "meta" | "command" | "cmd" | "win" | "super" => Some(Key::Meta),

        // Function keys
        "f1" => Some(Key::F1),
        "f2" => Some(Key::F2),
        "f3" => Some(Key::F3),
        "f4" => Some(Key::F4),
        "f5" => Some(Key::F5),
        "f6" => Some(Key::F6),
        "f7" => Some(Key::F7),
        "f8" => Some(Key::F8),
        "f9" => Some(Key::F9),
        "f10" => Some(Key::F10),
        "f11" => Some(Key::F11),
        "f12" => Some(Key::F12),

        // Navigation keys
        "up" | "uparrow" => Some(Key::UpArrow),
        "down" | "downarrow" => Some(Key::DownArrow),
        "left" | "leftarrow" => Some(Key::LeftArrow),
        "right" | "rightarrow" => Some(Key::RightArrow),
        "home" => Some(Key::Home),
        "end" => Some(Key::End),
        "pageup" | "pgup" => Some(Key::PageUp),
        "pagedown" | "pgdn" => Some(Key::PageDown),

        // Special keys
        "return" | "enter" => Some(Key::Return),
        "escape" | "esc" => Some(Key::Escape),
        "tab" => Some(Key::Tab),
        "backspace" | "back" => Some(Key::Backspace),
        "delete" | "del" => Some(Key::Delete),
        "space" | " " => Some(Key::Space),
        "capslock" | "caps" => Some(Key::CapsLock),

        // If single character, return as Unicode key
        _ if key_str.len() == 1 => key_str.chars().next().map(Key::Unicode),

        // Unknown key
        _ => None,
    }
}
