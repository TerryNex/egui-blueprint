# egui-blueprint TODO List

> **Last Updated:** 2026-01-07  
> **Status Legend:** `[ ]` Not Started | `[/]` In Progress | `[x]` Completed | `[!]` Blocked/Cannot Fix
> **Agent Assignment:** `🤖 Agent: [NAME]` - Claimed by AI agent

---

## 🔧 Task Modules for Multi-Agent Development

> Each module is **independent** and can be developed in parallel.  
> Agents should **only modify files relevant to their module**.  
> Use `🤖 Agent: [NAME]` to claim a module.

---

## Module A: Desktop Input Automation

**🤖 Agent:** `🤖 Agent: Antigravity-A`  
**Files:** `node_types.rs`, `executor.rs`, `editor.rs` (node finder only)  
**Dependencies:** None

### Tasks

- [x] `Click` node - Click at screen coordinates (x, y)
- [x] `DoubleClick` node - Double-click at coordinates
- [x] `RightClick` node - Right-click at coordinates
- [x] `MouseMove` node - Move cursor to coordinates
- [x] `MouseDown/MouseUp` nodes - Explicit press/release
- [x] `Scroll` node - Mouse wheel simulation
- [x] `KeyPress` node - Press and release a key
- [x] `KeyDown/KeyUp` nodes - Explicit key press/release
- [x] `TypeText` node - Type a string of text
- [x] `HotKey` node - Key combinations (Ctrl+C, etc.)

### Implementation Notes

- Use `enigo` crate for cross-platform input simulation
- Add to `Cargo.toml`: `enigo = "0.2"` ✅ Done

---

## Module B: Touch Automation (Mobile)

**🤖 Agent:** `[ UNCLAIMED ]`  
**Files:** `node_types.rs`, `executor.rs`, `editor.rs` (node finder only)  
**Dependencies:** Module A (for base patterns)

### Tasks

- [ ] `Tap` node - Single tap at coordinates
- [ ] `DoubleTap` node - Double tap
- [ ] `LongPress` node - Press and hold
- [ ] `Swipe` node - Swipe from point A to B
- [ ] `Pinch` node - Pinch zoom gesture
- [ ] `MultiTouch` node - Multiple simultaneous touches

### Implementation Notes

- Platform-specific: Android (ADB), iOS (separate implementation)
- May need different execution path for mobile vs desktop

---

## Module C: Screenshot & Image Tools

**🤖 Agent:** `🤖 Agent: Antigravity-C`  
**Files:** `node_types.rs`, `executor.rs`, `main.rs` (UI for region select)  
**Dependencies:** None

### Tasks

- [x] `ScreenCapture` node - Capture full screen
- [ ] `RegionSelect` UI - Visual box selection overlay
- [x] `SaveScreenshot` - Auto-save to project folder
- [ ] `ImageLibrary` UI - Browse saved images

### Implementation Notes

- Use `screenshots` crate for screen capture
- Add to `Cargo.toml`: `screenshots = "0.8"` or `xcap = "0.0.8"`

---

## Module D: Image Recognition

**🤖 Agent:** `🤖 Agent: Antigravity-D`  
**Files:** `node_types.rs`, `executor.rs`, new file `image_match.rs`  
**Dependencies:** Module C (for screenshots)

### Tasks

- [x] `FindImage` node - Template matching on screen
- [x] `WaitForImage` node - Wait until image appears
- [x] `GetPixelColor` node - Get color at coordinates
- [x] `FindColor` node - Search for color in region
- [x] `WaitForColor` node - Wait for color to appear
- [x] `ImageSimilarity` - Fuzzy matching with threshold

### Implementation Notes

- Use `image` crate for pixel operations
- Template matching: compare RGB values with tolerance
- Optimize: sample grid instead of every pixel

---

## Module E: Recording System

**🤖 Agent:** `[ UNCLAIMED ]`  
**Files:** `main.rs`, `editor.rs`, new file `recorder.rs`  
**Dependencies:** Module A (needs Click nodes to exist)

### Tasks

- [ ] Recording UI - Start/Stop buttons in toolbar
- [ ] Single Action Recording - Capture one click as node
- [ ] Continuous Recording - Record until stop
- [ ] Auto-Add Nodes - Create nodes from recorded actions
- [ ] Smart Placement - Avoid overlapping existing nodes
- [ ] Record to Group - Wrap in Node Group

### Implementation Notes

- Use system-level input hooks to capture events
- Convert captured events to corresponding node types

---

## Module F: Node Groups / Functions

**🤖 Agent:** `[ UNCLAIMED ]`  
**Files:** `graph.rs`, `editor.rs`, `node_types.rs`  
**Dependencies:** None (but Recording may use this)

### Tasks

- [ ] Group visual box - Resizable rectangle around nodes
- [ ] Group drag - Move all contained nodes together
- [ ] Group collapse - Collapse to single node
- [ ] Function ports - Input/output parameters
- [ ] Function call node - Invoke a group as function

### Implementation Notes

- `NodeGroup` struct already exists in `graph.rs`
- Draw groups BEHIND nodes in render order

---

## Module G: System Control

**🤖 Agent:** `🤖 Antigravity`  
**Files:** `node_types.rs`, `executor.rs`  
**Dependencies:** None

### Tasks

- [x] `RunCommand` node - Execute shell command
- [x] `LaunchApp` node - Launch application
- [x] `CloseApp` node - Close application window
- [x] `FocusWindow` node - Bring window to foreground
- [x] `GetWindowPosition` node - Get window coordinates
- [x] `SetWindowPosition` node - Move/resize window

### Implementation Notes

- Use `std::process::Command` for shell commands
- Window manipulation: platform-specific APIs

### Pending API Implementation (Module G)

- [x] `FocusWindow` - Implemented with AppleScript on macOS, wmctrl on Linux
- [x] `GetWindowPosition` - Implemented with AppleScript on macOS, xdotool on Linux
- [x] `SetWindowPosition` - Implemented with AppleScript on macOS, wmctrl on Linux

---

## Module H: Data Operations

**🤖 Agent:** `🤖 Agent: Claude-H` ✅ COMPLETED  
**Files:** `node_types.rs`, `executor.rs`, `graph.rs` (for new data types)  
**Dependencies:** None

### Tasks

- [x] Array type support - Push, Pop, Get, Set, Length
- [x] `JSONParse` node - Parse JSON string
- [x] `JSONStringify` node - Convert to JSON string
- [x] `HTTPRequest` node - GET/POST requests
- [x] `ArrayCreate` node - Create empty array
- [x] `ArrayGet` node - Get element by index
- [x] `ArrayLength` node - Get array length

### Implementation Notes

- Extended `VariableValue` enum with `Array(Vec<VariableValue>)` variant
- Extended `DataType` enum with `Array` variant
- Used `serde_json` for JSON operations (already available in project)
- Used `curl` command for HTTP requests (portable, no additional dependencies)

---

## Completed Bugs (Reference)

### Bug #5, #8, #10, #13, #19, #27 - All COMPLETED ✅

See individual entries in CHANGELOG.md for details.

### Bug #25: Node Drag Performance - `[ ]` Pending

### Bug #26: Collision-Based Node Pushing - `[ ]` Pending

### Bug #28: Node Context Menu Conflict - `[ ]` Pending

**Problem:** Right-clicking on a node shows the node's context menu (Rename, Copy, etc.) but it flashes and disappears because the background's context menu (Create Group from Selection) is also triggered simultaneously.

**Root Cause:** The background context menu uses `ui.interact(clip_rect, ...)` which covers the entire canvas, including nodes. Even with `interaction_consumed` check, the context menus conflict.

**Potential Solutions:**
1. Check if pointer is over any node before showing background context menu
2. Use a different mechanism for the background context menu (e.g., only trigger on truly empty areas)
3. Add a small delay or priority system for context menus


---

## How to Claim a Module

1. Edit this file
2. Replace `[ UNCLAIMED ]` with your agent name, e.g., `🤖 Agent: Claude-A`
3. Only work on files listed in your module
4. Mark tasks with `[x]` when complete
5. Update CHANGELOG.md with your changes

---

## Low Priority / Nice to Have

- [ ] Hot-reloading for scripts
- [x] Node search/filter in add menu
- [ ] Minimap for large graphs
- [ ] Node comments/annotations
- [ ] Undo/Redo improvements
- [ ] Export to standalone executable
- [ ] Node presets/templates
