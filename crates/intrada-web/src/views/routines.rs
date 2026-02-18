use leptos::prelude::*;
use leptos_router::components::A;

use intrada_core::{Event, RoutineEvent, RoutineView, ViewModel};

use crate::components::{Button, ButtonVariant, Card, PageHeading};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Management page for saved routines — lists all routines with edit/delete actions.
#[component]
pub fn RoutinesListView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();

    view! {
        <div>
            <PageHeading text="Routines" />

            {move || {
                if is_loading.get() {
                    return view! {
                        <div class="flex justify-center py-12">
                            <div class="animate-spin rounded-full h-8 w-8 border-2 border-indigo-400 border-t-transparent"></div>
                        </div>
                    }.into_any();
                }

                let vm = view_model.get();

                if vm.routines.is_empty() {
                    view! {
                        <div class="text-center py-12 px-4 sm:px-6 lg:px-0">
                            <p class="text-gray-400">"No saved routines yet."</p>
                            <p class="text-sm text-gray-500 mt-2">"Save a setlist as a routine during session building or from the session summary."</p>
                            <div class="mt-6">
                                <A href="/sessions/new" attr:class="w-full sm:w-auto inline-flex items-center justify-center rounded-lg bg-indigo-600 px-4 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 motion-safe:transition-colors min-h-[44px]">
                                    "New Session"
                                </A>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="space-y-3">
                            {vm.routines.iter().map(|routine| {
                                view! {
                                    <RoutineRow routine=routine.clone() />
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                        <p class="text-sm text-gray-400 mt-4">
                            {format!("{} routine{}", vm.routines.len(), if vm.routines.len() == 1 { "" } else { "s" })}
                        </p>
                    }.into_any()
                }
            }}
        </div>
    }
}

/// A single routine row with name, entry count, edit link, and delete action.
#[component]
fn RoutineRow(routine: RoutineView) -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let confirm_delete = RwSignal::new(false);

    let id = routine.id.clone();
    let id_for_delete = routine.id.clone();
    let name = routine.name.clone();
    let entry_count = routine.entry_count;
    let entries = routine.entries.clone();
    let edit_href = format!("/routines/{}/edit", id);

    view! {
        <Card>
            {move || {
                if confirm_delete.get() {
                    let core_del = core.clone();
                    let id_del = id_for_delete.clone();
                    view! {
                        <div>
                            <p class="text-sm text-red-300 mb-3">"Delete this routine? This cannot be undone."</p>
                            <div class="flex gap-2">
                                <Button
                                    variant=ButtonVariant::Danger
                                    loading=Signal::derive(move || is_submitting.get())
                                    on_click=Callback::new(move |_| {
                                        let event = Event::Routine(RoutineEvent::DeleteRoutine { id: id_del.clone() });
                                        let core_ref = core_del.borrow();
                                        let effects = core_ref.process_event(event);
                                        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                    })
                                >
                                    {move || if is_submitting.get() { "Deleting\u{2026}" } else { "Confirm Delete" }}
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
                    let name = name.clone();
                    let entries = entries.clone();
                    let edit_href = edit_href.clone();
                    view! {
                        <div class="space-y-3">
                            <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
                                <div class="flex-1 min-w-0">
                                    <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1">
                                        <span class="text-sm font-medium text-white">{name}</span>
                                        <span class="inline-flex items-center rounded-full bg-indigo-500/20 px-2 py-0.5 text-xs font-medium text-indigo-300">
                                            {format!("{} item{}", entry_count, if entry_count == 1 { "" } else { "s" })}
                                        </span>
                                    </div>
                                </div>
                                <div class="flex gap-3 sm:ml-4">
                                    <A href=edit_href attr:class="text-xs text-indigo-300 hover:text-indigo-200 font-medium">
                                        "Edit"
                                    </A>
                                    <button
                                        class="text-xs text-red-400 hover:text-red-300 font-medium"
                                        on:click=move |_| { confirm_delete.set(true); }
                                    >
                                        "Delete"
                                    </button>
                                </div>
                            </div>
                            // Entry details
                            <div class="mt-1 pt-2 space-y-1.5">
                                {entries.into_iter().map(|entry| {
                                    view! {
                                        <div class="flex items-center gap-2 text-xs">
                                            <span class="text-white">{entry.item_title}</span>
                                            <span class="text-gray-500">{entry.item_type}</span>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        </div>
                    }.into_any()
                }
            }}
        </Card>
    }
}
