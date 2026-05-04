use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, PointerEvent};

/// Transient state for an in-progress drag operation.
#[derive(Clone, Debug)]
pub struct DragState {
    /// The `id` of the entry being dragged.
    pub dragged_entry_id: String,
    /// Pointer capture ID (from `PointerEvent.pointer_id()`).
    pub pointer_id: i32,
    /// Y coordinate when pointer went down.
    pub start_y: f64,
    /// Current Y coordinate during drag.
    pub current_y: f64,
    /// Index the dragged entry started at.
    pub source_index: usize,
    /// Index where the entry would be inserted if dropped now.
    pub hover_index: usize,
    /// Pixel height of the source row at drag-start. Used to translate
    /// surrounding rows by exactly one slot when the dragged row passes
    /// them — gives the iOS UITableView reorder feel.
    pub source_height: f64,
    /// Whether the 5px movement threshold has been exceeded.
    pub committed: bool,
}

/// Return value from `use_drag_reorder`.
#[derive(Clone)]
pub struct DragReorderReturn {
    /// Signal that is `true` while a drag is in progress (and committed past threshold).
    pub is_dragging: Signal<bool>,
    /// The `id` of the entry currently being dragged, or `None`.
    pub dragged_id: Signal<Option<String>>,
    /// The source index of the dragged entry (its original position), or `None`.
    pub source_index: Signal<Option<usize>>,
    /// The hover (target) index where the row would be inserted if released now, or `None`.
    pub hover_index: Signal<Option<usize>>,
    /// Live Y-offset (px) of the dragged row from its original position. Drives
    /// `transform: translateY(...)` on the source row so it follows the finger.
    pub live_offset_y: Signal<f64>,
    /// Pixel height of the dragged source row, for translating siblings by
    /// exactly one slot during reflow.
    pub source_height: Signal<f64>,
    /// Closure to attach to `on:pointerdown` on each drag handle.
    /// Call with `(entry_id, source_index, event)`.
    pub on_pointer_down: Callback<(String, usize, PointerEvent)>,
}

/// Movement threshold in pixels before drag is committed.
const DRAG_THRESHOLD_PX: f64 = 5.0;

/// Creates a reusable drag-and-drop reorder hook.
///
/// # Arguments
///
/// * `on_reorder` – callback invoked with `(entry_id, new_position)` when a valid drop occurs.
/// * `item_count` – reactive signal of the current item count (for clamping).
/// * `container_ref` – `NodeRef` of the container element whose children are the draggable rows.
pub fn use_drag_reorder(
    on_reorder: Callback<(String, usize)>,
    item_count: Signal<usize>,
    container_ref: NodeRef<leptos::html::Div>,
) -> DragReorderReturn {
    let drag_state: RwSignal<Option<DragState>> = RwSignal::new(None);

    let is_dragging =
        Signal::derive(move || drag_state.get().map(|s| s.committed).unwrap_or(false));

    let dragged_id = Signal::derive(move || {
        drag_state.get().and_then(|s| {
            if s.committed {
                Some(s.dragged_entry_id)
            } else {
                None
            }
        })
    });

    let hover_index = Signal::derive(move || {
        drag_state.get().and_then(|s| {
            if s.committed {
                Some(s.hover_index)
            } else {
                None
            }
        })
    });

    let source_index = Signal::derive(move || {
        drag_state.get().and_then(|s| {
            if s.committed {
                Some(s.source_index)
            } else {
                None
            }
        })
    });

    let live_offset_y = Signal::derive(move || {
        drag_state
            .get()
            .filter(|s| s.committed)
            .map(|s| s.current_y - s.start_y)
            .unwrap_or(0.0)
    });

    let source_height = Signal::derive(move || {
        drag_state
            .get()
            .filter(|s| s.committed)
            .map(|s| s.source_height)
            .unwrap_or(0.0)
    });

    // --- Register window-level pointer event listeners ---
    // We use Closure::forget() to leak the closures (same pattern as session_timer.rs).
    // These listeners live for the component's lifetime. In a WASM SPA this is acceptable.
    {
        let pointer_move_handler: Closure<dyn Fn(PointerEvent)> =
            Closure::new(move |ev: PointerEvent| {
                drag_state.update(|state| {
                    let Some(s) = state.as_mut() else { return };

                    // Only respond to the captured pointer
                    if ev.pointer_id() != s.pointer_id {
                        return;
                    }

                    s.current_y = ev.client_y() as f64;

                    // Check threshold (FR-007 / T018: 5px movement before committing)
                    if !s.committed {
                        let dy = (s.current_y - s.start_y).abs();
                        if dy < DRAG_THRESHOLD_PX {
                            return;
                        }
                        s.committed = true;
                    }

                    // Compute hover index from container children bounding rects.
                    // Pass the source_index so we can adjust for the "gap" left by
                    // the dragged item (avoids off-by-one when dragging downward).
                    let new_hover = compute_hover_index(
                        &container_ref,
                        s.current_y,
                        item_count.get_untracked(),
                        s.source_index,
                    );
                    s.hover_index = new_hover;
                });
            });

        let pointer_up_handler: Closure<dyn Fn(PointerEvent)> =
            Closure::new(move |ev: PointerEvent| {
                let current = drag_state.get_untracked();
                let Some(state) = current else { return };

                if ev.pointer_id() != state.pointer_id {
                    return;
                }

                if state.committed && state.hover_index != state.source_index {
                    on_reorder.run((state.dragged_entry_id.clone(), state.hover_index));
                }

                drag_state.set(None);
            });

        let pointer_cancel_handler: Closure<dyn Fn(PointerEvent)> =
            Closure::new(move |ev: PointerEvent| {
                let current = drag_state.get_untracked();
                if let Some(state) = current {
                    if ev.pointer_id() == state.pointer_id {
                        drag_state.set(None);
                    }
                }
            });

        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback(
                "pointermove",
                pointer_move_handler.as_ref().unchecked_ref(),
            );
            let _ = window.add_event_listener_with_callback(
                "pointerup",
                pointer_up_handler.as_ref().unchecked_ref(),
            );
            let _ = window.add_event_listener_with_callback(
                "pointercancel",
                pointer_cancel_handler.as_ref().unchecked_ref(),
            );
        }

        // Leak closures so they stay alive (same pattern as session_timer.rs)
        pointer_move_handler.forget();
        pointer_up_handler.forget();
        pointer_cancel_handler.forget();
    }

    // Clean up drag state when component unmounts
    on_cleanup(move || {
        drag_state.set(None);
    });

    // --- Pointer down (called from DragHandle via Callback) ---
    let on_pointer_down = Callback::new(
        move |(entry_id, source_index, ev): (String, usize, PointerEvent)| {
            // Pointer capture is set by DragHandle on its button element (currentTarget).
            // Measure the source row's height so siblings can shift by one slot.
            let source_height = measure_source_row_height(&container_ref, source_index);

            drag_state.set(Some(DragState {
                dragged_entry_id: entry_id,
                pointer_id: ev.pointer_id(),
                start_y: ev.client_y() as f64,
                current_y: ev.client_y() as f64,
                source_index,
                hover_index: source_index,
                source_height,
                committed: false,
            }));
        },
    );

    DragReorderReturn {
        is_dragging,
        dragged_id,
        source_index,
        hover_index,
        live_offset_y,
        source_height,
        on_pointer_down,
    }
}

/// Look up the offsetHeight of the row at `source_index` inside the container,
/// keyed by `data-entry-index`. Falls back to 56px (a reasonable compact-row
/// height) if the lookup fails — preserves drag UX even if the DOM is in
/// flux at pointer-down.
fn measure_source_row_height(
    container_ref: &NodeRef<leptos::html::Div>,
    source_index: usize,
) -> f64 {
    const FALLBACK_HEIGHT_PX: f64 = 56.0;
    let Some(container) = container_ref.get_untracked() else {
        return FALLBACK_HEIGHT_PX;
    };
    let element: &Element = &container;
    let children = element.children();
    for i in 0..children.length() {
        if let Some(child) = children.item(i) {
            if let Some(idx_str) = child.get_attribute("data-entry-index") {
                if idx_str
                    .parse::<usize>()
                    .map(|idx| idx == source_index)
                    .unwrap_or(false)
                {
                    if let Ok(html) = child.dyn_into::<HtmlElement>() {
                        return html.offset_height() as f64;
                    }
                }
            }
        }
    }
    FALLBACK_HEIGHT_PX
}

/// Compute the hover index based on the pointer's Y position and the container's children.
///
/// We look at each child row's bounding rect midpoint. The hover index is the position
/// where the dragged item would be inserted (i.e., the first row whose midpoint is below
/// the pointer's Y).
///
/// The `source_index` parameter is used to handle the off-by-one issue when dragging
/// downward: because the dragged item will be *removed* from its source position before
/// being inserted, all indices after source shift up by one. We adjust so the returned
/// hover_index is the final insertion position in the *post-removal* list.
fn compute_hover_index(
    container_ref: &NodeRef<leptos::html::Div>,
    pointer_y: f64,
    count: usize,
    source_index: usize,
) -> usize {
    if count == 0 {
        return 0;
    }

    let Some(container) = container_ref.get() else {
        return 0;
    };

    let element: &Element = &container;
    let children = element.children();

    if children.length() == 0 {
        return 0;
    }

    // Walk through children and collect midpoints of entry rows
    // (identified by `data-entry-index` attribute).
    let mut midpoints: Vec<(usize, f64)> = Vec::new();

    for i in 0..children.length() {
        if let Some(child) = children.item(i) {
            if let Some(idx_str) = child.get_attribute("data-entry-index") {
                if let Ok(idx) = idx_str.parse::<usize>() {
                    let rect = child.get_bounding_client_rect();
                    let mid = rect.top() + rect.height() / 2.0;
                    midpoints.push((idx, mid));
                }
            }
        }
    }

    if midpoints.is_empty() {
        return 0;
    }

    // Find the visual insertion point: the position before the first item
    // whose midpoint is below the pointer.
    let mut visual_index = midpoints.len(); // default: after all items
    for (i, (_idx, mid)) in midpoints.iter().enumerate() {
        if pointer_y < *mid {
            visual_index = i;
            break;
        }
    }

    // Adjust for the "gap" left by the dragged item.
    // When the source item is removed, items after it shift up by one.
    // If the visual target is *after* the source, we need to subtract one
    // to get the correct insertion index in the post-removal list.
    // If visual_index == source_index or source_index + 1, it's a no-op
    // (dropping the item back roughly where it was).
    if visual_index > source_index {
        visual_index.saturating_sub(1)
    } else {
        visual_index
    }
}
