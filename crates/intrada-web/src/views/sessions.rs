use leptos::prelude::*;

use intrada_core::{Event, PracticeSessionView, SessionEvent, ViewModel};

use crate::components::{Button, ButtonVariant, Card, PageHeading};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::SharedCore;

/// All-sessions list view showing every completed practice session.
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
                            <p class="text-slate-500">"No practice sessions recorded yet."</p>
                            <p class="text-sm text-slate-400 mt-2">"Start a practice session to begin tracking your progress."</p>
                        </div>
                    }.into_any()
                } else {
                    let core = core.clone();
                    view! {
                        <div class="space-y-3">
                            {vm.sessions.iter().map(|session| {
                                view! {
                                    <SessionRow
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

/// A completed session row with summary info and delete action.
#[component]
fn SessionRow(
    session: PracticeSessionView,
    core: SharedCore,
    view_model: RwSignal<ViewModel>,
) -> impl IntoView {
    let confirm_delete = RwSignal::new(false);

    let id_for_delete = session.id.clone();
    let started_at = session.started_at.clone();
    let total_duration = session.total_duration_display.clone();
    let completion_status = session.completion_status.clone();
    let session_notes = session.notes.clone();
    let entry_count = session.entries.len();

    view! {
        <Card>
            {move || {
                if confirm_delete.get() {
                    let core_del = core.clone();
                    let id_del = id_for_delete.clone();
                    view! {
                        <div>
                            <p class="text-sm text-red-800 mb-3">"Delete this session? This cannot be undone."</p>
                            <div class="flex gap-2">
                                <Button variant=ButtonVariant::Danger on_click=Callback::new(move |_| {
                                    let event = Event::Session(SessionEvent::DeleteSession { id: id_del.clone() });
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
                    let started_at = started_at.clone();
                    let total_duration = total_duration.clone();
                    let completion_status = completion_status.clone();
                    let session_notes = session_notes.clone();
                    view! {
                        <div class="flex items-center justify-between">
                            <div class="flex-1">
                                <div class="flex items-baseline gap-3">
                                    <span class="text-sm font-medium text-slate-900">
                                        {total_duration}
                                    </span>
                                    <span class="text-xs text-slate-500">
                                        {format!("{} item{}", entry_count, if entry_count == 1 { "" } else { "s" })}
                                    </span>
                                    {if completion_status == "ended_early" {
                                        Some(view! {
                                            <span class="inline-flex items-center rounded-md bg-amber-50 px-2 py-0.5 text-xs font-medium text-amber-700 ring-1 ring-amber-600/20 ring-inset">
                                                "Ended Early"
                                            </span>
                                        })
                                    } else {
                                        None
                                    }}
                                    <span class="text-xs text-slate-400">{started_at}</span>
                                </div>
                                {session_notes.map(|n| {
                                    view! {
                                        <p class="text-sm text-slate-600 mt-1">{n}</p>
                                    }
                                })}
                            </div>
                            <div class="flex gap-2 ml-4">
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
