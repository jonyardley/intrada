use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::PointerEvent;

/// A six-dot grip icon used as the drag handle for reorderable list entries.
///
/// Applies `touch-action: none` and `user-select: none` to prevent scroll
/// interference on mobile devices. Minimum 44x44px touch target for
/// accessibility compliance (WCAG 2.5.5).
#[component]
pub fn DragHandle(
    /// The id of the entry this handle belongs to.
    entry_id: String,
    /// The current index (position) of this entry in the list.
    index: usize,
    /// Callback from `use_drag_reorder` to initiate drag on pointerdown.
    on_pointer_down: Callback<(String, usize, PointerEvent)>,
) -> impl IntoView {
    let entry_id_down = entry_id;

    view! {
        <button
            type="button"
            role="button"
            aria-label="Drag to reorder"
            class="flex items-center justify-center w-11 h-11 min-w-[44px] min-h-[44px] cursor-grab text-faint hover:text-secondary select-none"
            style="touch-action: none; user-select: none; -webkit-user-select: none;"
            on:pointerdown=move |ev: PointerEvent| {
                ev.prevent_default();
                // Set pointer capture on the button (currentTarget), not the SVG
                // child that may be the actual ev.target(). This ensures all
                // subsequent pointer events fire on this element reliably.
                if let Some(ct) = ev.current_target() {
                    if let Ok(el) = ct.dyn_into::<web_sys::HtmlElement>() {
                        let _ = el.set_pointer_capture(ev.pointer_id());
                    }
                }
                on_pointer_down.run((entry_id_down.clone(), index, ev));
            }
            on:contextmenu=move |ev: leptos::ev::MouseEvent| {
                // Suppress long-press context menu on mobile (T019 / FR-007)
                ev.prevent_default();
            }
        >
            // Six-dot grip SVG icon — pointer-events:none so button always
            // receives the pointerdown, not the SVG or its circle children.
            <svg
                width="16"
                height="16"
                viewBox="0 0 16 16"
                fill="currentColor"
                aria-hidden="true"
                style="pointer-events: none;"
            >
                <circle cx="5" cy="3" r="1.5" />
                <circle cx="11" cy="3" r="1.5" />
                <circle cx="5" cy="8" r="1.5" />
                <circle cx="11" cy="8" r="1.5" />
                <circle cx="5" cy="13" r="1.5" />
                <circle cx="11" cy="13" r="1.5" />
            </svg>
        </button>
    }
}
