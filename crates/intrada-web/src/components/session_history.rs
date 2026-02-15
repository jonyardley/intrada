use leptos::prelude::*;

use intrada_core::{Event, SessionEvent, SessionView, UpdateSession, ViewModel};

use crate::components::{Button, ButtonVariant, Card};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::SharedCore;

/// Session history list for a specific library item.
/// Shows sessions in reverse chronological order with edit/delete actions.
#[component]
pub fn SessionHistory(item_id: String) -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();

    let item_id_for_list = item_id.clone();

    view! {
        <div class="mt-8">
            <h3 class="text-lg font-semibold text-slate-900 mb-4">"Practice Sessions"</h3>
            {move || {
                let vm = view_model.get();
                let sessions: Vec<&SessionView> = vm.sessions.iter()
                    .filter(|s| s.item_id == item_id_for_list)
                    .collect();

                if sessions.is_empty() {
                    view! {
                        <p class="text-sm text-slate-500">"No practice sessions logged yet."</p>
                    }.into_any()
                } else {
                    let core = core.clone();
                    view! {
                        <div class="space-y-3">
                            {sessions.into_iter().map(|session| {
                                view! {
                                    <SessionRow
                                        session=session.clone()
                                        core=core.clone()
                                        view_model=view_model
                                    />
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

/// A single session row with inline edit and delete functionality.
#[component]
fn SessionRow(
    session: SessionView,
    core: SharedCore,
    view_model: RwSignal<ViewModel>,
) -> impl IntoView {
    let editing = RwSignal::new(false);
    let confirm_delete = RwSignal::new(false);
    let edit_duration = RwSignal::new(session.duration_minutes.to_string());
    let edit_notes = RwSignal::new(session.notes.clone().unwrap_or_default());

    let id_for_edit = session.id.clone();
    let id_for_delete = session.id.clone();
    let duration = session.duration_minutes;

    view! {
        <Card>
            {move || {
                if editing.get() {
                    let core_edit = core.clone();
                    let id_save = id_for_edit.clone();
                    view! {
                        <div class="space-y-3">
                            <div>
                                <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-duration">"Duration (minutes)"</label>
                                <input
                                    id="edit-duration"
                                    type="number"
                                    min="1"
                                    max="1440"
                                    class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                                    bind:value=edit_duration
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-notes">"Notes"</label>
                                <textarea
                                    id="edit-notes"
                                    rows="2"
                                    class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                                    bind:value=edit_notes
                                />
                            </div>
                            <div class="flex gap-2">
                                <Button variant=ButtonVariant::Primary on_click=Callback::new(move |_| {
                                    let dur: Option<u32> = edit_duration.get().parse().ok();
                                    let notes_val = edit_notes.get();
                                    let notes_update = if notes_val.is_empty() {
                                        Some(None)
                                    } else {
                                        Some(Some(notes_val))
                                    };
                                    let event = Event::Session(SessionEvent::Update {
                                        id: id_save.clone(),
                                        input: UpdateSession {
                                            duration_minutes: dur,
                                            notes: notes_update,
                                        },
                                    });
                                    let core_ref = core_edit.borrow();
                                    let effects = core_ref.process_event(event);
                                    process_effects(&core_ref, effects, &view_model);
                                    editing.set(false);
                                })>
                                    "Save"
                                </Button>
                                <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| {
                                    editing.set(false);
                                })>
                                    "Cancel"
                                </Button>
                            </div>
                        </div>
                    }.into_any()
                } else if confirm_delete.get() {
                    let core_del = core.clone();
                    let id_del = id_for_delete.clone();
                    view! {
                        <div>
                            <p class="text-sm text-red-800 mb-3">"Delete this session? This cannot be undone."</p>
                            <div class="flex gap-2">
                                <Button variant=ButtonVariant::Danger on_click=Callback::new(move |_| {
                                    let event = Event::Session(SessionEvent::Delete { id: id_del.clone() });
                                    let core_ref = core_del.borrow();
                                    let effects = core_ref.process_event(event);
                                    process_effects(&core_ref, effects, &view_model);
                                })>
                                    "Confirm Delete"
                                </Button>
                                <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| {
                                    confirm_delete.set(false);
                                })>
                                    "Cancel"
                                </Button>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="flex items-center justify-between">
                            <div class="flex-1">
                                <div class="flex items-baseline gap-3">
                                    <span class="text-sm font-medium text-slate-900">
                                        {format!("{} min", duration)}
                                    </span>
                                    <span class="text-xs text-slate-400">{session.logged_at.clone()}</span>
                                </div>
                                {session.notes.clone().map(|n| {
                                    view! {
                                        <p class="text-sm text-slate-600 mt-1">{n}</p>
                                    }
                                })}
                            </div>
                            <div class="flex gap-2 ml-4">
                                <button
                                    class="text-xs text-indigo-600 hover:text-indigo-800 font-medium"
                                    on:click=move |_| { editing.set(true); }
                                >
                                    "Edit"
                                </button>
                                <button
                                    class="text-xs text-red-600 hover:text-red-800 font-medium"
                                    on:click=move |_| { confirm_delete.set(true); }
                                >
                                    "Delete"
                                </button>
                            </div>
                        </div>
                    }.into_any()
                }
            }}
        </Card>
    }
}
