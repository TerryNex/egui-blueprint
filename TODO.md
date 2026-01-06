# egui-blueprint TODO List

> **Last Updated:** 2026-01-07  
> **Status Legend:** `[ ]` Not Started | `[/]` In Progress | `[x]` Completed | `[!]` Blocked/Cannot Fix

---

## High Priority - Bugs

### Bug #5: Input Focus Conflict - Node Deletion on Typing

- **Status:** `[x]` ✅ COMPLETED
- **Issue:** When typing in an input field (e.g., variable value), pressing delete/backspace also deletes the selected `NodeType` instead of just removing characters from the input.
- **Solution:** Added focus check before processing delete key.

### Bug #8: SetVariable Port Spacing Breaks Connection Lines

- **Status:** `[x]` ✅ COMPLETED
- **Issue:** After adding spacing to `SetVariable` node ports, the connection lines are drawn at old positions instead of the actual port locations.
- **Solution:** Added 20px offset in `get_port_screen_pos()` for GetVariable/SetVariable nodes.

### Bug #10: Command+C Copy Not Working & Multi-Select Right-Click Issue

- **Status:** `[x]` ✅ COMPLETED
- **Issue:** When multiple nodes are selected, right-clicking to copy changes selection to only the right-clicked node. Command+C also not working.
- **Solution:** Added Copy event check and ctrl modifier, preserved multi-selection on right-click.

### Bug #13: Duplicate NodeType "Divide"

- **Status:** `[x]` ✅ COMPLETED
- **Issue:** There are two `Divide` node types appearing in the node menu.
- **Solution:** Removed duplicate entries from node options list.

### Bug #19: Canvas Pan Causes Line/Node Position Drift

- **Status:** `[x]` ✅ COMPLETED
- **Issue:** When dragging canvas upward or leftward, connection lines and NodeTypes have positional drift.
- **Solution:** Added VIRTUAL_OFFSET (5000, 5000) to keep all coordinates positive during rendering.

### Bug #25: Node Drag Performance Issue (NEW)

- **Status:** `[ ]`
- **Issue:** After fixing drag selection, dragging two overlapping nodes causes lag/stutter.
- **Root Cause:** Current implementation checks `dragging_node.is_none()` for each node, causing performance issues.
- **Proposed Solution:** Instead of updating z-order during drag, update it only on mouse release (bring-to-front on drop).

### Bug #26: Collision-Based Node Pushing (Alternative Approach)

- **Status:** `[ ]`
- **Issue:** Alternative to z-order layering - prevent node overlap by pushing nodes away.
- **Implementation:** Detect collision during drag and push blocking nodes out of the way.
- **Complexity:** Medium-High

---

## High Priority - Features

### Feature #1: Selection Box / Node Group (UE5-style BP Group)

- **Status:** `[ ]`
- **Issue:** Need to implement a selection box/group feature similar to UE5 Blueprint groups.
- **Complexity:** High

### Feature #2: Secondary Canvas Center for Reset

- **Status:** `[ ]`
- **Issue:** Canvas reset position should center on all existing NodeTypes rather than origin (0,0).
- **Complexity:** Low-Medium

### Feature #3: Font Size Settings

- **Status:** `[ ]`
- **Issue:** Add ability to adjust font size in the editor.
- **Complexity:** Low

### Feature #4: Auto-Load Last Used Script on Startup

- **Status:** `[ ]`
- **Issue:** Program should automatically load the last used script on startup.
- **Complexity:** Low

### Feature #6: Node Z-Order by Last Click

- **Status:** `[x]` ✅ COMPLETED
- **Issue:** Nodes should be ordered by last-clicked (recently clicked on top) rather than creation order.
- **Solution:** Added `z_order` field to Node, sorted rendering by z_order, updated on click/drag.

### Feature #7: Editable Node Title Names

- **Status:** `[x]` ✅ COMPLETED (Display name support added)
- **Issue:** Node titles should be editable. Also display custom names in Nodes List panel.
- **Solution:** Added `display_name: Option<String>` field, UI shows custom name if set.

---

## Medium Priority - Features

### Feature #9: SetVariable String Input Support

- **Status:** `[x]` ✅ COMPLETED
- **Issue:** SetVariable should support string type input.
- **Note:** Already implemented via Variable Type context menu.

### Feature #11/23: Mouse Scroll Wheel Zoom

- **Status:** `[x]` ✅ COMPLETED
- **Issue:** Mouse scroll wheel should support zoom in/out.
- **Solution:** Added raw_scroll_delta handling for zoom with cursor-centered zoom.

### Feature #12: Highlight Selected Node in Nodes List

- **Status:** `[x]` ✅ COMPLETED
- **Issue:** Clicking a node in the Nodes List should select and highlight the corresponding NodeType.
- **Solution:** Added selection on click, visual highlight, and pan to node.

### Feature #14: Divide by Zero Protection

- **Status:** `[x]` ✅ COMPLETED
- **Issue:** Divide node should not allow division by 0.
- **Solution:** If divisor is 0, use 1 instead.

### Feature #15: Colored Log Output with Variable Highlighting

- **Status:** `[x]` ✅ COMPLETED (Already exists)
- **Note:** Log output already parses `{variable}` patterns with colored segments.

### Feature #16: Type Selection for Arithmetic Nodes

- **Status:** `[x]` ✅ COMPLETED
- **Note:** Context menu allows switching between Float/Integer types.

---

## Medium Priority - Logic Features

### Feature #17: Branch Node Clarification & Comparison Nodes

- **Status:** `[x]` ✅ COMPLETED
- **Issue:** Need comparison nodes: `>`, `<`, `==`, `>=`, `<=`, `!=`
- **Solution:** Added all comparison nodes with execution logic. Branch condition uses to_bool() (>0 for numbers, "true" for strings).

### Feature #18: Loop Nodes (For, While, Count)

- **Status:** `[/]` IN PROGRESS (Port definitions added, execution pending)
- **Issue:** Need loop control flow nodes for automation scripts.
- **Done:** ForLoop, WhileLoop port definitions added
- **TODO:** Implement loop execution logic in executor

### Feature #22: Delay Node

- **Status:** `[/]` IN PROGRESS (Port definition added, execution pending)
- **Issue:** Need a delay/wait node for timed automation.
- **Done:** Delay node type and ports added
- **TODO:** Implement thread::sleep execution

---

## UI/UX Improvements

### Feature #20: Disable Command+/-/0 Window Zoom, Map to Canvas Zoom

- **Status:** `[x]` ✅ COMPLETED
- **Issue:** Command+ `+`/`-`/`0` should control canvas zoom, not window zoom.
- **Solution:** Keyboard shortcuts already mapped to canvas zoom in editor.rs.

### Feature #21: Convert Nodes Panel to Collapsible Window

- **Status:** `[x]` ✅ COMPLETED
- **Issue:** Nodes panel should be an `egui::Window` so it can be collapsed/minimized.
- **Solution:** Converted SidePanel to egui::Window with .collapsible(true).

---

## Automation Foundation

### Feature #24: Analysis of Missing Automation Script Nodes

- **Status:** `[/]` IN PROGRESS

#### Control Flow

- [x] Entry - Start point
- [x] Branch - Conditional
- [x] ForLoop - Port definitions done
- [x] WhileLoop - Port definitions done
- [ ] Sequence - Execute multiple flows in order
- [ ] Gate - On/Off flow control

#### Variables

- [x] GetVariable
- [x] SetVariable

#### Math

- [x] Add, Subtract, Multiply, Divide
- [ ] Modulo (%)
- [ ] Power (^)
- [ ] Abs, Min, Max, Clamp
- [ ] Random

#### Comparison

- [x] Equals
- [x] NotEquals ✅ NEW
- [x] GreaterThan
- [x] GreaterThanOrEqual ✅ NEW
- [x] LessThan
- [x] LessThanOrEqual ✅ NEW

#### Logic

- [x] And ✅ NEW
- [x] Or ✅ NEW
- [x] Not ✅ NEW
- [ ] Xor

#### String Operations (Missing)

- [ ] Concat
- [ ] Split
- [ ] Length
- [ ] Contains
- [ ] Replace
- [ ] Format (with placeholders)

#### Type Conversion

- [x] ToInteger
- [x] ToFloat
- [x] ToString

#### I/O

- [x] Print (BlueprintFunction)
- [ ] ReadInput
- [ ] FileRead
- [ ] FileWrite

#### Timing

- [x] Delay ✅ (Port definitions done)

---

## Low Priority / Nice to Have

- [ ] Hot-reloading for scripts
- [x] Node search/filter in add menu (already exists via text input)
- [ ] Minimap for large graphs
- [ ] Node comments/annotations
- [ ] Undo/Redo improvements
- [ ] Export to standalone executable
- [ ] Node presets/templates

---

## Cannot Fix / Deferred

*(No items currently)*

---

## Completed Summary (This Session)

1. ✅ Bug #5: Input Focus Conflict
2. ✅ Bug #10: Multi-Select Right-Click Copy
3. ✅ Bug #13: Duplicate Divide
4. ✅ Feature #6: Node Z-Order
5. ✅ Feature #7: Editable Node Titles
6. ✅ Feature #11/23: Scroll Wheel Zoom
7. ✅ Feature #12: Highlight in Nodes List
8. ✅ Feature #14: Divide by Zero Protection
9. ✅ Feature #17: Comparison Nodes (6 nodes added)
10. ✅ Feature #17: Logic Nodes (And, Or, Not)
11. ✅ Feature #18: Loop Node Types (ports defined)
12. ✅ Feature #20: Keyboard Zoom Mapping
13. ✅ Feature #21: Collapsible Nodes Window
14. ✅ Feature #22: Delay Node (ports defined)

---

## Remaining Work

### Bugs

- Bug #8: SetVariable Port Spacing (connection line positions)
- Bug #19: Canvas Pan Position Drift

### Features

- Feature #1: Node Groups
- Feature #2: Canvas Center Reset
- Feature #3: Font Size Settings
- Feature #4: Auto-Load Last Script
- Feature #18: Loop Execution Logic
- Feature #22: Delay Execution Logic

---

## Notes

1. **Priority Order:** Focus on bugs first, then high-priority features
2. **Testing:** Run `cargo check` before each commit
3. **Documentation:** Update README.md when adding new node types
4. **Changelog:** Record all changes in CHANGELOG.md
