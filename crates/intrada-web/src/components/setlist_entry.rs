use leptos::prelude::*;

use intrada_core::SetlistEntryView;

use crate::components::TypeBadge;

/// A single entry in the setlist (building or active phase).
#[component]
pub fn SetlistEntryRow(
    entry: SetlistEntryView,
    #[prop(default = None)] on_remove: Option<Callback<String>>,
    #[prop(default = None)] on_move_up: Option<Callback<String>>,
    #[prop(default = None)] on_move_down: Option<Callback<String>>,
    #[prop(default = true)] show_controls: bool,
) -> impl IntoView {
    let show = show_controls;
    let entry_id = entry.id.clone();
    let entry_id_up = entry.id.clone();
    let entry_id_down = entry.id.clone();

    view! {
        <div class="flex items-center gap-3 rounded-lg border border-slate-200 bg-white px-4 py-3">
            <span class="text-sm font-mono text-slate-400 w-6 text-right">
                {entry.position + 1}
            </span>
            <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                    <span class="text-sm font-medium text-slate-900 truncate">{entry.item_title}</span>
                    <TypeBadge item_type=entry.item_type />
                </div>
                {if !entry.duration_display.is_empty() && entry.duration_display != "0s" {
                    Some(view! {
                        <span class="text-xs text-slate-500">{entry.duration_display}</span>
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
                                    class="p-1 text-slate-400 hover:text-slate-600"
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
                                    class="p-1 text-slate-400 hover:text-slate-600"
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
                                    class="p-1 text-red-400 hover:text-red-600"
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
