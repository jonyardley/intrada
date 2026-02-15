use leptos::prelude::*;
use leptos_router::components::A;

use intrada_core::{Event, SessionEvent, SessionView, UpdateSession, ViewModel};

use crate::components::{Button, ButtonVariant, Card, PageHeading};
use crate::core_bridge::process_effects;
use crate::types::SharedCore;

/// All-sessions list view showing every practice session across all library items.
#[component]
pub fn SessionsListView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();

    view! {
        <div>
            <PageHeading text="Practice Sessions" />

            {move || {
                let vm = view_model.get();

                if vm.sessions.is_empty() {
                    view! {
                        <div class="text-center py-12">
                            <p class="text-slate-500">"No practice sessions logged yet."</p>
                            <p class="text-sm text-slate-400 mt-2">"Go to a library item and log your first session."</p>
                        </div>
                    }.into_any()
                } else {
                    let core = core.clone();
                    view! {
                        <div class="space-y-3">
                            {vm.sessions.iter().map(|session| {
                                view! {
                                    <AllSessionRow
                                        session=session.clone()
                                        core=core.clone()
                                        view_model=view_model
                                    />
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                        <p class="text-sm text-slate-400 mt-4">
                            {format!("{} session{}", vm.sessions.len(), if vm.sessions.len() == 1 { "" } else { "s" })}
                        </p>
                    }.into_any()
                }
            }}
        </div>
    }
}

/// A session row in the all-sessions list with item info, edit, and delete.
#[component]
fn AllSessionRow(
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
    let item_href = format!("/library/{}", session.item_id);
    let item_title = session.item_title.clone();
    let item_type = session.item_type.clone();
    let logged_at = session.logged_at.clone();
    let session_notes = session.notes.clone();

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
                    let item_href = item_href.clone();
                    let item_title = item_title.clone();
                    let item_type = item_type.clone();
                    let logged_at = logged_at.clone();
                    let session_notes = session_notes.clone();
                    view! {
                        <div class="flex items-center justify-between">
                            <div class="flex-1">
                                <div class="flex items-baseline gap-3">
                                    <span class="text-sm font-medium text-slate-900">
                                        {format!("{} min", duration)}
                                    </span>
                                    <A href=item_href attr:class="text-sm text-indigo-600 hover:text-indigo-800">
                                        {item_title}
                                    </A>
                                    <span class="text-xs text-slate-400">{item_type}</span>
                                    <span class="text-xs text-slate-400">{logged_at}</span>
                                </div>
                                {session_notes.map(|n| {
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
