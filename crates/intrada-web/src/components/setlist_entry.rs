use leptos::prelude::*;
use web_sys::PointerEvent;

use intrada_core::ItemKind;

use crate::components::{DragHandle, Icon, IconName, TypeBadge};

/// A single entry in the setlist (building or active phase).
///
/// Takes the minimum data needed to render — id + title + type, plus an
/// optional duration string and position number for surfaces that need
/// them. Decoupled from any specific entry view-model so this row primitive
/// can be reused by routine entries (which have a smaller shape than
/// session entries) without padding callsites with default fields.
#[component]
pub fn SetlistEntryRow(
    /// Stable id for the entry — used as the payload for remove / move /
    /// drag callbacks.
    #[prop(into)]
    id: String,
    /// Title shown in the row.
    #[prop(into)]
    item_title: String,
    /// Type — drives the trailing TypeBadge colouring.
    item_type: ItemKind,
    /// Optional duration string ("5 min", "10 min · 3 reps"). Hidden when
    /// empty or "0s".
    #[prop(default = String::new(), into)]
    duration_display: String,
    /// Position in the list — only rendered when `compact = false`. When
    /// shown, displays as `position + 1` left of the title.
    #[prop(default = 0)]
    position: usize,
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
    /// Compact row style matching the Pencil session-builder review sheet:
    /// flat row with bottom border, no card background, no position number,
    /// title + meta stacked. Used in the review sheet; active session keeps
    /// the default card-backed row.
    #[prop(default = false)]
    compact: bool,
) -> impl IntoView {
    let show = show_controls;
    let id_for_remove = id.clone();
    let id_for_up = id.clone();
    let id_for_down = id.clone();
    let id_for_drag = id;

    view! {
        <div
            class=move || {
                let dragging = is_dragging_this.get();
                match (compact, dragging) {
                    // Compact + dragging: keep full opacity (the row physically
                    // tracks the finger via the parent wrapper's transform —
                    // dimming would just look broken). Subtle bg lifts it off
                    // the list under it.
                    (true, true) => "flex items-center gap-2 py-1 border-b border-border-default bg-surface-secondary rounded-md",
                    (true, false) => "flex items-center gap-2 py-1 border-b border-border-default",
                    (false, true) => "flex items-center gap-3 rounded-lg bg-surface-secondary px-4 py-3 drag-active ring-2 ring-accent-focus",
                    (false, false) => "flex items-center gap-3 rounded-lg bg-surface-secondary px-4 py-3",
                }
            }
            data-entry-index=index.to_string()
        >
            // Drag handle (leftmost, before position number)
            {on_drag_pointer_down.map(|cb| {
                view! {
                    <DragHandle
                        entry_id=id_for_drag.clone()
                        index=index
                        on_pointer_down=cb
                    />
                }
            })}

            {(!compact).then(|| view! {
                <span class="text-sm font-mono text-faint w-6 text-right">
                    {position + 1}
                </span>
            })}
            <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                    <span class="text-sm font-medium text-primary truncate">{item_title}</span>
                    {if !compact {
                        Some(view! { <TypeBadge item_type=item_type.clone() /> })
                    } else {
                        None
                    }}
                </div>
                {if !duration_display.is_empty() && duration_display != "0s" {
                    Some(view! {
                        <span class="text-xs text-muted">{duration_display}</span>
                    })
                } else {
                    None
                }}
            </div>
            {if compact {
                Some(view! { <TypeBadge item_type=item_type /> })
            } else {
                None
            }}
            {if show {
                Some(view! {
                    <div class="flex gap-1">
                        {on_move_up.map(|cb| {
                            let id = id_for_up.clone();
                            view! {
                                <button
                                    class="p-1 text-faint hover:text-secondary"
                                    title="Move up"
                                    on:click=move |_| cb.run(id.clone())
                                >
                                    <Icon name=IconName::ChevronUp class="w-4 h-4" />
                                </button>
                            }
                        })}
                        {on_move_down.map(|cb| {
                            let id = id_for_down.clone();
                            view! {
                                <button
                                    class="p-1 text-faint hover:text-secondary"
                                    title="Move down"
                                    on:click=move |_| cb.run(id.clone())
                                >
                                    <Icon name=IconName::ChevronDown class="w-4 h-4" />
                                </button>
                            }
                        })}
                        {on_remove.map(|cb| {
                            let id = id_for_remove.clone();
                            view! {
                                <button
                                    class="p-1 text-danger-text hover:text-danger-hover"
                                    title="Remove"
                                    on:click=move |_| cb.run(id.clone())
                                >
                                    <Icon name=IconName::X class="w-4 h-4" />
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
