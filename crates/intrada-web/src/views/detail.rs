use leptos::prelude::*;

use intrada_core::domain::exercise::ExerciseEvent;
use intrada_core::domain::piece::PieceEvent;
use intrada_core::{Event, ViewModel};

use crate::components::{BackLink, Button, ButtonVariant, Card, FieldLabel, TypeBadge};
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
    let type_for_badge = item_type.clone();
    let type_for_delete = item_type;

    view! {
        <div>
            // Back button
            <BackLink label="Back to Library" on_click=Callback::new(move |_| { view_state.set(ViewState::List); }) />

            // Delete confirmation banner (FR-011)
            {move || {
                if show_delete_confirm.get() {
                    let id_del = id_for_delete.clone();
                    let core_del = core.clone();
                    let item_type_del = type_for_delete.clone();
                    Some(view! {
                        <div class="mb-6 rounded-lg bg-red-50 border border-red-200 p-4" role="alert">
                            <p class="text-sm text-red-800 mb-3">
                                "Are you sure you want to delete this item? This action cannot be undone."
                            </p>
                            <div class="flex gap-3">
                                <Button variant=ButtonVariant::Danger on_click=Callback::new(move |_| {
                                        let event = if item_type_del == "piece" {
                                            Event::Piece(PieceEvent::Delete { id: id_del.clone() })
                                        } else {
                                            Event::Exercise(ExerciseEvent::Delete { id: id_del.clone() })
                                        };
                                        let core_ref = core_del.borrow();
                                        let effects = core_ref.process_event(event);
                                        process_effects(&core_ref, effects, &view_model);
                                        view_state.set(ViewState::List);
                                    })>
                                    "Confirm Delete"
                                </Button>
                                <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| { show_delete_confirm.set(false); })>
                                    "Cancel"
                                </Button>
                            </div>
                        </div>
                    })
                } else {
                    None
                }
            }}

            // Detail card
            <Card>
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
                    <TypeBadge item_type=type_for_badge.clone() />
                </div>

                // Fields grid (FR-007, FR-008: omit empty optional fields)
                <dl class="grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-4 mb-6">
                    {category.map(|cat| {
                        view! {
                            <div>
                                <FieldLabel text="Category" />
                                <dd class="mt-1 text-sm text-slate-700">{cat}</dd>
                            </div>
                        }
                    })}
                    {key.map(|k| {
                        view! {
                            <div>
                                <FieldLabel text="Key" />
                                <dd class="mt-1 text-sm text-slate-700">{k}</dd>
                            </div>
                        }
                    })}
                    {tempo.map(|t| {
                        view! {
                            <div>
                                <FieldLabel text="Tempo" />
                                <dd class="mt-1 text-sm text-slate-700">{t}</dd>
                            </div>
                        }
                    })}
                </dl>

                // Notes
                {notes.map(|n| {
                    view! {
                        <div class="mb-6">
                            <FieldLabel text="Notes" />
                            <dd class="text-sm text-slate-700 whitespace-pre-wrap">{n}</dd>
                        </div>
                    }
                })}

                // Tags
                {if !tags.is_empty() {
                    Some(view! {
                        <div class="mb-6">
                            <FieldLabel text="Tags" />
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
            </Card>

            // Action buttons (FR-009, FR-011)
            <div class="mt-6 flex gap-3">
                <Button variant=ButtonVariant::Primary on_click=Callback::new(move |_| {
                        if type_for_edit == "piece" {
                            view_state.set(ViewState::EditPiece(id_for_edit.clone()));
                        } else {
                            view_state.set(ViewState::EditExercise(id_for_edit.clone()));
                        }
                    })>
                    "Edit"
                </Button>
                <Button variant=ButtonVariant::DangerOutline on_click=Callback::new(move |_| { show_delete_confirm.set(true); })>
                    "Delete"
                </Button>
            </div>
        </div>
    }.into_any()
}
