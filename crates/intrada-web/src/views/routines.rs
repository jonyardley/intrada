use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::{Event, RoutineEvent, RoutineView, ViewModel};

use crate::components::{
    Button, ButtonVariant, ContextMenu, ContextMenuAction, EmptyState, GroupedList, GroupedListRow,
    IconName, PageHeading, SkeletonCardList, SwipeActions,
};
use intrada_web::core_bridge::{process_effects, process_effects_with_core};
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Management page for saved routines — lists all routines with edit/delete actions.
#[component]
pub fn RoutinesListView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();

    view! {
        <div>
            <PageHeading text="Routines" subtitle="Save and reuse your favourite practice structures." />

            {move || {
                if is_loading.get() {
                    return view! {
                        <SkeletonCardList count=2 />
                    }.into_any();
                }

                let vm = view_model.get();

                if vm.routines.is_empty() {
                    view! {
                        <EmptyState
                            icon=IconName::ListChecks
                            title="No saved routines yet"
                            body="Save a setlist as a routine when building a session or from the session summary."
                        >
                            <A href="/sessions/new" attr:class="cta-link">
                                "New Session"
                            </A>
                        </EmptyState>
                    }.into_any()
                } else {
                    let routine_count = vm.routines.len();
                    view! {
                        <GroupedList aria_label="Saved routines">
                            {vm.routines.into_iter().map(|routine| {
                                view! {
                                    <GroupedListRow>
                                        <RoutineRow routine=routine />
                                    </GroupedListRow>
                                }
                            }).collect::<Vec<_>>()}
                        </GroupedList>
                        <p class="text-sm text-muted mt-4">
                            {format!("{} routine{}", routine_count, if routine_count == 1 { "" } else { "s" })}
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
    let id_for_swipe = routine.id.clone();
    let id_for_menu_delete = routine.id.clone();
    let name = routine.name.clone();
    let entry_count = routine.entry_count;
    let entries = routine.entries.clone();
    let edit_href = format!("/routines/{}/edit", id);
    let edit_href_for_menu = edit_href.clone();

    // Direct-delete callback used by both the iOS swipe-to-delete gesture
    // and the long-press context menu's Delete action. Skips the in-card
    // confirmation banner — the swipe / long-press gesture is itself the
    // deliberate confirmation, matching native UISwipeActionsConfiguration.
    let core_for_gesture = core.clone();
    let direct_delete = Callback::new(move |routine_id: String| {
        let event = Event::Routine(RoutineEvent::DeleteRoutine { id: routine_id });
        let effects = {
            let core_ref = core_for_gesture.borrow();
            core_ref.process_event(event)
        };
        process_effects_with_core(
            &core_for_gesture,
            effects,
            &view_model,
            &is_loading,
            &is_submitting,
        );
    });

    let menu_actions = vec![
        ContextMenuAction {
            label: "Edit".to_string(),
            destructive: false,
            on_select: Callback::new(move |_| {
                let navigate = use_navigate();
                navigate(&edit_href_for_menu, NavigateOptions::default());
            }),
        },
        ContextMenuAction {
            label: "Delete".to_string(),
            destructive: true,
            on_select: Callback::new(move |_| {
                direct_delete.run(id_for_menu_delete.clone());
            }),
        },
    ];

    view! {
        {move || {
            if confirm_delete.get() {
                let core_del = core.clone();
                let id_del = id_for_delete.clone();
                view! {
                    <div class="p-card sm:p-card-comfortable">
                        <p class="text-sm text-danger-text mb-3">"Delete this routine? This cannot be undone."</p>
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
                let menu_actions = menu_actions.clone();
                let id_for_swipe = id_for_swipe.clone();
                view! {
                    <ContextMenu actions=menu_actions>
                        <SwipeActions on_delete=Callback::new(move |_| {
                            direct_delete.run(id_for_swipe.clone());
                        })>
                            <div class="p-card sm:p-card-comfortable space-y-3">
                                    <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
                                        <div class="flex-1 min-w-0">
                                            <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1">
                                                <span class="text-sm font-medium text-primary">{name}</span>
                                                <span class="inline-flex items-center rounded-full bg-badge-piece-bg px-2 py-0.5 text-xs font-medium text-accent-text">
                                                    {format!("{} item{}", entry_count, if entry_count == 1 { "" } else { "s" })}
                                                </span>
                                            </div>
                                        </div>
                                        <div class="flex gap-3 sm:ml-4">
                                            <A href=edit_href attr:class="text-xs text-accent-text hover:text-accent-hover font-medium">
                                                "Edit"
                                            </A>
                                            <button
                                                class="text-xs text-danger-text hover:text-danger-hover font-medium"
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
                                                    <span class="text-primary">{entry.item_title}</span>
                                                    <span class="text-faint">{entry.item_type.to_string()}</span>
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                            </div>
                        </SwipeActions>
                    </ContextMenu>
                }.into_any()
            }
        }}
    }
}
