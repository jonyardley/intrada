use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use leptos_router::hooks::use_params_map;
use leptos_router::NavigateOptions;

use intrada_core::domain::exercise::ExerciseEvent;
use intrada_core::domain::piece::PieceEvent;
use intrada_core::{Event, ViewModel};

use crate::components::{BackLink, Button, ButtonVariant, Card, FieldLabel, TypeBadge};
use crate::core_bridge::process_effects;
use crate::types::SharedCore;

#[component]
pub fn DetailView(view_model: RwSignal<ViewModel>, core: SharedCore) -> impl IntoView {
    let params = use_params_map();
    let id = params.read().get("id").unwrap_or_default();
    let navigate = use_navigate();

    let show_delete_confirm = RwSignal::new(false);

    // Find the item in the current ViewModel
    let item = view_model
        .get_untracked()
        .items
        .into_iter()
        .find(|i| i.id == id);

    let Some(item) = item else {
        // Item not found — show message with link back to list
        return view! {
            <div class="text-center py-8">
                <p class="text-slate-600 mb-4">"Item not found."</p>
                <A href="/" attr:class="text-indigo-600 hover:text-indigo-800 font-medium">
                    "\u{2190} Back to Library"
                </A>
            </div>
        }
        .into_any();
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

    // Build edit href based on item type
    let edit_href = if type_for_edit == "piece" {
        format!("/pieces/{}/edit", id_for_edit)
    } else {
        format!("/exercises/{}/edit", id_for_edit)
    };

    view! {
        <div>
            // Back link
            <BackLink label="Back to Library" href="/".to_string() />

            // Delete confirmation banner (FR-011)
            {move || {
                if show_delete_confirm.get() {
                    let id_del = id_for_delete.clone();
                    let core_del = core.clone();
                    let item_type_del = type_for_delete.clone();
                    let navigate_del = navigate.clone();
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
                                        navigate_del("/", NavigateOptions { replace: true, ..Default::default() });
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
                <A href=edit_href attr:class="inline-flex items-center justify-center rounded-lg bg-indigo-600 px-4 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 transition-colors">
                    "Edit"
                </A>
                <Button variant=ButtonVariant::DangerOutline on_click=Callback::new(move |_| { show_delete_confirm.set(true); })>
                    "Delete"
                </Button>
            </div>
        </div>
    }.into_any()
}
