# Contributing to egui-blueprint

Thank you for your interest in contributing to egui-blueprint! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for everyone. We expect all contributors to:

- Use welcoming and inclusive language
- Be respectful of differing viewpoints and experiences
- Gracefully accept constructive criticism
- Focus on what is best for the community
- Show empathy towards other community members

### Unacceptable Behavior

- Trolling, insulting/derogatory comments, and personal or political attacks
- Public or private harassment
- Publishing others' private information without explicit permission
- Other conduct which could reasonably be considered inappropriate in a professional setting

## Getting Started

### Prerequisites

- **Rust**: Install from [rustup.rs](https://rustup.rs/) (stable channel recommended)
- **System Dependencies**:
  - macOS: Xcode Command Line Tools (`xcode-select --install`)
  - Linux: `libxcb`, `libxrandr`, `libxcursor` (for screen capture and input automation)
  - Windows: No additional dependencies required

### Development Environment Setup

1. **Clone the Repository**
   ```bash
   git clone https://github.com/TerryNex/egui-blueprint.git
   cd egui-blueprint
   ```

2. **Build the Project**
   ```bash
   cargo build
   ```

3. **Run the Application**
   ```bash
   cargo run
   ```

4. **Check Code Quality**
   ```bash
   cargo check
   cargo clippy
   ```

## Development Workflow

### Branch Creation

1. **Create a Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```
   or
   ```bash
   git checkout -b fix/issue-description
   ```

2. **Keep Your Branch Updated**
   ```bash
   git fetch origin
   git rebase origin/main
   ```

### Making Changes

1. **Write Clean Code**: Follow the code style guidelines below
2. **Test Your Changes**: Ensure existing functionality still works
3. **Document Your Changes**: Update relevant documentation
4. **Commit Frequently**: Make small, logical commits

### Testing Requirements

While this project doesn't have a comprehensive test suite yet, you should:

1. **Manual Testing**:
   - Run the application and verify your changes work as expected
   - Test edge cases and error conditions
   - Verify cross-platform compatibility when possible

2. **Node Testing** (for new node types):
   - Create a test blueprint that exercises your node
   - Test with various input combinations
   - Verify output values are correct
   - Test error handling (invalid inputs, missing connections)

3. **Integration Testing**:
   - Ensure your changes don't break existing nodes
   - Test interaction with related nodes
   - Verify the execution flow works correctly

## Code Style

### Rust Conventions

We follow standard Rust conventions with some project-specific guidelines:

#### Naming Conventions

- **Types & Structs**: `PascalCase`
  ```rust
  struct NodeType { }
  enum DataType { }
  ```

- **Functions & Variables**: `snake_case`
  ```rust
  fn evaluate_node(node_id: NodeId) -> Option<VariableValue> { }
  let node_count = graph.nodes.len();
  ```

- **Constants**: `SCREAMING_SNAKE_CASE`
  ```rust
  const MAX_LOOP_ITERATIONS: usize = 1000;
  const DEFAULT_DELAY_MS: u64 = 100;
  ```

- **Private Fields**: Prefix with underscore when needed
  ```rust
  struct ExecutionContext {
      variables: HashMap<String, VariableValue>,
      _internal_cache: Vec<u8>,
  }
  ```

#### Language Requirements

**All code, comments, documentation, and log messages MUST be in English.**

```rust
// ✅ CORRECT
fn get_pixel_color(x: i32, y: i32) -> Option<(u8, u8, u8)> {
    // Capture screenshot and extract pixel color
    log::info!("Getting pixel at ({}, {})", x, y);
}

// ❌ INCORRECT
fn 获取像素颜色(x: i32, y: i32) -> Option<(u8, u8, u8)> {
    // 捕获屏幕截图并提取像素颜色
    log::info!("获取像素在 ({}, {})", x, y);
}
```

#### Error Handling

Use proper error handling patterns:

```rust
// ✅ CORRECT - Explicit error handling
match screenshot_result {
    Ok(image) => {
        log::info!("Screenshot captured successfully");
        Some(image)
    }
    Err(e) => {
        log::error!("Failed to capture screenshot: {}", e);
        None
    }
}

// ✅ CORRECT - Early return on error
fn process_image(path: &str) -> Option<Image> {
    let image = image::open(path).ok()?;
    Some(image.resize(100, 100, FilterType::Nearest))
}

// ❌ INCORRECT - Unwrap in production code
let image = image::open(path).unwrap(); // Crashes on error!
```

#### Documentation Standards

Document all public APIs and complex logic:

```rust
/// Evaluates a node and returns its output value.
///
/// # Arguments
/// * `node_id` - The unique identifier of the node to evaluate
/// * `port_name` - The name of the output port to retrieve
/// * `context` - Execution context containing variables and state
///
/// # Returns
/// * `Some(VariableValue)` - The computed output value
/// * `None` - If the node cannot be evaluated or the port doesn't exist
///
/// # Examples
/// ```
/// let result = evaluate_node_output(
///     node_id,
///     "Result",
///     &mut context
/// );
/// ```
fn evaluate_node_output(
    node_id: NodeId,
    port_name: &str,
    context: &mut ExecutionContext,
) -> Option<VariableValue> {
    // Implementation...
}
```

For complex algorithms, add inline comments:

```rust
// Use normalized cross-correlation for robust template matching
// This handles brightness variations better than pixel-by-pixel comparison
let correlation = imageproc::template_matching::match_template(
    &haystack,
    &template,
    MatchTemplateMethod::CrossCorrelationNormalized,
);
```

### Code Formatting

Run `cargo fmt` before committing:

```bash
cargo fmt
```

### Linting

Fix any warnings from Clippy:

```bash
cargo clippy -- -W clippy::all
```

## Commit Guidelines

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification.

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

#### Types

- **feat**: New feature
- **fix**: Bug fix
- **docs**: Documentation changes
- **style**: Code style changes (formatting, missing semicolons, etc.)
- **refactor**: Code refactoring without changing functionality
- **test**: Adding or updating tests
- **chore**: Maintenance tasks (dependencies, build config, etc.)

#### Scope (Optional)

The scope specifies which part of the codebase is affected:

- `editor` - Editor UI and interaction
- `executor` - Blueprint execution engine
- `nodes` - Node type definitions
- `automation` - Input automation (mouse/keyboard)
- `image` - Image recognition and screenshot features
- `io` - File I/O and HTTP operations
- `ui` - General UI improvements

#### Subject

- Use imperative mood ("add" not "added")
- Don't capitalize the first letter
- No period at the end
- Keep it under 50 characters

#### Examples of Good Commit Messages

```
feat(nodes): add ForEachLine node for line-by-line text processing

Implements a new control flow node that iterates over multi-line text.
Each iteration outputs the current line content and 0-based index.
Works seamlessly with FileRead output for text file processing.

Closes #42
```

```
fix(automation): correct MouseUp position during drag operations

Previously, MouseUp events showed the same position as MouseDown.
Now uses enigo to query real-time cursor position, correctly showing
the release position for drag operations.
```

```
docs: update README with Module D image recognition features
```

```
refactor(executor): extract image matching to separate module

Moved template matching algorithms from mod.rs to image_matching.rs
for better code organization and AI analyzability.
```

```
chore(deps): update imageproc to 0.25 for NCC support
```

## Pull Request Process

### Before Submitting

- [ ] Code follows the style guidelines
- [ ] All code, comments, and documentation are in English
- [ ] Code has been formatted with `cargo fmt`
- [ ] No warnings from `cargo clippy`
- [ ] Changes have been manually tested
- [ ] Documentation has been updated (if applicable)
- [ ] CHANGELOG.md has been updated (add to Unreleased section)
- [ ] Commit messages follow the commit guidelines

### PR Description Template

```markdown
## Description
Brief description of what this PR does.

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Related Issues
Fixes #(issue number)

## Changes Made
- Change 1
- Change 2
- Change 3

## Testing
Describe how you tested your changes:
- [ ] Tested manually
- [ ] Created test blueprint
- [ ] Tested on multiple platforms (specify: macOS/Linux/Windows)

## Screenshots (if applicable)
Add screenshots or GIFs demonstrating the changes.

## Checklist
- [ ] My code follows the code style of this project
- [ ] All code is in English
- [ ] I have performed a self-review of my own code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have updated the documentation accordingly
- [ ] I have added an entry to CHANGELOG.md
- [ ] My changes generate no new warnings
- [ ] I have tested my changes manually
```

### Review Process

1. Submit your PR with a clear description
2. Respond to review comments promptly
3. Make requested changes in new commits (don't force push)
4. Once approved, a maintainer will merge your PR

## Adding New Node Types

This is one of the most common contributions. Here's a step-by-step guide:

### Step 1: Define the Node Type

Add your node type to `src/node_types.rs`:

```rust
pub enum NodeType {
    // ... existing types ...
    
    /// Your new node - describe what it does
    YourNodeName,
}
```

### Step 2: Define Ports

Add port definitions in `src/editor/mod.rs` in the `get_ports_for_type` function:

```rust
NodeType::YourNodeName => {
    let mut inputs = vec![
        Port {
            name: "Execute".to_string(),
            data_type: DataType::ExecutionFlow,
        },
        Port {
            name: "InputParam".to_string(),
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

### Step 3: Add to Node Finder

Add your node to the appropriate category in the node finder menu (in `src/editor/mod.rs`, `show_node_finder` function):

```rust
if ui.button("YourNodeName").clicked() {
    // Create and add the node
}
```

### Step 4: Implement Execution Logic

Add evaluation logic in `src/executor/mod.rs`:

**For flow nodes** (nodes with execution flow), add to `execute_flow`:

```rust
NodeType::YourNodeName => {
    // Get input values
    let input_param = evaluate_node_output(node_id, "InputParam", &graph, context, depth + 1)
        .and_then(|v| to_integer(&v));
    
    // Perform your logic
    let result = format!("Processed: {}", input_param.unwrap_or(0));
    
    // Store output
    context
        .variables
        .insert(format!("__out_{}_{}", node_id, "Result"), VariableValue::String(result));
    
    // Continue execution flow
    execute_next_flow(node_id, "Next", graph, context, depth, stop_flag);
}
```

**For value nodes** (nodes without execution flow), add to `evaluate_node_output`:

```rust
NodeType::YourNodeName => {
    if port_name == "Result" {
        let input = evaluate_node_output(node_id, "InputParam", graph, context, depth + 1)?;
        let value = to_integer(&input)?;
        Some(VariableValue::String(format!("Value: {}", value)))
    } else {
        None
    }
}
```

### Step 5: Test Your Node

1. Run the application: `cargo run`
2. Create a test blueprint using your new node
3. Connect inputs and execute
4. Verify outputs are correct
5. Test error cases (missing inputs, invalid values)

### Step 6: Update Documentation

1. Add an entry to `CHANGELOG.md` under the `[Unreleased]` section
2. If it's a major feature, consider updating the README

### Example: Complete Implementation

Here's a complete example of a simple "Multiply By Two" node:

```rust
// 1. Add to NodeType enum (node_types.rs)
MultiplyByTwo,

// 2. Add ports (editor/mod.rs in get_ports_for_type)
NodeType::MultiplyByTwo => {
    let inputs = vec![
        Port {
            name: "Value".to_string(),
            data_type: DataType::Integer,
        },
    ];
    let outputs = vec![
        Port {
            name: "Result".to_string(),
            data_type: DataType::Integer,
        },
    ];
    (inputs, outputs)
}

// 3. Add to node finder (editor/mod.rs in show_node_finder)
ui.label("Math");
if ui.button("MultiplyByTwo").clicked() {
    let node = Node {
        id: NodeId::new_v4(),
        node_type: NodeType::MultiplyByTwo,
        position: graph_editor.screen_to_graph(menu_pos),
        z_order: graph_editor.next_z_order,
        display_name: None,
    };
    graph_editor.next_z_order += 1;
    graph.nodes.push(node);
    *show_finder = false;
}

// 4. Implement evaluation (executor/mod.rs in evaluate_node_output)
NodeType::MultiplyByTwo => {
    if port_name == "Result" {
        let input = evaluate_node_output(node_id, "Value", graph, context, depth + 1)?;
        let value = to_integer(&input)?;
        Some(VariableValue::Integer(value * 2))
    } else {
        None
    }
}
```

## Contact Information

- **Issues**: [GitHub Issues](https://github.com/TerryNex/egui-blueprint/issues)
- **Discussions**: [GitHub Discussions](https://github.com/TerryNex/egui-blueprint/discussions)
- **Email**: For security issues, contact the maintainers privately

## Questions?

If you have questions about contributing:

1. Check existing issues and discussions
2. Create a new discussion thread
3. Ask in your pull request if it's specific to your changes

Thank you for contributing to egui-blueprint! 🎉
