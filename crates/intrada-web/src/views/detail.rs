use leptos::prelude::*;

use intrada_core::domain::exercise::ExerciseEvent;
use intrada_core::domain::piece::PieceEvent;
use intrada_core::{Event, ViewModel};

use crate::core_bridge::process_effects;
use crate::types::{SharedCore, ViewState};

#[component]
pub fn DetailView(
    id: String,
    view_model: RwSignal<ViewModel>,
    view_state: RwSignal<ViewState>,
    core: SharedCore,
) -> impl IntoView {
    let show_delete_confirm = RwSignal::new(false);

    // Find the item in the current ViewModel
    let item = view_model
        .get_untracked()
        .items
        .into_iter()
        .find(|i| i.id == id);

    let Some(item) = item else {
        // Item not found — navigate back to list (handles deleted-item edge case)
        view_state.set(ViewState::List);
        return view! { <p>"Item not found."</p> }.into_any();
    };

    let item_id = item.id.clone();
    let item_type = item.item_type.clone();
    let is_piece = item_type == "piece";

    // Clone fields for display
    let title = item.title.clone();
    let subtitle = item.subtitle.clone();
    let category = item.category.clone();
    let key = item.key.clone();
    let tempo = item.tempo.clone();
    let notes = item.notes.clone();
    let tags = item.tags.clone();
    let created_at = item.created_at.clone();
    let updated_at = item.updated_at.clone();

    // Clone IDs for closures
    let id_for_edit = item_id.clone();
    let id_for_delete = item_id.clone();
    let type_for_edit = item_type.clone();

    view! {
        <div>
            // Back button
            <button
                class="mb-6 inline-flex items-center gap-1 text-sm text-slate-500 hover:text-slate-700 transition-colors"
                on:click=move |_| { view_state.set(ViewState::List); }
            >
                "\u{2190} Back to Library"
            </button>

            // Delete confirmation banner (FR-011)
            {move || {
                if show_delete_confirm.get() {
                    let id_del = id_for_delete.clone();
                    let core_del = core.clone();
                    let item_type_del = item_type.clone();
                    Some(view! {
                        <div class="mb-6 rounded-lg bg-red-50 border border-red-200 p-4" role="alert">
                            <p class="text-sm text-red-800 mb-3">
                                "Are you sure you want to delete this item? This action cannot be undone."
                            </p>
                            <div class="flex gap-3">
                                <button
                                    class="rounded-lg bg-red-600 px-3.5 py-2 text-sm font-medium text-white hover:bg-red-500 transition-colors"
                                    on:click=move |_| {
                                        let event = if item_type_del == "piece" {
                                            Event::Piece(PieceEvent::Delete { id: id_del.clone() })
                                        } else {
                                            Event::Exercise(ExerciseEvent::Delete { id: id_del.clone() })
                                        };
                                        let core_ref = core_del.borrow();
                                        let effects = core_ref.process_event(event);
                                        process_effects(&core_ref, effects, &view_model);
                                        view_state.set(ViewState::List);
                                    }
                                >
                                    "Confirm Delete"
                                </button>
                                <button
                                    class="rounded-lg bg-white px-3.5 py-2 text-sm font-medium text-slate-700 border border-slate-300 hover:bg-slate-50 transition-colors"
                                    on:click=move |_| { show_delete_confirm.set(false); }
                                >
                                    "Cancel"
                                </button>
                            </div>
                        </div>
                    })
                } else {
                    None
                }
            }}

            // Detail card
            <div class="bg-white rounded-xl shadow-sm border border-slate-200 p-6">
                // Header: title + type badge
                <div class="flex items-start justify-between gap-3 mb-6">
                    <div>
                        <h2 class="text-2xl font-bold text-slate-900">{title}</h2>
                        {if !subtitle.is_empty() {
                            Some(view! {
                                <p class="text-lg text-slate-500 mt-1">{subtitle.clone()}</p>
                            })
                        } else {
                            None
                        }}
                    </div>
                    <span class={if is_piece {
                        "inline-flex items-center rounded-full bg-violet-100 px-3 py-1 text-sm font-medium text-violet-800"
                    } else {
                        "inline-flex items-center rounded-full bg-emerald-100 px-3 py-1 text-sm font-medium text-emerald-800"
                    }}>
                        {if is_piece { "Piece" } else { "Exercise" }}
                    </span>
                </div>

                // Fields grid (FR-007, FR-008: omit empty optional fields)
                <dl class="grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-4 mb-6">
                    {category.map(|cat| {
                        view! {
                            <div>
                                <dt class="text-xs font-medium text-slate-400 uppercase tracking-wider">"Category"</dt>
                                <dd class="mt-1 text-sm text-slate-700">{cat}</dd>
                            </div>
                        }
                    })}
                    {key.map(|k| {
                        view! {
                            <div>
                                <dt class="text-xs font-medium text-slate-400 uppercase tracking-wider">"Key"</dt>
                                <dd class="mt-1 text-sm text-slate-700">{k}</dd>
                            </div>
                        }
                    })}
                    {tempo.map(|t| {
                        view! {
                            <div>
                                <dt class="text-xs font-medium text-slate-400 uppercase tracking-wider">"Tempo"</dt>
                                <dd class="mt-1 text-sm text-slate-700">{t}</dd>
                            </div>
                        }
                    })}
                </dl>

                // Notes
                {notes.map(|n| {
                    view! {
                        <div class="mb-6">
                            <dt class="text-xs font-medium text-slate-400 uppercase tracking-wider mb-1">"Notes"</dt>
                            <dd class="text-sm text-slate-700 whitespace-pre-wrap">{n}</dd>
                        </div>
                    }
                })}

                // Tags
                {if !tags.is_empty() {
                    Some(view! {
                        <div class="mb-6">
                            <dt class="text-xs font-medium text-slate-400 uppercase tracking-wider mb-2">"Tags"</dt>
                            <dd class="flex flex-wrap gap-1.5">
                                {tags.into_iter().map(|tag| {
                                    view! {
                                        <span class="inline-flex items-center rounded-md bg-slate-100 px-2.5 py-1 text-xs text-slate-600">
                                            {tag}
                                        </span>
                                    }
                                }).collect::<Vec<_>>()}
                            </dd>
                        </div>
                    })
                } else {
                    None
                }}

                // Timestamps
                <div class="border-t border-slate-100 pt-4 grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs text-slate-400">
                    <div>
                        <span class="font-medium">"Created: "</span>{created_at}
                    </div>
                    <div>
                        <span class="font-medium">"Updated: "</span>{updated_at}
                    </div>
                </div>
            </div>

            // Action buttons (FR-009, FR-011)
            <div class="mt-6 flex gap-3">
                <button
                    class="rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-500 transition-colors"
                    on:click=move |_| {
                        if type_for_edit == "piece" {
                            view_state.set(ViewState::EditPiece(id_for_edit.clone()));
                        } else {
                            view_state.set(ViewState::EditExercise(id_for_edit.clone()));
                        }
                    }
                >
                    "Edit"
                </button>
                <button
                    class="rounded-lg bg-white px-4 py-2 text-sm font-medium text-red-600 border border-red-300 hover:bg-red-50 transition-colors"
                    on:click=move |_| { show_delete_confirm.set(true); }
                >
                    "Delete"
                </button>
            </div>
        </div>
    }.into_any()
}
