# Agent Context

Project: egui Blueprint Node Editor
Goal: Create a visual programming system similar to UE5 Blueprints using Rust and egui.

## Mandatory Workflow Rules

> ⚠️ **IMPORTANT**: These rules MUST be followed for EVERY user request.

### 1. Issue Tracking (TODO.md)

When the user reports bugs or requests features:

1. **Always create/update `TODO.md`** with a comprehensive list of all issues
2. Categorize items by: Bugs, High Priority Features, Medium Priority Features, etc.
3. Include for each item:
   - Clear description of the issue
   - Root cause analysis (if known)
   - Proposed solution/implementation approach
   - Complexity estimate
   - Status marker: `[ ]` Not Started, `[/]` In Progress, `[x]` Completed, `[!]` Blocked
4. **Never lose issue information** - if an item cannot be fixed immediately, it must remain in the TODO list with explanation
5. Update `TODO.md` status after completing each item

### 2. Code Verification

Before final delivery of ANY code changes:

1. **Run `cargo check`** to ensure code compiles
2. Fix any compilation errors before submitting
3. Document any warnings that should be addressed later

### 3. Documentation Updates

- Update `README.md` when adding new features
- Update `CHANGELOG.md` with version, date, and changes
- Keep `agent.md` current with project context

---

## Technical Stack

- Language: Rust
- GUI Framework: egui
- Node Graph Library: egui_node_graph2 (recommended) or tinted_egui_nodes

## Planned Features

- Node Type System (Function, Branch, Loop, Var, Math, etc.)
- Data Type System (Int, Float, String, Bool, Flow, etc.)
- Graph Data Model
- Execution Engine (Interpreter)
- Variable System (Local, Input, Output)
- Serialization (serde)
- Hot-reloading
- Debugging tools

## References

- [egui Goals](https://github.com/emilk/egui?tab=readme-ov-file#goals)
- [egui Documentation (0.33.3)](https://docs.rs/egui/0.33.3/egui/)

---

## Module Location Guide

> [!IMPORTANT]
> **File Size Guideline**: Each code file should remain under 300 lines when possible to improve readability and reduce cognitive load when making changes.

### Editor Code (`src/editor.rs` or `src/editor/`)

| Module | Purpose |
|--------|---------|
| `editor.rs` | Main GraphEditor struct, show() function, all UI logic |
| Future: `style.rs` | EditorStyle, color definitions |
| Future: `node_renderer.rs` | Node drawing, size calculation |
| Future: `connection_renderer.rs` | Bezier curves, port positions |
| Future: `node_ports.rs` | Port definitions per node type |
| Future: `groups.rs` | Node group functionality |
| Future: `node_finder.rs` | Add node search menu |

### Executor Code (`src/executor.rs` or `src/executor/`)

| Module | Purpose |
|--------|---------|
| `executor.rs` | Legacy single-file version (functional) |
| `executor/mod.rs` | Interpreter entry point, run_async |
| `executor/context.rs` | ExecutionContext, variable storage |
| `executor/flow_control.rs` | execute_flow, control flow nodes |
| `executor/node_eval.rs` | evaluate_node, evaluate_input |
| `executor/type_conversions.rs` | to_bool, to_float, to_string, compute_math |
| `executor/automation.rs` | string_to_key, keyboard helpers |
| `executor/image_recognition.rs` | find_template_in_image, compare_images |

### Data Structures (`src/`)

| File | Purpose |
|------|---------|
| `graph.rs` | BlueprintGraph, Node, Connection, VariableValue |
| `node_types.rs` | NodeType enum, DataType enum |
| `history.rs` | UndoStack for undo/redo |

---

## Critical Knowledge: DPI Scaling (Retina Displays)

> [!CAUTION]
> **macOS Retina displays use different coordinate systems!**

### The Problem

- **`enigo.location()`** returns **LOGICAL** coordinates (what user sees)
- **`xcap.capture_image()`** captures in **PHYSICAL** pixels (2x on Retina)
- **`monitor.width()`** returns **LOGICAL** width

### The Solution

Always calculate `scale_factor` when working with screen capture:

```rust
// Calculate DPI scale factor
let logical_width = monitor.width().ok().unwrap_or(img.width()) as f32;
let physical_width = img.width() as f32;
let scale_factor = physical_width / logical_width;

// Convert LOGICAL to PHYSICAL coords (for image access)
let physical_x = (logical_x as f32 * scale_factor) as u32;
let physical_y = (logical_y as f32 * scale_factor) as u32;

// Convert PHYSICAL to LOGICAL coords (for output)
let logical_x = (physical_x as f32 / scale_factor) as i64;
let logical_y = (physical_y as f32 / scale_factor) as i64;
```

### Affected Nodes

- `GetPixelColor` - Input: logical, Access: physical
- `FindColor` - Input region: logical, Search: physical, Output: logical
- `WaitForColor` - Input coords: logical, Access: physical
- `FindImage` - Already handled in `image_matching.rs`
- Cursor Info Overlay - Uses enigo (logical) + xcap (physical)

### Reference

See `src/executor/image_matching.rs` lines 11-15 for canonical documentation.
