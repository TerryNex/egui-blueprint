# Issue Tracking & Solutions Reference

> **Purpose:** Record difficult bugs and their solutions for future reference.

---

## Resolved Critical Issues

### Issue: Node Position Drift When Outside Viewport

**Symptom:**

- Nodes moved beyond left/top edge of canvas caused visual position drift
- Connection lines remained correct (endpoints at proper port positions)
- Later-created nodes followed earlier nodes' accumulated offset
- Only occurred when approaching left/top edges, not right/bottom

**Root Cause:**

- `ui.interact()` and `ui.scope_builder()` for inline editors participate in egui's layout cursor system
- When nodes are outside the visible clip_rect, these calls accumulate offset that affects subsequent nodes
- Connection lines use only `ui.painter()` which bypasses layout system entirely

**Solution:**

```rust
// Wrap each node's draw_node() in isolated UI context
let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(clip_rect));
self.draw_node(&mut child_ui, node, ...);
```

**Key Insight:** For canvas-style editors with pan/zoom:

- Use `ui.painter()` for all visual drawing (coordinates independent)
- When using `ui.interact()` or widgets, isolate each element in `new_child()` to prevent layout accumulation

---

### Issue: Drag Without Pre-Selection

**Symptom:** Users had to click to select a node first, then drag. Expected to drag immediately on mouse down.

**Root Cause:** Selection logic triggered on `clicked` (mouse up) instead of `pressed` (mouse down).

**Solution:**

```rust
// Change from: if clicked && !clicked_port
// To: if pressed && !clicked_port
if pressed && !clicked_port {
    // Handle selection immediately on mouse down
}
```

---

### Issue: Space Key Opens Node Finder While Editing

**Symptom:** Pressing space while editing node name or text field opens node finder menu.

**Solution:**

```rust
let any_text_edit_has_focus = ui.ctx().memory(|m| m.focused().is_some());
if input_space && self.node_finder.is_none() && self.editing_node_name.is_none() && !any_text_edit_has_focus {
    // Open node finder
}
```

---

## egui Canvas Editor Best Practices

1. **Coordinate System:** Use VIRTUAL_OFFSET (e.g., 5000.0, 5000.0) to keep all coordinates positive
2. **Drawing:** Use `ui.painter()` methods which are clip-independent
3. **Interaction:** Wrap each interactive element in `ui.new_child()` to isolate layout
4. **Hit Testing:** For complex cases, use manual `rect.contains(pointer_pos)` checks
5. **Context Menus:** Keep `ui.interact()` for `response.context_menu()` support
