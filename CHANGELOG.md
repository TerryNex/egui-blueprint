# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased] - 2026-01-06

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
