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

---

### Issue: FindImage Retina Scaling & Mismatch

**Symptom:**

- `FindImage` fails to find template on Retina screens even when cropped from current screen (Case B)
- Or fails to find template when perfectly matched in `Quick Capture` (Case A) depending on scaling logic
- Mismatch between `screencapture` CLI (Logical pixels @1x) and `xcap` crate (Physical pixels @2x)

**Root Cause:**

1. **Resolution Mismatch:** `screencapture` saves logical pixels (e.g. 100x100), while `xcap` captures physical screen buffer (200x200). A direct pixel match fails.
2. **Coordinate Space:** `Enigo` (mouse control) expects Logical coordinates, but `xcap` image search returns Physical coordinates.
3. **Phase Mismatch:** Naive downscaling of `xcap` image (2x -> 1x) can introduce aliasing or phase shift if crop coordinates are odd numbers.

**Solution:**

**1. Multi-Scale Pyramid Search:**

```rust
// Pass 1: Try exact match (Scale 1x) - Handles Physical Template (Case A)
if let Some((x, y)) = search_on_buffer(screen, 1) {
    if screen.width() > 2000 {
        return (x / 2, y / 2, true); // Convert Physical -> Logical
    }
    return (x, y, true);
}

// Pass 2: Try Downscaled match (Scale 2x) - Handles Logical Template (Case B)
if screen.width() > 2000 {
    let downscaled = image::imageops::resize(screen, ...);
    if let Some((x, y)) = search_on_buffer(&downscaled, 2) {
         // downscaled is already logical size
        return (x, y, true);
    }
}
```

**2. Coordinate Normalization:**

- Always return **Logical Coordinates** from the executor.
- If match found in Physical pass on Retina, divide x,y by 2.

**Key Insight:**

- Never assume screen capture and template have same density.
- Always try original scale first (preserves data), then gracefully degrade to downscaled search.
- UI Automation/Mouse control always operates in Logical space (Points), while Image Processing usually operates in Physical space (Pixels).
