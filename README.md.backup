# egui Blueprint Node Editor

A visual programming system similar to UE5 Blueprints, built with Rust and egui.

## Features
- Node Graph Editor
- Execution Flow
- Variable System
- Save/Load support

## Getting Started
1. `cargo run`

## Project Structure

### Core Modules
| File | Purpose | Lines |
|------|---------|-------|
| `main.rs` | Application entry point, UI layout | ~627 |
| `graph.rs` | Graph data structures (Node, Connection, Variable) | ~89 |
| `node_types.rs` | Node type definitions | ~154 |
| `history.rs` | Undo/redo stack | ~59 |

### Editor Module (`src/editor.rs` or `src/editor/`)
Handles the visual graph editor UI:
- Node rendering and interaction
- Connection drawing (bezier curves)
- Pan/zoom controls
- Selection box
- Node finder (search menu)
- Node groups

### Executor Module (`src/executor.rs` or `src/executor/`)
Handles blueprint execution:
- `mod.rs` - Interpreter entry point
- `context.rs` - Variable storage during execution
- `flow_control.rs` - Control flow (Branch, ForLoop, etc.)
- `node_eval.rs` - Node output evaluation
- `type_conversions.rs` - Type casting helpers
- `automation.rs` - Mouse/keyboard automation
- `image_recognition.rs` - Screen capture, template matching

## Modifying the Code

- **To modify editor behavior**: See `src/editor.rs` or the `src/editor/` module
- **To add new node types**: 
  1. Add to `src/node_types.rs`
  2. Add port definitions in `editor.rs` (`get_ports_for_type`)
  3. Add evaluation logic in `executor.rs` (`evaluate_node` or `execute_flow`)
