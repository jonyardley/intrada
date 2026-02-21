use leptos::prelude::*;
use web_sys::PointerEvent;

use intrada_core::SetlistEntryView;

use crate::components::{DragHandle, TypeBadge};

/// A single entry in the setlist (building or active phase).
#[component]
pub fn SetlistEntryRow(
    entry: SetlistEntryView,
    #[prop(default = None)] on_remove: Option<Callback<String>>,
    #[prop(default = None)] on_move_up: Option<Callback<String>>,
    #[prop(default = None)] on_move_down: Option<Callback<String>>,
    #[prop(default = true)] show_controls: bool,
    /// Whether this entry is currently being dragged (applies visual highlight).
    #[prop(default = Signal::derive(|| false))]
    is_dragging_this: Signal<bool>,
    /// Callback from `use_drag_reorder` hook for drag handle pointer down.
    /// If `None`, drag handle is not shown.
    #[prop(default = None)]
    on_drag_pointer_down: Option<Callback<(String, usize, PointerEvent)>>,
    /// The index of this entry in the list (used by drag handle).
    #[prop(default = 0)]
    index: usize,
) -> impl IntoView {
    let show = show_controls;
    let entry_id = entry.id.clone();
    let entry_id_up = entry.id.clone();
    let entry_id_down = entry.id.clone();
    let entry_id_drag = entry.id.clone();

    view! {
        <div
            class=move || {
                if is_dragging_this.get() {
                    "flex items-center gap-3 rounded-lg bg-surface-secondary px-4 py-3 drag-active ring-2 ring-accent-focus"
                } else {
                    "flex items-center gap-3 rounded-lg bg-surface-secondary px-4 py-3"
                }
            }
            data-entry-index=index.to_string()
        >
            // Drag handle (leftmost, before position number)
            {on_drag_pointer_down.map(|cb| {
                view! {
                    <DragHandle
                        entry_id=entry_id_drag.clone()
                        index=index
                        on_pointer_down=cb
                    />
                }
            })}

            <span class="text-sm font-mono text-faint w-6 text-right">
                {entry.position + 1}
            </span>
            <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                    <span class="text-sm font-medium text-white truncate">{entry.item_title}</span>
                    <TypeBadge item_type=entry.item_type />
                </div>
                {if !entry.duration_display.is_empty() && entry.duration_display != "0s" {
                    Some(view! {
                        <span class="text-xs text-muted">{entry.duration_display}</span>
                    })
                } else {
                    None
                }}
            </div>
            {if show {
                Some(view! {
                    <div class="flex gap-1">
                        {on_move_up.map(|cb| {
                            let id = entry_id_up.clone();
                            view! {
                                <button
                                    class="p-1 text-faint hover:text-secondary"
                                    title="Move up"
                                    on:click=move |_| cb.run(id.clone())
                                >
                                    "↑"
                                </button>
                            }
                        })}
                        {on_move_down.map(|cb| {
                            let id = entry_id_down.clone();
                            view! {
                                <button
                                    class="p-1 text-faint hover:text-secondary"
                                    title="Move down"
                                    on:click=move |_| cb.run(id.clone())
                                >
                                    "↓"
                                </button>
                            }
                        })}
                        {on_remove.map(|cb| {
                            let id = entry_id.clone();
                            view! {
                                <button
                                    class="p-1 text-danger-text hover:text-danger-hover"
                                    title="Remove"
                                    on:click=move |_| cb.run(id.clone())
                                >
                                    "✕"
                                </button>
                            }
                        })}
                    </div>
                })
            } else {
                None
            }}
        </div>
    }
}
