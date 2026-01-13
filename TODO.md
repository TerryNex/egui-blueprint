# egui-blueprint TODO List

> **Last Updated:** 2026-01-11  
> **Status Legend:** `[ ]` Not Started | `[/]` In Progress | `[x]` Completed | `[!]` Blocked/Cannot Fix  
> **Completed Tasks:** See [COMPLETED.md](./COMPLETED.md) for archived completed tasks

---

## 🔧 Priority: Performance & Stability

### Memory & Performance Optimization

- [ ] **Memory leak on script load** - Memory usage increases when loading different scripts
  - [ ] Investigate if old graph data is not being properly dropped
  - [ ] Consider clearing caches/buffers when loading new script
  - [ ] Profile memory allocations during load/unload cycles
  - [ ] Implement explicit cleanup of old script resources

### Execution Control

- [x] **Force stop infinite loops** - Add ability to stop runaway `while true` loops
  - [x] Display running threads in Debug window (via stop button visibility)
  - [x] Add "Force Stop" button to terminate execution
  - [x] Implement thread handle tracking in executor
  - [x] Add timeout/watchdog for long-running operations (checks stop flag each iteration)

---

## 🎨 UI/UX Improvements

### Style & Theming

- [x] **Complete node category colors** - Add missing colors in settings.json
  - [x] Add colors for: String, Logic, Comparison, Control Flow, I/O, Conversion, Screenshot, Image Recognition, Input Automation
  - [x] Ensure Nodes List window uses category colors (uses EditorStyle)
  - [x] Use pure colors (red, yellow, blue, green, cyan, purple) with transparency variations

- [x] **Style settings reset** - Add reset button to restore default colors
  - [x] Store default style configuration (EditorStyle::default())
  - [x] Add "Reset to Defaults" button in Style Settings window

### Output Log

- [x] **Export log to file** - Allow saving debug logs
  - [x] Add "Export" button to Output Log window
  - [x] Option to choose save location (scripts/logs/)
  - [x] Quick export to Desktop option
  - [x] Include timestamp in exported filename

---

## 📦 Pending Modules

### Module B: Touch Automation (Mobile)

**🤖 Agent:** `[ UNCLAIMED ]`

- [ ] `Tap` node - Single tap at coordinates
- [ ] `DoubleTap` node - Double tap
- [ ] `LongPress` node - Press and hold
- [ ] `Swipe` node - Swipe from point A to B
- [ ] `Pinch` node - Pinch zoom gesture
- [ ] `MultiTouch` node - Multiple simultaneous touches

### Module E: Recording System (Remaining)

- [ ] Record to Group - Wrap recorded actions in Node Group

### Module C: Screenshot & Image Tools (Remaining)

- [ ] `RegionSelect` UI - Visual box selection overlay
- [ ] `ImageLibrary` UI - Browse saved images

---

## 🐛 Known Bugs

### Bug #25: Node Drag Performance - `[ ]` Pending

Large graphs may experience lag during node drag operations.

### Bug #26: Collision-Based Node Pushing - `[ ]` Pending

Nodes do not push each other when overlapping.

### Bug #27: Cursor Info Color Display Incorrect - `[ ]` Pending

When hovering over colored areas, the displayed color in the Cursor Info tooltip is incorrect:

- Colors flicker between gray/white/black and the actual color when moving mouse
- When mouse is stationary, always shows `#6C6C6C` (gray)
- **Root Cause Analysis**: Likely XCap captures the app window itself (including the tooltip) rather than the underlying screen content. Need to capture screen without the app window, or delay capture.
- **Possible Solution**: Use system screenshot API that excludes current app, or temporarily hide tooltip during capture.

### Bug #28: Cursor Info Tooltip Limited to App Window - `[ ]` Pending

The Cursor Info tooltip only appears when cursor is inside the application window.

- Tooltip should follow cursor globally on the entire screen (like screenshot tools)
- **Root Cause**: egui Area positioning is relative to the app window, not the entire screen
- **Possible Solution**: Create a separate borderless/transparent overlay window that follows cursor globally

---

## 💡 Low Priority / Nice to Have

- [ ] Hot-reloading for scripts
- [ ] Minimap for large graphs
- [ ] Node comments/annotations
- [ ] Undo/Redo improvements
- [ ] Export to standalone executable
- [ ] Node presets/templates

---

## How to Claim a Module

1. Edit this file
2. Replace `[ UNCLAIMED ]` with your agent name, e.g., `🤖 Agent: Claude-A`
3. Only work on files listed in your module
4. Mark tasks with `[x]` when complete
5. Update CHANGELOG.md with your changes
