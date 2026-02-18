# Research: Drag-and-Drop Session Builder

**Feature**: 026-drag-drop-builder
**Date**: 2026-02-18

## R1: Drag-and-Drop API Choice

**Decision**: Use the Pointer Events API (`pointerdown`/`pointermove`/`pointerup`/`pointercancel`) instead of the HTML5 Drag and Drop API.

**Rationale**: The HTML5 DnD API does not fire `dragstart` from touch interactions on iOS Safari and has inconsistent behaviour on Android Chrome. Every major drag-and-drop library (SortableJS, dnd-kit, react-beautiful-dnd) uses pointer or touch events internally. The Pointer Events API provides a unified input model across mouse, touch, and stylus with broad browser support (iOS Safari 13+, all modern desktop browsers).

**Alternatives considered**:
- HTML5 Drag and Drop API — rejected due to broken mobile touch support
- Touch Events API (`touchstart`/`touchmove`/`touchend`) — viable but Pointer Events supersede it and provide the same coverage with a simpler API (`pointerType` property distinguishes mouse/touch/pen)
- External JS drag library via wasm-bindgen interop — rejected per project constraint (no external JS libraries)

## R2: Touch Scroll vs Drag Conflict Resolution

**Decision**: Restrict drag initiation to a dedicated drag handle element. Apply `touch-action: none` and `user-select: none` CSS on the handle only. Require a 5px movement threshold before committing to drag mode.

**Rationale**: On touch devices, the browser must distinguish scroll gestures from drag gestures. If the entire row is draggable, scrolling breaks. By limiting drag initiation to a small handle element (~44x44px), the rest of the row and surrounding area remain scrollable. `touch-action: none` on the handle tells the browser not to perform native scroll/zoom for touches starting on that element.

**Alternatives considered**:
- Long-press to initiate drag (no handle) — rejected because it conflicts with browser context menu and creates ambiguity between scroll-intent and drag-intent
- Drag from anywhere on desktop, handle-only on mobile — considered but FR-008 restricts to handle on desktop too for consistency and to avoid accidentally dragging when trying to click arrow buttons

## R3: Visual Feedback During Drag

**Decision**: Use a "drop indicator line" approach (no visual clone/ghost). The source item receives a subtle highlight (opacity, border change). A coloured horizontal line appears between entries at the computed target position.

**Rationale**: Moving the dragged element visually with the pointer (`translateY`) is complex, causes layout shifts that interfere with position calculations, and requires managing z-index/absolute positioning. The drop-indicator-only approach is simpler, matches modern UI patterns (Notion, Linear), and provides clear feedback with minimal code.

**Alternatives considered**:
- Ghost element following pointer with `transform: translateY` — rejected due to layout shift complexity
- HTML5 DnD ghost image via `setDragImage()` — not available with Pointer Events API
- Opacity dim on source + reorder animation on drop — this IS the chosen approach

## R4: Pointer Capture for Reliable Tracking

**Decision**: Use `Element.setPointerCapture(pointerId)` on the drag handle element after `pointerdown` to ensure all subsequent `pointermove`/`pointerup` events fire on the handle element, even if the pointer moves outside it.

**Rationale**: Without pointer capture, if the user's finger/mouse moves off the drag handle during drag, `pointermove` events stop firing on the handle. Pointer capture redirects all events for that pointer to the capturing element until `pointerup` or `releasePointerCapture`.

**Alternatives considered**:
- Attach listeners to `document` instead — works but is less clean; requires manual cleanup and can interfere with other interactions

## R5: Reusable Hook Architecture

**Decision**: Create a `use_drag_reorder` hook in `crates/intrada-web/src/hooks/use_drag_reorder.rs` that encapsulates drag state, pointer event handling, and position computation. The hook accepts a reorder callback and returns signals consumed by the view layer.

**Rationale**: Drag-and-drop reordering is used in two places: the session builder setlist (which dispatches `ReorderSetlist` core events) and the routine edit page entry list (which updates a local `RwSignal<Vec>`). Extracting shared logic into a hook avoids ~100 lines of duplicated pointer event handling while allowing each consumer to provide its own reorder operation.

**Alternatives considered**:
- Inline all logic in each component — rejected due to duplication between setlist builder and routine edit
- A `<DraggableList>` wrapper component — more complex to integrate with existing component APIs and Leptos's rendering model

## R6: web-sys Feature Requirements

**Decision**: Add the following web-sys features to `intrada-web/Cargo.toml`: `PointerEvent`, `Element`, `DomRect`, `HtmlElement`.

**Rationale**: These are the minimum features needed for pointer event handling (`PointerEvent`), position computation (`Element::get_bounding_client_rect()` → `DomRect`), and pointer capture (`HtmlElement::set_pointer_capture()`). The existing `Window`, `Storage`, `ClipboardEvent`, `DataTransfer` features are preserved.

**Alternatives considered**:
- Adding `Document` and `CssStyleDeclaration` — not needed since we use Leptos `class:` bindings for CSS toggling rather than direct style manipulation
