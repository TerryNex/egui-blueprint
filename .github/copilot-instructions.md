# GitHub Copilot Instructions for egui-blueprint

This file provides context and guidelines for GitHub Copilot when assisting with code in this project.

## Project Overview

**egui-blueprint** is a visual programming system similar to Unreal Engine Blueprints, built with Rust and egui. It allows users to create automation scripts through a node-based visual editor.

### Technology Stack

- **Language**: Rust (edition 2024)
- **UI Framework**: egui 0.33.3 + eframe
- **Graphics**: Native egui rendering
- **Serialization**: serde + serde_json
- **Input Automation**: enigo 0.2
- **Screen Capture**: xcap 0.8
- **Image Processing**: image 0.25, imageproc 0.25
- **Input Recording**: rdev (rustdesk fork)
- **Async**: crossbeam-channel, rayon

### Target Platforms

- macOS (primary development platform)
- Linux (supported with xcb/xrandr/wmctrl dependencies)
- Windows (supported, some features may differ)

### Architecture

```
egui-blueprint/
├── src/
│   ├── main.rs              # Application entry, UI layout, toolbar (~1734 lines)
│   ├── graph.rs             # Graph data structures (Node, Connection, Variable)
│   ├── node_types.rs        # NodeType and DataType enums
│   ├── history.rs           # Undo/Redo stack
│   ├── editor/              # Visual graph editor
│   │   ├── mod.rs          # Editor UI, node rendering, interaction (~2994 lines)
│   │   └── utils.rs        # Geometry utilities, color helpers
│   ├── executor/            # Blueprint execution engine
│   │   ├── mod.rs          # Main execution loop, node evaluation (~4305 lines)
│   │   ├── context.rs      # ExecutionContext (variable storage)
│   │   ├── flow_control.rs # ForLoop, WhileLoop execution
│   │   ├── node_eval.rs    # Node output evaluation
│   │   ├── automation.rs   # Mouse/keyboard automation nodes
│   │   ├── image_recognition.rs # Screen capture, FindImage, GetPixelColor
│   │   ├── helpers.rs      # Type conversions (to_bool, to_float, etc.)
│   │   ├── json_helpers.rs # JSON parsing/stringification
│   │   ├── image_matching.rs # Template matching algorithms
│   │   └── type_conversions.rs # Type casting
│   └── recorder/            # Input event recording
│       └── mod.rs
├── scripts/
│   ├── screenshots/         # Captured screenshots
│   ├── templates/           # Template images for FindImage
│   └── logs/               # Exported log files
└── Cargo.toml
```

## Code Style & Conventions

### English-Only Requirement

**CRITICAL: All code, comments, documentation, and log messages MUST be written in English.**

```rust
// ✅ CORRECT
fn get_pixel_color(x: i32, y: i32) -> Option<(u8, u8, u8)> {
    log::info!("Getting pixel at ({}, {})", x, y);
}

// ❌ INCORRECT - Do not use non-English
fn 获取像素颜色(x: i32, y: i32) -> Option<(u8, u8, u8)> {
    log::info!("获取像素在 ({}, {})", x, y);
}
```

### Naming Conventions

- **Types/Structs/Enums**: `PascalCase` (e.g., `NodeType`, `ExecutionContext`, `VariableValue`)
- **Functions/Variables**: `snake_case` (e.g., `evaluate_node`, `node_count`, `get_pixel_color`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_LOOP_ITERATIONS`, `DEFAULT_DELAY_MS`)
- **Private fields**: Prefix with `_` when appropriate (e.g., `_internal_cache`)

### Error Handling Patterns

**Never use `.unwrap()` in production code.** Always handle errors gracefully:

```rust
// ✅ CORRECT - Explicit error handling
match result {
    Ok(value) => {
        log::info!("Operation successful");
        Some(value)
    }
    Err(e) => {
        log::error!("Operation failed: {}", e);
        None
    }
}

// ✅ CORRECT - Question mark operator for early return
fn process() -> Option<Image> {
    let image = image::open(path).ok()?;
    Some(image.resize(100, 100, FilterType::Nearest))
}

// ❌ INCORRECT - Unwrap causes crashes
let image = image::open(path).unwrap();
```

### Documentation Standards

Document all public APIs and complex logic:

```rust
/// Evaluates a node's output port and returns its value.
///
/// This function recursively evaluates upstream nodes to compute
/// the requested output value.
///
/// # Arguments
/// * `node_id` - The node to evaluate
/// * `port_name` - Name of the output port
/// * `graph` - The blueprint graph
/// * `context` - Execution context with variables
/// * `depth` - Recursion depth (for cycle detection)
///
/// # Returns
/// * `Some(VariableValue)` - The computed value
/// * `None` - If evaluation fails or port doesn't exist
fn evaluate_node_output(
    node_id: NodeId,
    port_name: &str,
    graph: &Graph,
    context: &mut ExecutionContext,
    depth: usize,
) -> Option<VariableValue> {
    // Implementation
}
```

## Architecture Guidelines

### Module Responsibilities

| Module | Purpose | Key Files |
|--------|---------|-----------|
| **Editor** | Visual graph editor UI, node rendering, user interaction | `editor/mod.rs`, `editor/utils.rs` |
| **Executor** | Blueprint execution engine, node evaluation | `executor/mod.rs`, `executor/flow_control.rs` |
| **Automation** | Mouse/keyboard input simulation | `executor/automation.rs` |
| **Image Recognition** | Screen capture, template matching, color detection | `executor/image_recognition.rs` |
| **Recorder** | Record user input events as blueprint nodes | `recorder/mod.rs` |
| **Graph** | Core data structures (Node, Connection, Variable) | `graph.rs` |
| **Node Types** | Node and data type definitions | `node_types.rs` |

### Adding New Node Types (Step-by-Step)

#### 1. Define the Node Type (`src/node_types.rs`)

```rust
pub enum NodeType {
    // ... existing types
    
    /// Your new node - describe what it does
    YourNewNode,
}
```

#### 2. Define Ports (`src/editor/mod.rs` in `get_ports_for_type`)

```rust
NodeType::YourNewNode => {
    let inputs = vec![
        Port {
            name: "Execute".to_string(),
            data_type: DataType::ExecutionFlow,
        },
        Port {
            name: "Value".to_string(),
            data_type: DataType::Integer,
        },
    ];
    let outputs = vec![
        Port {
            name: "Next".to_string(),
            data_type: DataType::ExecutionFlow,
        },
        Port {
            name: "Result".to_string(),
            data_type: DataType::String,
        },
    ];
    (inputs, outputs)
}
```

#### 3. Add to Node Finder Menu (`src/editor/mod.rs` in `show_node_finder`)

```rust
if ui.button("YourNewNode").clicked() {
    let node = Node {
        id: NodeId::new_v4(),
        node_type: NodeType::YourNewNode,
        position: graph_editor.screen_to_graph(menu_pos),
        z_order: graph_editor.next_z_order,
        display_name: None,
    };
    graph_editor.next_z_order += 1;
    graph.nodes.push(node);
    *show_finder = false;
}
```

#### 4. Implement Execution Logic (`src/executor/mod.rs`)

**For flow nodes** (with execution flow), add to `execute_flow`:

```rust
NodeType::YourNewNode => {
    // Evaluate inputs
    let value = evaluate_node_output(node_id, "Value", &graph, context, depth + 1)
        .and_then(|v| to_integer(&v))
        .unwrap_or(0);
    
    // Perform logic
    let result = format!("Processed: {}", value);
    log::info!("YourNewNode: {}", result);
    
    // Store outputs
    context.variables.insert(
        format!("__out_{}_{}", node_id, "Result"),
        VariableValue::String(result)
    );
    
    // Continue flow
    execute_next_flow(node_id, "Next", graph, context, depth, stop_flag);
}
```

**For value nodes** (no execution flow), add to `evaluate_node_output`:

```rust
NodeType::YourNewNode => {
    if port_name == "Result" {
        let input = evaluate_node_output(node_id, "Value", graph, context, depth + 1)?;
        let value = to_integer(&input)?;
        Some(VariableValue::String(format!("Value: {}", value)))
    } else {
        None
    }
}
```

#### 5. Add Node Category Color (`src/main.rs` in `EditorStyle::default`)

```rust
pub fn default() -> Self {
    let mut category_colors = HashMap::new();
    // ... existing colors
    category_colors.insert("YourCategory".to_string(), Color32::from_rgb(100, 200, 150));
    // ...
}
```

#### 6. Assign Category (`src/editor/mod.rs` in node rendering)

```rust
let category = match node.node_type {
    // ... existing mappings
    NodeType::YourNewNode => "YourCategory",
    // ...
};
```

## Common Patterns

### Node Output Storage

Flow nodes store outputs in the context using a special key format:

```rust
// Store output for a node
context.variables.insert(
    format!("__out_{}_{}", node_id, "OutputPortName"),
    VariableValue::String(result)
);

// Retrieve stored output later
let stored = context.variables.get(&format!("__out_{}_{}", node_id, "Result"));
```

### Input Evaluation

Always evaluate inputs before using them:

```rust
// Evaluate single input
let x = evaluate_node_output(node_id, "X", &graph, context, depth + 1)
    .and_then(|v| to_integer(&v))
    .unwrap_or(0);

// Multiple inputs
let source = evaluate_node_output(node_id, "Source", &graph, context, depth + 1)
    .and_then(|v| to_string(&v))
    .unwrap_or_default();

let keyword = evaluate_node_output(node_id, "Keyword", &graph, context, depth + 1)
    .and_then(|v| to_string(&v))
    .unwrap_or_default();
```

### Type Conversion

Use helper functions from `executor/helpers.rs`:

```rust
use crate::executor::helpers::{to_bool, to_integer, to_float, to_string};

let bool_val = to_bool(&variable_value);       // Option<bool>
let int_val = to_integer(&variable_value);     // Option<i64>
let float_val = to_float(&variable_value);     // Option<f64>
let string_val = to_string(&variable_value);   // Option<String>
```

### Error Logging

Always log errors for debugging:

```rust
match operation() {
    Ok(result) => {
        log::info!("Operation succeeded: {:?}", result);
        Some(result)
    }
    Err(e) => {
        log::error!("Operation failed: {}", e);
        None
    }
}
```

### Flow Control

Execute the next node in the flow:

```rust
// Simple next flow
execute_next_flow(node_id, "Next", graph, context, depth, stop_flag);

// Conditional flow (Branch)
if condition {
    execute_next_flow(node_id, "True", graph, context, depth, stop_flag);
} else {
    execute_next_flow(node_id, "False", graph, context, depth, stop_flag);
}

// Multiple sequential flows (Sequence)
execute_next_flow(node_id, "Out1", graph, context, depth, stop_flag);
execute_next_flow(node_id, "Out2", graph, context, depth, stop_flag);
```

## Testing Guidelines

### Manual Testing

Since the project doesn't have automated tests, always:

1. **Run the application**: `cargo run`
2. **Create a test blueprint** using your new node
3. **Test with various inputs**:
   - Valid inputs
   - Edge cases (empty strings, zero, negative numbers)
   - Invalid/missing inputs
4. **Check the log output** for errors
5. **Verify outputs** are correct

### Unit Test Pattern (if adding tests)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_evaluation() {
        let mut context = ExecutionContext::new();
        context.variables.insert("test_var".to_string(), VariableValue::Integer(42));
        
        let result = evaluate_some_function(&context);
        assert_eq!(result, Some(VariableValue::Integer(84)));
    }
}
```

### Integration Test Pattern

```rust
// Create a test blueprint programmatically
let mut graph = Graph::default();

// Add nodes
let entry_node = Node {
    id: NodeId::new_v4(),
    node_type: NodeType::Entry,
    position: Pos2::ZERO,
    z_order: 0,
    display_name: None,
};
graph.nodes.push(entry_node);

// Add connections
let conn = Connection {
    id: ConnectionId::new_v4(),
    from_node: entry_node.id,
    from_port: "Execute".to_string(),
    to_node: next_node.id,
    to_port: "Execute".to_string(),
};
graph.connections.push(conn);

// Execute and verify
let mut context = ExecutionContext::new();
execute_graph(&graph, &mut context);
```

## Performance Considerations

### Image Processing

- **Screen capture is expensive**: Cache screenshots when possible
- **Template matching is CPU-intensive**: Use appropriate tolerance values
- **Region capture is faster**: Prefer `RegionCapture` over full `ScreenCapture`

```rust
// ✅ CORRECT - Capture only needed region
let region = capture_region(x, y, width, height)?;

// ❌ SLOW - Capture full screen then crop
let full_screen = capture_full_screen()?;
let region = crop_image(&full_screen, x, y, width, height);
```

### UI Rendering

- **Don't update every frame**: Use `ctx.request_repaint_after()` for timed updates
- **Cache computed values**: Store expensive calculations in state
- **Limit log output**: Too many log entries can slow down the UI

### Execution Engine

- **Prevent infinite loops**: Use `MAX_LOOP_ITERATIONS` constant
- **Check stop flag**: Allow user to interrupt long-running operations
- **Avoid deep recursion**: Track `depth` parameter in evaluation

```rust
const MAX_LOOP_ITERATIONS: usize = 1000;

for i in 0..MAX_LOOP_ITERATIONS {
    if stop_flag.load(Ordering::Relaxed) {
        log::info!("Loop stopped by user");
        break;
    }
    // Loop body
}
```

## Platform-Specific Notes

### macOS

- **Input automation**: Uses Core Graphics, requires Accessibility permissions
- **Screen capture**: Works with `xcap` out of the box
- **Window management**: Uses AppleScript (`osascript`)
- **Retina displays**: Coordinates may need scaling (usually handled automatically)

### Linux

- **Dependencies**: Requires `libxcb`, `libxrandr`, `libxcursor`
- **Window management**: Uses `wmctrl`, `xdotool`
- **Screen capture**: Uses X11 or Wayland (via `xcap`)
- **Input automation**: Uses X11 test extension

### Windows

- **Input automation**: Uses Windows API (via `enigo`)
- **Screen capture**: Uses Windows Graphics Capture API
- **Window management**: Uses Win32 API
- **Permissions**: May require running as administrator

## Common Issues & Solutions

### Issue: FindImage not working

**Cause**: Template image scale mismatch (Retina displays)

**Solution**: Use multi-scale pyramid matching (already implemented in NCC algorithm)

### Issue: Mouse automation fails

**Cause**: Missing accessibility permissions on macOS

**Solution**: Grant accessibility permissions in System Preferences → Security & Privacy → Accessibility

### Issue: Screen capture returns empty image

**Cause**: Invalid screen index or display not found

**Solution**: Verify display index with `xcap::Window::all()` first

### Issue: Node execution freezes

**Cause**: Infinite loop without stop flag check

**Solution**: Add stop flag check in loop:

```rust
if stop_flag.load(Ordering::Relaxed) {
    break;
}
```

### Issue: Type conversion fails

**Cause**: Unexpected VariableValue variant

**Solution**: Use helper functions and provide defaults:

```rust
let value = to_integer(&var).unwrap_or(0);
```

## Git Workflow

### Commit Message Format

Follow Conventional Commits:

```
<type>(<scope>): <subject>

<optional body>

<optional footer>
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

**Scopes**: `editor`, `executor`, `nodes`, `automation`, `image`, `io`, `ui`

**Examples**:

```
feat(nodes): add GetTimestamp node for Unix timestamp output

fix(automation): correct MouseUp position during drag operations

docs: update README with image recognition module details

refactor(executor): extract helpers to separate module
```

### Branch Naming

- `feature/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation updates
- `refactor/description` - Code refactoring

## Key Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `egui` | 0.33.3 | Immediate mode GUI framework |
| `eframe` | 0.33.3 | Native window management for egui |
| `serde` | 1.0.228 | Serialization framework |
| `serde_json` | 1.0.148 | JSON serialization |
| `uuid` | 1.19.0 | Unique IDs for nodes/connections |
| `enigo` | 0.2 | Cross-platform input automation |
| `xcap` | 0.8 | Cross-platform screen capture |
| `image` | 0.25 | Image loading/processing |
| `imageproc` | 0.25 | Template matching (NCC algorithm) |
| `rdev` | (rustdesk fork) | Input event recording |
| `crossbeam-channel` | 0.5 | Async communication |
| `rayon` | 1.11.0 | Parallel processing |
| `dirs` | 5.0 | Cross-platform home directory |

## Questions to Ask Before Implementation

Before implementing a new feature or fix, consider:

1. **Does this affect cross-platform compatibility?** Test on multiple platforms if possible.
2. **Does this require new permissions?** (e.g., Accessibility on macOS)
3. **Is this a breaking change?** Will existing blueprints still work?
4. **Does this affect performance?** Consider optimization for expensive operations.
5. **Is error handling sufficient?** Never use `.unwrap()` without a good reason.
6. **Is the code documented?** Add doc comments for complex logic.
7. **Is everything in English?** Code, comments, logs must be English-only.
8. **Does this need a CHANGELOG entry?** Add to Unreleased section.

## Helpful Commands

### Development

```bash
# Build project
cargo build

# Run application
cargo run

# Run with debug logging
RUST_LOG=debug cargo run

# Check for errors without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy

# Build release version (optimized)
cargo build --release
```

### Debugging

```bash
# Run with verbose logging
RUST_LOG=trace cargo run

# Check dependencies
cargo tree

# Update dependencies
cargo update

# Clean build artifacts
cargo clean
```

### Git

```bash
# Create feature branch
git checkout -b feature/my-feature

# Check status
git status

# Stage changes
git add .

# Commit with conventional format
git commit -m "feat(nodes): add new node type"

# Push to remote
git push origin feature/my-feature
```

---

**Remember**: This project is a visual programming tool for automation. Focus on creating intuitive, reliable, and performant nodes that users can easily combine to build complex workflows.
