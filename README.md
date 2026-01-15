# egui-blueprint

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](https://www.rust-lang.org/)
[![egui](https://img.shields.io/badge/egui-0.33.3-blue.svg)](https://github.com/emilk/egui)

> A visual programming system inspired by Unreal Engine Blueprints, built with Rust and egui. Create powerful automation scripts through an intuitive node-based interface.

![egui-blueprint Screenshot](https://via.placeholder.com/800x400?text=egui-blueprint+Screenshot)

## 🎯 Executive Summary

**egui-blueprint** is a cross-platform visual programming environment that enables users to build automation workflows without writing code. Through a drag-and-drop node editor, users can create complex scripts for:

- 🖱️ Desktop automation (mouse/keyboard control)
- 🖼️ Image recognition and template matching
- 📊 Data processing (arrays, JSON, HTTP)
- 🪟 Window management
- 📸 Screen capture and manipulation
- 🎬 Input event recording

**Key Highlights:**
- ✅ **Zero-code automation**: Build scripts visually with nodes
- ✅ **Real-time execution**: See your blueprint run with visual feedback
- ✅ **Cross-platform**: Runs on macOS, Linux, and Windows
- ✅ **Extensible**: Easy to add new node types
- ✅ **Recording**: Capture mouse/keyboard events as nodes
- ✅ **Visual debugging**: Watch nodes execute in real-time

## 📚 Table of Contents

- [Features](#-features)
- [Getting Started](#-getting-started)
- [Usage Examples](#-usage-examples)
- [Architecture](#-architecture)
- [Development Guide](#-development-guide)
- [Design Decisions](#-design-decisions)
- [Technology Stack](#-technology-stack)
- [AI Assistance Disclosure](#-ai-assistance-disclosure)
- [Contributing](#-contributing)
- [Contact & Support](#-contact--support)
- [Acknowledgments](#-acknowledgments)

## ✨ Features

### Module A: Desktop Input Automation

Control mouse and keyboard programmatically for UI testing, automation, and RPA workflows.

| Node | Description | Inputs | Outputs |
|------|-------------|--------|---------|
| `Click` | Left-click at coordinates | X, Y | Next (flow) |
| `DoubleClick` | Double-click at coordinates | X, Y | Next (flow) |
| `RightClick` | Right-click at coordinates | X, Y | Next (flow) |
| `MouseMove` | Move cursor to position | X, Y | Next (flow) |
| `MouseDown` | Press mouse button | X, Y, Button | Next (flow) |
| `MouseUp` | Release mouse button | X, Y, Button | Next (flow) |
| `Scroll` | Mouse wheel scroll | X, Y, Amount | Next (flow) |
| `KeyPress` | Press and release key | Key | Next (flow) |
| `KeyDown` | Press key (hold) | Key | Next (flow) |
| `KeyUp` | Release key | Key | Next (flow) |
| `TypeText` | Type text string | Text | Next (flow) |
| `TypeString` | Type with character delay | Text, Delay | Next (flow) |
| `HotKey` | Key combination (Ctrl+C) | Modifiers, Key | Next (flow) |

**Features:**
- Cross-platform input simulation via `enigo` crate
- Special key support (F1-F12, arrows, Enter, Escape, Tab, etc.)
- Modifier combinations (Ctrl, Shift, Alt, Meta/Command)
- Configurable typing speed

### Module C: Screenshot & Image Tools

Capture screen content for image processing, OCR, or visual testing.

| Node | Description | Inputs | Outputs |
|------|-------------|--------|---------|
| `ScreenCapture` | Capture full screen | Display | Next, Image |
| `RegionCapture` | Capture screen region | X, Y, Width, Height | Next, Image |
| `SaveScreenshot` | Save image to file | Image, Path | Next, Success |

**Features:**
- Multi-monitor support
- Region selection with visual overlay
- Screenshots saved to `scripts/screenshots/`
- Template images to `scripts/templates/`
- Interactive 📸 Capture button with region selector

### Module D: Image Recognition

Template matching and color detection for visual automation.

| Node | Description | Inputs | Outputs |
|------|-------------|--------|---------|
| `GetPixelColor` | Get RGB at coordinates | X, Y | Next, R, G, B |
| `FindColor` | Search for color in region | Region, Color, Tolerance | Next, X, Y, Found |
| `WaitForColor` | Wait for color to appear | Region, Color, Timeout | Next, X, Y, Success |
| `FindImage` | Template matching | Template, Region, Tolerance | Next, X, Y, Found |
| `WaitForImage` | Wait for image to appear | Template, Region, Timeout | Next, X, Y, Success |
| `ImageSimilarity` | Compare two images | Image1, Image2, Tolerance | Similarity |

**Features:**
- **NCC Algorithm**: Normalized Cross-Correlation for robust matching
- **Multi-scale search**: Handles Retina/HiDPI displays
- **Tolerance control**: 0 (exact) to 255 (any match)
- **Image thumbnails**: Visual preview on FindImage nodes
- **Template library**: Browse images from `scripts/templates/`

### Module E: Input Recording

Record user interactions and replay them as blueprints.

| Feature | Description |
|---------|-------------|
| **Smart Click Detection** | Quick clicks (< 200ms) → `Click` node; drags → `MouseDown`/`MouseUp` |
| **Cursor Info Overlay** | Real-time display of cursor position and pixel color |
| **Record Delays** | Auto-insert `Delay` nodes between actions |
| **Record Moves** | Optional mouse movement recording |
| **Auto-grouping** | Recorded nodes wrapped in visual groups |

**Workflow:**
1. Click 🔴 Record button
2. Perform actions (click, type, move mouse)
3. Click ⏹ Stop to generate blueprint
4. Edit and customize the generated nodes

### Module G: System Control

Launch applications, run commands, and manage windows.

| Node | Description | Inputs | Outputs |
|------|-------------|--------|---------|
| `RunCommand` | Execute shell command | Command | Next, Output, ExitCode |
| `LaunchApp` | Open application | AppName | Next, Success |
| `CloseApp` | Terminate application | AppName | Next, Success |
| `FocusWindow` | Bring window to front | WindowTitle | Next, Success |
| `GetWindowPosition` | Get window bounds | WindowTitle | Next, X, Y, Width, Height |
| `SetWindowPosition` | Move/resize window | WindowTitle, X, Y, W, H | Next, Success |

**Platform Support:**
- **macOS**: AppleScript (`osascript`)
- **Linux**: `wmctrl`, `xdotool`
- **Windows**: Win32 API

### Module H: Data Operations

Process arrays, JSON, and HTTP data.

| Node | Description | Inputs | Outputs |
|------|-------------|--------|---------|
| `ArrayCreate` | Create empty array | - | Array |
| `ArrayPush` | Append to array | Variable, Value | Next, Array |
| `ArrayPop` | Remove last element | Variable | Next, Value, Array |
| `ArrayGet` | Get element by index | Array/Variable, Index | Value |
| `ArraySet` | Set element at index | Variable, Index, Value | Next, Array |
| `ArrayLength` | Get array size | Array/Variable | Length |
| `JSONParse` | Parse JSON string | JSON | Next, Value |
| `JSONStringify` | Convert to JSON | Value | Next, JSON |
| `HTTPRequest` | Make HTTP request | URL, Method, Body | Next, Response, Success |

**Features:**
- Dynamic array manipulation with chaining
- REST API integration (GET, POST, PUT, DELETE, etc.)
- JSON serialization/deserialization
- Variable-based or direct array operations

### Additional Modules

#### Control Flow
- `ForLoop`, `WhileLoop`, `ForEachLine` - Iteration
- `Branch` - Conditional execution
- `Sequence` - Execute multiple flows in order
- `Gate` - On/off flow control
- `WaitForCondition` - Block until condition is true
- `Delay` - Timed pause

#### Math & Logic
- **Math**: Add, Subtract, Multiply, Divide, Modulo, Power, Abs, Min, Max, Clamp, Random
- **Comparison**: Equals, NotEquals, GreaterThan, GreaterThanOrEqual, LessThan, LessThanOrEqual
- **Logic**: And, Or, Not, Xor

#### String Operations
- `Concat`, `Split`, `Length`, `Contains`, `Replace`, `Format`
- `StringJoin` - Dynamic concatenation with auto-expand inputs
- `StringBetween` - Extract between delimiters
- `StringTrim` - Whitespace trimming with modes
- `ExtractAfter`, `ExtractUntil` - Pattern-based extraction

#### I/O
- `FileRead`, `FileWrite` - File operations
- `ReadInput` - User input prompts

#### Variables
- `GetVariable`, `SetVariable` - Variable management
- Persistent context across execution

## 🚀 Getting Started

### Prerequisites

1. **Rust Toolchain** (stable channel)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **System Dependencies**

   **macOS:**
   ```bash
   xcode-select --install
   ```

   **Linux (Ubuntu/Debian):**
   ```bash
   sudo apt-get install libxcb1-dev libxrandr-dev libxcursor-dev
   ```

   **Windows:**
   - No additional dependencies required

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/TerryNex/egui-blueprint.git
   cd egui-blueprint
   ```

2. **Build the project**
   ```bash
   cargo build --release
   ```

3. **Run the application**
   ```bash
   cargo run --release
   ```

### Your First Script: Simple Click Automation

Let's create a blueprint that clicks at specific coordinates.

1. **Launch egui-blueprint**: `cargo run`

2. **Add nodes** (Right-click on canvas or use the toolbar):
   - Add `Entry` node (execution starting point)
   - Add `Click` node from **Input Automation** category

3. **Connect nodes**:
   - Drag from `Entry`'s **Execute** port to `Click`'s **Execute** port

4. **Configure Click node**:
   - Set X: `500`
   - Set Y: `300`

5. **Run the blueprint**:
   - Click ▶️ **Run** button in toolbar
   - Watch as the cursor moves and clicks at (500, 300)

6. **Save your work**:
   - File → Save As → `my_first_script.json`

**Result:** You've created a reusable automation script! Modify coordinates, add delays, or chain multiple clicks together.

## 💡 Usage Examples

### Example 1: Web Form Automation

```
Entry → Delay(1000) → Click(100, 200) → TypeText("john@example.com")
  → Click(100, 250) → TypeText("password123") → Click(150, 300)
```

**Workflow:**
1. Wait 1 second
2. Click email field
3. Type email
4. Click password field
5. Type password
6. Click submit button

### Example 2: Template Matching Loop

```
Entry → SetVariable(found=false) → WhileLoop(Condition: NOT found)
  → FindImage(template="button.png") → SetVariable(found=Found)
  → Branch(found) → [True: Click(X, Y)] [False: Delay(500)]
```

**Workflow:**
1. Initialize `found` variable
2. Loop until image is found
3. Search for template image
4. If found, click at location
5. If not found, wait 500ms and retry

### Example 3: Data Processing with HTTP

```
Entry → HTTPRequest(URL="https://api.example.com/data", Method="GET")
  → JSONParse(Response) → ArrayGet(data, 0)
  → GetVariable(name="value") → SetVariable(result)
```

**Workflow:**
1. Fetch data from API
2. Parse JSON response
3. Get first array element
4. Extract value field
5. Store in result variable

### Example 4: Screen Monitoring

```
Entry → ForLoop(Start=0, End=10)
  → GetPixelColor(X=100, Y=100)
  → Branch(R > 200 AND G < 50 AND B < 50)
  → [True: Click(100, 100)] → Delay(1000)
```

**Workflow:**
1. Loop 10 times
2. Check pixel color at (100, 100)
3. If pixel is red-ish, click it
4. Wait 1 second between iterations

## 🏗️ Architecture

### Project Structure

```
egui-blueprint/
├── src/
│   ├── main.rs              # Application entry, UI layout (~1734 lines)
│   ├── graph.rs             # Graph data structures (Node, Connection, Variable)
│   ├── node_types.rs        # NodeType and DataType enums
│   ├── history.rs           # Undo/Redo stack
│   ├── editor/              # Visual graph editor
│   │   ├── mod.rs          # Node rendering, interaction (~2994 lines)
│   │   └── utils.rs        # Geometry utilities, color helpers
│   ├── executor/            # Blueprint execution engine
│   │   ├── mod.rs          # Main execution loop (~4305 lines)
│   │   ├── context.rs      # Variable storage during execution
│   │   ├── flow_control.rs # ForLoop, WhileLoop execution
│   │   ├── node_eval.rs    # Node output evaluation
│   │   ├── automation.rs   # Mouse/keyboard automation
│   │   ├── image_recognition.rs # Screen capture, FindImage
│   │   ├── helpers.rs      # Type conversions (to_bool, to_float)
│   │   ├── json_helpers.rs # JSON parsing/stringification
│   │   ├── image_matching.rs # Template matching algorithms
│   │   └── type_conversions.rs # Type casting
│   └── recorder/            # Input event recording
│       └── mod.rs
├── scripts/
│   ├── screenshots/         # Captured screenshots
│   ├── templates/           # Template images for FindImage
│   └── logs/               # Exported log files
├── Cargo.toml
├── README.md
├── CHANGELOG.md
└── CONTRIBUTING.md
```

### Execution Flow

1. **Graph Loading**: Deserialize blueprint from JSON
2. **Entry Point**: Find `Entry` node and begin execution
3. **Flow Execution**: Follow execution flow connections
4. **Node Evaluation**: Recursively evaluate input ports
5. **Type Conversion**: Convert between data types as needed
6. **Output Storage**: Store node outputs in execution context
7. **Loop Handling**: Execute loop bodies with iteration tracking
8. **Error Handling**: Log errors and continue execution when possible

### Data Flow

- **ExecutionFlow**: White connections, control program flow
- **Data**: Colored connections based on type (Integer, String, Boolean, etc.)
- **Variables**: Persistent storage across nodes using `GetVariable`/`SetVariable`
- **Outputs**: Cached in context with `__out_{node_id}_{port}` keys

## 🛠️ Development Guide

### Adding New Node Types

See the detailed guide in [CONTRIBUTING.md](CONTRIBUTING.md#adding-new-node-types) for step-by-step instructions.

**Quick Overview:**
1. Define node type in `src/node_types.rs`
2. Add port definitions in `src/editor/mod.rs`
3. Add to node finder menu
4. Implement execution logic in `src/executor/mod.rs`
5. Assign category color
6. Test your node

### Testing Workflow

Since the project uses manual testing:

1. **Build**: `cargo build`
2. **Run**: `cargo run`
3. **Create test blueprint** with your new node
4. **Execute** and verify outputs
5. **Check logs** for errors
6. **Test edge cases** (invalid inputs, missing connections)

### Project Structure Reference

| Component | File | Responsibility |
|-----------|------|----------------|
| **UI Layout** | `main.rs` | Application window, toolbar, panels |
| **Graph Editor** | `editor/mod.rs` | Node rendering, connections, interactions |
| **Execution Engine** | `executor/mod.rs` | Blueprint interpretation and execution |
| **Node Definitions** | `node_types.rs` | NodeType and DataType enums |
| **Data Structures** | `graph.rs` | Node, Connection, Variable structs |
| **Undo/Redo** | `history.rs` | Command pattern for history |
| **Automation** | `executor/automation.rs` | Mouse/keyboard nodes |
| **Image Recognition** | `executor/image_recognition.rs` | Screen capture, FindImage |
| **Recording** | `recorder/mod.rs` | Input event capture |

## 🎨 Design Decisions

### Why Rust?

**Performance**: Automation scripts often involve tight loops and real-time processing. Rust's zero-cost abstractions and native performance ensure minimal overhead.

**Safety**: Memory safety without garbage collection prevents crashes and undefined behavior in long-running automation tasks.

**Cross-platform**: Excellent cross-platform support with crates like `enigo`, `xcap`, and `rdev`.

**Concurrency**: Safe concurrent execution for recording, execution, and UI rendering.

### Why egui?

**Immediate Mode**: Perfect for dynamic, interactive node editors where the graph structure changes frequently.

**Lightweight**: Fast startup and minimal dependencies compared to web-based solutions.

**Cross-platform**: Single codebase for desktop platforms (macOS, Linux, Windows).

**Customizable**: Full control over rendering and interaction logic.

**No JavaScript**: Pure Rust solution without web tech overhead.

### Architecture Benefits

✅ **Separation of Concerns**: Editor, Executor, and Recorder are independent modules  
✅ **Extensibility**: Easy to add new node types with minimal code changes  
✅ **Testability**: Core logic isolated from UI for easier testing  
✅ **Debuggability**: Visual execution highlighting shows program flow  

### Trade-offs

⚖️ **No Web UI**: Desktop-only (trade-off for native performance)  
⚖️ **Manual Testing**: No automated test suite (planned for future)  
⚖️ **Platform Differences**: Some nodes behave differently on each OS  
⚖️ **Learning Curve**: Rust's steep learning curve for contributors  

## 🔧 Technology Stack

| Category | Crate | Version | Purpose |
|----------|-------|---------|---------|
| **UI Framework** | `egui` | 0.33.3 | Immediate mode GUI |
| | `eframe` | 0.33.3 | Native window management |
| | `egui_extras` | 0.33.3 | Extra widgets (image loaders) |
| **Serialization** | `serde` | 1.0.228 | Trait-based serialization |
| | `serde_json` | 1.0.148 | JSON serialization |
| **Input Automation** | `enigo` | 0.2 | Cross-platform input simulation |
| **Screen Capture** | `xcap` | 0.8 | Cross-platform screen capture |
| **Image Processing** | `image` | 0.25 | Image loading/manipulation |
| | `imageproc` | 0.25 | Template matching (NCC) |
| **Input Recording** | `rdev` | (rustdesk fork) | Input event monitoring |
| **Async** | `crossbeam-channel` | 0.5 | Thread-safe channels |
| | `rayon` | 1.11.0 | Parallel processing |
| **Utilities** | `uuid` | 1.19.0 | Unique IDs for nodes |
| | `anyhow` | 1.0.100 | Error handling |
| | `log` | 0.4.29 | Logging facade |
| | `env_logger` | 0.11.8 | Logger implementation |
| | `chrono` | 0.4.42 | Date/time operations |
| | `dirs` | 5.0 | Home directory detection |
| | `sysinfo` | 0.35 | System monitoring |

### Platform-Specific Notes

**macOS:**
- Input automation requires Accessibility permissions
- Window management uses AppleScript
- Retina display handling with coordinate scaling

**Linux:**
- Requires X11 libraries (`libxcb`, `libxrandr`)
- Window management via `wmctrl`, `xdotool`
- Wayland support limited (via XWayland)

**Windows:**
- Uses Win32 API for input and windows
- May require administrator privileges for some operations

## 🤖 AI Assistance Disclosure

This project has been developed with significant AI assistance (Claude, GitHub Copilot) for rapid prototyping and iteration.

### Development Statistics

- **Total Lines of Code**: ~15,000 lines of Rust
- **Core Modules**: 8 automation modules (A, C, D, E, G, H + Math, String, Control Flow)
- **Node Types**: 100+ node types
- **Development Time**: ~2 weeks (with AI assistance)
- **AI Contribution**: ~70% code generation, 30% human refinement

### Ethics Statement

AI has been used responsibly:
- ✅ All AI-generated code reviewed and understood
- ✅ Architecture decisions made by human developers
- ✅ Code adheres to Rust best practices and idioms
- ✅ Full transparency about AI assistance
- ✅ No proprietary code used in AI training (to our knowledge)

### Quality Assurance

Despite AI assistance, this project maintains high standards:
- All code manually tested
- Follows Rust conventions and clippy lints
- Comprehensive error handling
- Cross-platform compatibility verified
- Performance optimizations applied

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

**Quick Checklist:**
- [ ] Code follows Rust conventions and project style
- [ ] All code and comments are in English
- [ ] Changes have been manually tested
- [ ] CHANGELOG.md has been updated
- [ ] Commit messages follow conventional commits format
- [ ] Documentation updated (if applicable)

**Popular Contribution Areas:**
- Adding new node types
- Platform-specific fixes (macOS/Linux/Windows)
- Performance optimizations
- Documentation improvements
- Bug fixes

## 📞 Contact & Support

- **Issues**: [GitHub Issues](https://github.com/TerryNex/egui-blueprint/issues)
- **Discussions**: [GitHub Discussions](https://github.com/TerryNex/egui-blueprint/discussions)
- **Pull Requests**: [Contributing Guide](CONTRIBUTING.md)

**Found a bug?** Please open an issue with:
- Blueprint file (`.json`) if applicable
- Steps to reproduce
- Expected vs. actual behavior
- Platform (macOS/Linux/Windows) and version

**Have a feature request?** Open a discussion or issue describing:
- Use case and motivation
- Proposed solution or API
- Any alternative approaches considered

## 🙏 Acknowledgments

This project would not be possible without these excellent open-source projects:

- **[egui](https://github.com/emilk/egui)** - The amazing immediate mode GUI library
- **[enigo](https://github.com/enigo-rs/enigo)** - Cross-platform input automation
- **[xcap](https://github.com/nashaofu/xcap)** - Screen capture made easy
- **[imageproc](https://github.com/image-rs/imageproc)** - Robust template matching
- **[rdev](https://github.com/rustdesk-org/rdev)** - Input event recording
- **Rust Community** - For excellent documentation and crates

Special thanks to:
- **Unreal Engine** - For the blueprint visual programming paradigm
- **AI Assistants** - Claude and GitHub Copilot for accelerating development
- **Contributors** - Everyone who has submitted issues, PRs, and feedback

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Built with ❤️ using Rust and egui**

**Star this repo** if you find it useful! ⭐
