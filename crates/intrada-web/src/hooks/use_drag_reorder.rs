use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, PointerEvent};

use crate::haptics::{haptic_light, haptic_selection, haptic_success};

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
    /// Snapshot of `(entry_index, midpoint_y)` for every row at drag-start,
    /// captured BEFORE any transforms are applied. The hover-index calculation
    /// uses these static positions throughout the drag — reading live
    /// `getBoundingClientRect` values would drift, since the source row's
    /// transformed rect moves with the finger.
    pub natural_midpoints: Vec<(usize, f64)>,
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

impl DragReorderReturn {
    /// Build the inline `style` value for a row at position `idx` based on
    /// the current drag state. Encapsulates the "source row tracks the
    /// finger; rows between source and hover translate by source_height"
    /// logic so consumers don't reimplement it. Used as
    /// `style=drag.row_style_for(idx)`.
    ///
    /// Returns an empty string when no drag is in progress (which clears
    /// any previous transform on release).
    pub fn row_style_for(&self, idx: usize) -> impl Fn() -> String + Copy {
        let source_index = self.source_index;
        let hover_index = self.hover_index;
        let live_offset_y = self.live_offset_y;
        let source_height = self.source_height;
        move || {
            let Some(src) = source_index.get() else {
                return String::new();
            };
            if idx == src {
                let off = live_offset_y.get();
                return format!(
                    "transform: translateY({off}px) scale(1.02); transition: none; position: relative; z-index: 10; box-shadow: 0 8px 20px rgba(0,0,0,0.35);"
                );
            }
            let Some(hov) = hover_index.get() else {
                return String::new();
            };
            let h = source_height.get();
            let displaced_down = hov > src && idx > src && idx <= hov;
            let displaced_up = hov < src && idx >= hov && idx < src;
            if displaced_down {
                format!(
                    "transform: translateY(-{h}px); transition: transform 200ms cubic-bezier(0.4, 0, 0.2, 1);"
                )
            } else if displaced_up {
                format!(
                    "transform: translateY({h}px); transition: transform 200ms cubic-bezier(0.4, 0, 0.2, 1);"
                )
            } else {
                "transform: translateY(0); transition: transform 200ms cubic-bezier(0.4, 0, 0.2, 1);".to_string()
            }
        }
    }
}

/// Movement threshold in pixels before drag is committed.
const DRAG_THRESHOLD_PX: f64 = 5.0;

/// Creates a reusable drag-and-drop reorder hook.
///
/// # Arguments
///
/// * `on_reorder` – callback invoked with `(entry_id, new_position)` when a valid drop occurs.
/// * `container_ref` – `NodeRef` of the container element whose children are the draggable rows.
pub fn use_drag_reorder(
    on_reorder: Callback<(String, usize)>,
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
                // Capture the haptic-trigger conditions inside the update
                // closure (so we can read pre/post state), then fire haptics
                // AFTER drag_state.update() returns — calling into JS while
                // holding a borrow on the RwSignal can deadlock under Leptos's
                // signal scheduler.
                let mut commit_haptic = false;
                let mut selection_haptic = false;

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
                        commit_haptic = true;
                    }

                    // Compute hover index against the snapshot of natural row
                    // midpoints captured at pointer-down. Using a static
                    // snapshot avoids drift: reading live
                    // `getBoundingClientRect` would include the source row's
                    // own transform, making its midpoint chase the finger.
                    let new_hover = compute_hover_index_from_snapshot(
                        &s.natural_midpoints,
                        s.current_y,
                        s.source_index,
                    );
                    if new_hover != s.hover_index {
                        // The dragged row has crossed a sibling's slot —
                        // matches iOS UITableView's "tick" feedback when
                        // entries reorder under the finger.
                        selection_haptic = true;
                        s.hover_index = new_hover;
                    }
                });

                if commit_haptic {
                    // Light "lift" haptic the first time the gesture passes the
                    // 5px threshold and visually commits — same beat as iOS's
                    // begin-dragging feedback.
                    haptic_light();
                } else if selection_haptic {
                    haptic_selection();
                }
            });

        let pointer_up_handler: Closure<dyn Fn(PointerEvent)> =
            Closure::new(move |ev: PointerEvent| {
                let current = drag_state.get_untracked();
                let Some(state) = current else { return };

                if ev.pointer_id() != state.pointer_id {
                    return;
                }

                let did_reorder = state.committed && state.hover_index != state.source_index;
                if did_reorder {
                    on_reorder.run((state.dragged_entry_id.clone(), state.hover_index));
                }

                drag_state.set(None);

                // Fire haptic AFTER state cleared + reorder dispatched so the
                // sensation lands in sync with the row settling into its new
                // slot. Light snap-back if released without a reorder, success
                // tick if the order actually changed.
                if did_reorder {
                    haptic_success();
                } else if state.committed {
                    haptic_light();
                }
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
            // Pointer capture is set by DragHandle on its button element
            // (currentTarget). Snapshot the natural row geometry now —
            // before any drag transforms are applied — so the move-handler
            // can compute hover indices against static positions.
            let snapshot = snapshot_row_geometry(&container_ref, source_index);

            drag_state.set(Some(DragState {
                dragged_entry_id: entry_id,
                pointer_id: ev.pointer_id(),
                start_y: ev.client_y() as f64,
                current_y: ev.client_y() as f64,
                source_index,
                hover_index: source_index,
                source_height: snapshot.source_height,
                natural_midpoints: snapshot.midpoints,
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

/// Snapshot of row geometry captured at pointer-down: natural midpoints for
/// hover-index calculation plus the source row's height for sibling reflow.
struct RowGeometrySnapshot {
    /// `(entry_index, midpoint_y)` for each row keyed by `data-entry-index`.
    midpoints: Vec<(usize, f64)>,
    /// `offsetHeight` of the source row, with a sensible fallback.
    source_height: f64,
}

/// Walk the container's children once and collect each row's natural midpoint
/// plus the source row's height. Called at pointer-down before any drag
/// transforms are applied, so the rects are unaffected by translateY.
///
/// Falls back to a 56px source height (compact-row default) if the source row
/// can't be measured — preserves drag UX even if the DOM is in flux.
fn snapshot_row_geometry(
    container_ref: &NodeRef<leptos::html::Div>,
    source_index: usize,
) -> RowGeometrySnapshot {
    const FALLBACK_HEIGHT_PX: f64 = 56.0;

    let Some(container) = container_ref.get_untracked() else {
        return RowGeometrySnapshot {
            midpoints: Vec::new(),
            source_height: FALLBACK_HEIGHT_PX,
        };
    };
    let children = container.children();

    let mut midpoints = Vec::with_capacity(children.length() as usize);
    let mut source_height = FALLBACK_HEIGHT_PX;

    for i in 0..children.length() {
        let Some(child) = children.item(i) else {
            continue;
        };
        let Some(idx_str) = child.get_attribute("data-entry-index") else {
            continue;
        };
        let Ok(idx) = idx_str.parse::<usize>() else {
            continue;
        };
        let rect = child.get_bounding_client_rect();
        let mid = rect.top() + rect.height() / 2.0;
        midpoints.push((idx, mid));
        if idx == source_index {
            if let Ok(html) = child.dyn_into::<HtmlElement>() {
                source_height = html.offset_height() as f64;
            }
        }
    }

    RowGeometrySnapshot {
        midpoints,
        source_height,
    }
}

/// Compute the hover (insertion) index from a static midpoints snapshot.
///
/// Walks the snapshot in DOM order; the insertion point is the position before
/// the first row whose midpoint sits below the pointer. The result is then
/// adjusted for the "gap" left by the source row when it's removed: if the
/// visual target is past the source, decrement so the returned index is the
/// position in the post-removal list.
fn compute_hover_index_from_snapshot(
    snapshot: &[(usize, f64)],
    pointer_y: f64,
    source_index: usize,
) -> usize {
    if snapshot.is_empty() {
        return 0;
    }

    let mut visual_index = snapshot.len();
    for (i, (_idx, mid)) in snapshot.iter().enumerate() {
        if pointer_y < *mid {
            visual_index = i;
            break;
        }
    }

    if visual_index > source_index {
        visual_index.saturating_sub(1)
    } else {
        visual_index
    }
}
