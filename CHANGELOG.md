# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased] - 2026-01-07

### Added

- **Font Size Settings** (Feature #3): Added adjustable font size (8-24px) in Style Settings window
- **Math Nodes**: Modulo (%), Power (^), Abs, Min, Max, Clamp, Random
- **Logic Node**: Xor (exclusive or)
- **String Nodes**: Concat, Split, Length, Contains, Replace, Format
- **Control Flow Nodes**: Sequence (executes multiple flows in order), Gate (on/off flow control)
- **I/O Nodes**: ReadInput, FileRead, FileWrite
- **WhileLoop Execution**: Full execution logic with 1000 iteration safety limit
- **Module D: Image Recognition** (Agent: Antigravity-D):
  - **Pixel Nodes**: `GetPixelColor` (RGB at coordinates)
  - **Color Search**: `FindColor` (search region for color), `WaitForColor` (wait with timeout)
  - **Image Matching**: `FindImage` (template matching), `WaitForImage` (wait for image)
  - **Comparison**: `ImageSimilarity` (compare two images with tolerance)
  - Uses `image` crate (v0.25) for pixel operations and `xcap` for screen capture
  - Grid sampling optimization for performance
- **Module H: Data Operations** (by Agent Claude-H):
  - **Array Type**: Added `Array` variant to `DataType` and `VariableValue` enums
  - **Array Nodes**: ArrayCreate, ArrayPush, ArrayPop, ArrayGet, ArraySet, ArrayLength
  - **JSON Nodes**: JSONParse (string to value), JSONStringify (value to string)
  - **HTTP Node**: HTTPRequest (GET/POST using curl)
- **Module A: Desktop Input Automation** (Agent: Antigravity-A):
  - Mouse Nodes: `Click`, `DoubleClick`, `RightClick`, `MouseMove`, `MouseDown`, `MouseUp`, `Scroll`
  - Keyboard Nodes: `KeyPress`, `KeyDown`, `KeyUp`, `TypeText`, `HotKey`
  - Uses `enigo` crate (v0.2) for cross-platform input simulation
  - Supports special keys (F1-F12, Arrow keys, Enter, Escape, Tab, etc.)
  - `HotKey` node supports modifier combinations (Ctrl, Shift, Alt, Meta/Command)
- **Module G: System Control** (Agent: Antigravity-G):
  - **App Management**: `RunCommand` (Shell), `LaunchApp`, `CloseApp`
  - **Window Management**: `FocusWindow` (Stub), `GetWindowPosition` (Stub), `SetWindowPosition` (Stub)
  - Cross-platform support for App Launch/Close (macOS/Windows/Linux)
- **Module C: Screenshot & Image Tools** (Agent: Antigravity-C):
  - **Screen Capture**: `ScreenCapture` node - Capture full screen or specific display
  - **File Save**: `SaveScreenshot` node - Copy captured image to specified path
  - Uses `xcap` crate (v0.8) for cross-platform screen capture
  - Screenshots saved to `scripts/screenshots/` with timestamps
- **Module G: Window APIs Implemented** (Agent: Antigravity):
  - `FocusWindow` - Real implementation using AppleScript on macOS, wmctrl on Linux
  - `GetWindowPosition` - Real implementation using AppleScript on macOS, xdotool on Linux  
  - `SetWindowPosition` - Real implementation using AppleScript on macOS, wmctrl on Linux
  - Added result caching for GetWindowPosition to avoid repeated AppleScript calls
- **String Nodes & UX** (Agent: Antigravity):
  - **Connection UX**: Improved port interaction with larger hitboxes (16px → 24px), larger visuals (5px → 7px), and hover glow effects
  - **StringJoin**: Dynamic concatenation node that auto-expands inputs as you connect them
  - **StringBetween**: Extract text between two delimiter strings (Source, Before, After)

### Changed

- Updated node finder menu with all new node types
- EditorStyle now includes `font_size: f32` field with serde default

## [0.1.1] - 2026-01-06

### Added

- **Node Z-Order** (Feature #6): Nodes now layer by last-click order. Most recently clicked/dragged nodes appear on top
- **Editable Node Titles** (Feature #7): Nodes can have custom display names via `display_name` field
- **Scroll Wheel Zoom** (Feature #11/23): Mouse scroll wheel now supports zoom in/out on canvas
- **Node Highlight in List** (Feature #12): Clicking nodes in the Nodes panel now selects and highlights them
- **Collapsible Nodes Window** (Feature #21): Nodes panel converted to a collapsible egui::Window
- **Comparison Nodes** (Feature #17): Added Equals (==), NotEquals (!=), GreaterThan (>), GreaterThanOrEqual (>=), LessThan (<), LessThanOrEqual (<=)
- **Logic Nodes** (Feature #17): Added And (&&), Or (||), Not (!) nodes
- **Loop Nodes** (Feature #18): Added ForLoop and WhileLoop node types with port definitions
- **Delay Node** (Feature #22): Added Delay node for timed execution (duration in ms)

### Fixed

- **Input Focus Conflict** (Bug #5): Delete/Backspace now correctly checks if text input has focus before deleting nodes
- **Multi-Select Copy** (Bug #10): Right-click copy now preserves multi-selection instead of selecting only the clicked node
- **Right-Click Selection** (Bug #10 part 2): Right-clicking a selected node no longer clears multi-selection; only left-click changes selection
- **Duplicate Divide** (Bug #13): Removed duplicate "Divide" entries from node menu
- **Divide by Zero** (Feature #14): Division by zero now uses 1 as divisor instead of returning None
- **SetVariable Port Positions** (Bug #8): Fixed connection line positions for GetVariable/SetVariable nodes by including the 20px offset in port calculations
- **FindColor Output** (Bug #UserReported): Fixed issue where `FindColor`, `GetPixelColor`, `FindImage` outputs (X, Y, Found) were not accessible to downstream nodes

### Changed

- Node struct now includes `z_order: u64` and `display_name: Option<String>` fields
- GraphEditor now includes `next_z_order: u64` counter for z-order management
- Nodes are rendered in z_order sorted order (lowest to highest)

## [0.1.0] - Initial Development

### Added

- Basic node graph editor with egui
- Node types: BlueprintFunction, Branch, ForLoop, GetVariable, SetVariable, Add, Subtract, Multiply, Divide, Entry
- Execution engine with async support
- Save/Load functionality with JSON serialization
- Undo/Redo history system
- Variable system
- Connection drawing with bezier curves
- Colored log output with variable highlighting
