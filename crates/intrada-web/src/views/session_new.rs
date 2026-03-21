use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::{Event, RoutineEvent, SessionEvent, SessionStatusView, ViewModel};

use intrada_core::ItemKind;

use crate::components::{
    BackLink, Button, ButtonVariant, Card, DropIndicator, Icon, IconName, LibraryListRow,
    RoutineLoader, RoutineSaveForm, SetlistEntryRow, SlideUpSheet, StickyBottomBar,
};
use intrada_web::core_bridge::process_effects;
use intrada_web::hooks::use_drag_reorder;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Session creation view with tap-to-queue UX.
///
/// Desktop (≥768px): Split-view — library list on left, setlist panel on right.
/// Mobile (<768px): Full-screen library list with sticky bottom bar.
///   Tapping the bottom bar opens a slide-up sheet with the setlist.
///
/// If an active session exists, shows a recovery banner instead.
#[component]
pub fn SessionNewView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let navigate = use_navigate();

    let started_building = RwSignal::new(false);
    let sheet_open = RwSignal::new(false);
    let search_query = RwSignal::new(String::new());
    let filter_type = RwSignal::new(None::<ItemKind>);

    // Dispatch StartBuilding on mount (same as before)
    {
        let vm = view_model.get_untracked();
        if vm.session_status == SessionStatusView::Idle {
            let core_ref = core.borrow();
            let effects = core_ref.process_event(Event::Session(SessionEvent::StartBuilding));
            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
            started_building.set(true);
        }
    }

    // Navigate on state transitions
    Effect::new(move |_| {
        let vm = view_model.get();
        match vm.session_status {
            SessionStatusView::Active if started_building.get_untracked() => {
                navigate(
                    "/sessions/active",
                    NavigateOptions {
                        replace: true,
                        ..Default::default()
                    },
                );
            }
            SessionStatusView::Idle if started_building.get_untracked() => {
                navigate(
                    "/sessions",
                    NavigateOptions {
                        replace: true,
                        ..Default::default()
                    },
                );
            }
            _ => {}
        }
    });

    let core_abandon = core.clone();
    let navigate_resume = use_navigate();

    // Setlist item count and total minutes
    let setlist_count = Signal::derive(move || {
        view_model
            .get()
            .building_setlist
            .as_ref()
            .map(|s| s.entries.len())
            .unwrap_or(0)
    });

    let total_minutes = Signal::derive(move || {
        view_model
            .get()
            .building_setlist
            .as_ref()
            .map(|s| {
                s.entries
                    .iter()
                    .filter_map(|e| e.planned_duration_secs)
                    .sum::<u32>()
                    / 60
            })
            .unwrap_or(0)
    });

    let setlist_empty = Signal::derive(move || setlist_count.get() == 0);

    // Derive set of item IDs in the setlist (for tap-to-toggle highlight)
    let setlist_item_ids = Signal::derive(move || {
        view_model
            .get()
            .building_setlist
            .as_ref()
            .map(|s| {
                s.entries
                    .iter()
                    .map(|e| e.item_id.clone())
                    .collect::<std::collections::HashSet<String>>()
            })
            .unwrap_or_default()
    });

    view! {
        <div>
            // Active session recovery banner
            {move || {
                let vm = view_model.get();
                if vm.session_status == SessionStatusView::Active {
                    let core_a = core_abandon.clone();
                    let nav = navigate_resume.clone();
                    Some(view! {
                        <div class="p-4">
                            <BackLink label="Back to Practice" href="/sessions".to_string() />
                            <Card>
                                <div class="space-y-3">
                                    <p class="text-sm text-secondary">"You have a session in progress."</p>
                                    <div class="flex gap-3">
                                        <Button variant=ButtonVariant::Primary on_click=Callback::new(move |_| {
                                            nav("/sessions/active", NavigateOptions { replace: true, ..Default::default() });
                                        })>"Resume Session"</Button>
                                        <Button variant=ButtonVariant::Danger on_click=Callback::new(move |_| {
                                            let event = Event::Session(SessionEvent::AbandonSession);
                                            let core_ref = core_a.borrow();
                                            let effects = core_ref.process_event(event);
                                            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                        })>"Discard Session"</Button>
                                    </div>
                                </div>
                            </Card>
                        </div>
                    })
                } else {
                    None
                }
            }}

            // Session builder (only when building)
            <Show when=move || view_model.get().session_status == SessionStatusView::Building>
                <div class="flex h-full -mx-4 sm:-mx-6 -my-6 sm:-my-10">
                    // ── Left: Library list (tap-to-queue) ──
                    <div class="flex flex-col w-full md:w-80 md:shrink-0 md:border-r md:border-border-default md:overflow-y-auto">
                        <div class="p-4 space-y-3">
                            <div class="flex items-start justify-between gap-3">
                                <div>
                                    <BackLink label="Back to Practice" href="/sessions".to_string() />
                                    <h1 class="text-lg font-semibold text-primary mt-2">"New Session"</h1>
                                </div>
                            </div>
                            // Search field
                            <input
                                type="text"
                                class="input-base text-sm"
                                placeholder="Search library..."
                                bind:value=search_query
                            />
                            // Type filter (All / Pieces / Exercises)
                            <div class="inline-flex items-center rounded-full bg-surface-input p-1 gap-1">
                                {
                                    let tabs: Vec<(Option<ItemKind>, &str)> = vec![
                                        (None, "All"),
                                        (Some(ItemKind::Piece), "Pieces"),
                                        (Some(ItemKind::Exercise), "Exercises"),
                                    ];
                                    tabs.into_iter().map(|(kind, label)| {
                                        let kind_for_active = kind.clone();
                                        let kind_for_click = kind.clone();
                                        let is_active = Signal::derive(move || filter_type.get() == kind_for_active);
                                        view! {
                                            <button
                                                type="button"
                                                class=move || if is_active.get() {
                                                    "flex-1 inline-flex items-center justify-center px-3 py-1.5 text-xs font-medium rounded-full bg-accent text-primary"
                                                } else {
                                                    "flex-1 inline-flex items-center justify-center px-3 py-1.5 text-xs font-medium rounded-full text-muted hover:text-primary cursor-pointer"
                                                }
                                                on:click=move |_| filter_type.set(kind_for_click.clone())
                                            >
                                                {label}
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()
                                }
                            </div>
                        </div>

                        // Library item list
                        <div class="flex-1 overflow-y-auto pb-20 md:pb-4">
                            {
                                let core_for_list = core.clone();
                                move || {
                                let vm = view_model.get();
                                let selected_ids = setlist_item_ids.get();
                                let core_toggle = core_for_list.clone();
                                let query = search_query.get().to_lowercase();
                                let kind_filter = filter_type.get();

                                // Filter items by search query and type
                                let filtered_items: Vec<_> = vm.items.into_iter().filter(|item| {
                                    // Type filter
                                    if let Some(ref kind) = kind_filter {
                                        if item.item_type != *kind {
                                            return false;
                                        }
                                    }
                                    // Search filter
                                    if !query.is_empty() {
                                        let title_match = item.title.to_lowercase().contains(&query);
                                        let subtitle_match = item.subtitle.to_lowercase().contains(&query);
                                        if !title_match && !subtitle_match {
                                            return false;
                                        }
                                    }
                                    true
                                }).collect();

                                view! {
                                    <ul role="list" aria-label="Library items">
                                        {filtered_items.into_iter().map(|item| {
                                            let item_id = item.id.clone();
                                            let is_in_setlist = selected_ids.contains(&item_id);
                                            let core_t = core_toggle.clone();
                                            let toggle_id = item_id.clone();

                                            // Find the entry_id for removal (if already in setlist)
                                            let entry_id_for_remove = if is_in_setlist {
                                                vm.building_setlist.as_ref().and_then(|s| {
                                                    s.entries.iter().find(|e| e.item_id == toggle_id).map(|e| e.id.clone())
                                                })
                                            } else {
                                                None
                                            };

                                            view! {
                                                <LibraryListRow
                                                    item=item
                                                    is_selected=is_in_setlist
                                                    show_selection=true
                                                    on_click=Callback::new(move |()| {
                                                        let event = if let Some(ref eid) = entry_id_for_remove {
                                                            Event::Session(SessionEvent::RemoveFromSetlist { entry_id: eid.clone() })
                                                        } else {
                                                            Event::Session(SessionEvent::AddToSetlist { item_id: toggle_id.clone() })
                                                        };
                                                        let core_ref = core_t.borrow();
                                                        let effects = core_ref.process_event(event);
                                                        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                                    })
                                                />
                                            }
                                        }).collect::<Vec<_>>()}
                                    </ul>
                                }
                            }}
                        </div>
                    </div>

                    // ── Right: Setlist panel (desktop only) ──
                    <div class="hidden md:flex md:flex-col md:flex-1 md:overflow-y-auto">
                        <div class="p-4 sm:p-6 space-y-4">
                            <SetlistPanel />
                        </div>
                    </div>
                </div>

                // ── Mobile: Sticky bottom bar + slide-up sheet ──
                {
                    let core_start_mobile = core.clone();
                    view! {
                        <StickyBottomBar
                            item_count=setlist_count
                            total_minutes=total_minutes
                            disabled=setlist_empty
                            on_summary_click=Callback::new(move |()| sheet_open.set(true))
                            on_start=Callback::new(move |()| {
                                let now = chrono::Utc::now();
                                let event = Event::Session(SessionEvent::StartSession { now });
                                let core_ref = core_start_mobile.borrow();
                                let effects = core_ref.process_event(event);
                                process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                            })
                        />
                    }
                }

                <SlideUpSheet
                    is_open=Signal::derive(move || sheet_open.get())
                    on_dismiss=Callback::new(move |()| sheet_open.set(false))
                >
                    <SetlistPanel />
                </SlideUpSheet>
            </Show>
        </div>
    }
}

/// Setlist panel content — renders in the desktop right pane or inside the mobile slide-up sheet.
/// Contains: session intention, setlist entries with drag-reorder, action buttons, routine load/save.
#[component]
fn SetlistPanel() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    let core_drag = core.clone();
    let core_actions = core.clone();
    let core_routine_save = core.clone();
    let core_session_intention = core.clone();

    // Drag-and-drop setup
    let setlist_container_ref = NodeRef::<leptos::html::Div>::new();

    let item_count = Signal::derive(move || {
        view_model
            .get()
            .building_setlist
            .as_ref()
            .map(|s| s.entries.len())
            .unwrap_or(0)
    });

    let on_reorder = Callback::new(move |(entry_id, new_position): (String, usize)| {
        let event = Event::Session(SessionEvent::ReorderSetlist {
            entry_id,
            new_position,
        });
        let core_ref = core_drag.borrow();
        let effects = core_ref.process_event(event);
        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
    });

    let drag = use_drag_reorder(on_reorder, item_count, setlist_container_ref);
    let dragged_id = drag.dragged_id;
    let drag_hover_index = drag.hover_index;
    let on_drag_pointer_down = drag.on_pointer_down;

    let session_intention_value = RwSignal::new(String::new());

    let setlist_empty = Signal::derive(move || {
        view_model
            .get()
            .building_setlist
            .as_ref()
            .map(|s| s.entries.is_empty())
            .unwrap_or(true)
    });

    view! {
        <div class="space-y-4">
            <h2 class="text-lg font-semibold text-primary">"Your Setlist"</h2>

            // Session intention
            <div>
                <label class="form-label" for="session-intention">"Practice Intention"</label>
                <p class="hint-text">"Optional — set a focus for your practice"</p>
                <input
                    id="session-intention"
                    type="text"
                    class="input-base"
                    placeholder="What will you focus on today?"
                    bind:value=session_intention_value
                    on:input=move |_| {
                        let value = session_intention_value.get();
                        let intention = if value.is_empty() { None } else { Some(value) };
                        let event = Event::Session(SessionEvent::SetSessionIntention { intention });
                        let core_ref = core_session_intention.borrow();
                        let effects = core_ref.process_event(event);
                        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                    }
                />
            </div>

            // Setlist entries with drag-reorder
            {move || {
                let vm = view_model.get();
                match vm.building_setlist {
                    Some(ref setlist) if !setlist.entries.is_empty() => {
                        let entries = setlist.entries.clone();
                        let entry_count = entries.len();
                        let core_remove = core.clone();
                        let core_entry_intention = core.clone();
                        let core_rep_target = core.clone();
                        let core_duration = core.clone();
                        view! {
                            <div node_ref=setlist_container_ref aria-roledescription="sortable" class="space-y-1">
                                {entries.into_iter().enumerate().map(|(idx, entry)| {
                                    let core_r = core_remove.clone();
                                    let core_ei = core_entry_intention.clone();
                                    let core_rt = core_rep_target.clone();
                                    let core_dur = core_duration.clone();

                                    let on_remove = Callback::new(move |entry_id: String| {
                                        let event = Event::Session(SessionEvent::RemoveFromSetlist { entry_id });
                                        let core_ref = core_r.borrow();
                                        let effects = core_ref.process_event(event);
                                        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                    });

                                    let eid = entry.id.clone();
                                    let is_dragging_this = Signal::derive(move || {
                                        dragged_id.get().as_deref() == Some(eid.as_str())
                                    });

                                    let drop_before_visible = Signal::derive(move || {
                                        drag_hover_index.get() == Some(idx)
                                    });

                                    let is_last = idx == entry_count - 1;
                                    let drop_after_visible = Signal::derive(move || {
                                        is_last && drag_hover_index.get() == Some(entry_count)
                                    });

                                    // Per-entry intention
                                    let entry_intention_id = entry.id.clone();
                                    let entry_intention_value = RwSignal::new(entry.intention.clone().unwrap_or_default());

                                    // Per-entry rep target
                                    let entry_rep_target_id = entry.id.clone();
                                    let entry_rep_target_id_clear = entry.id.clone();
                                    let has_rep_target = entry.rep_target.is_some();
                                    let current_rep_target = entry.rep_target.unwrap_or(intrada_core::validation::DEFAULT_REP_TARGET);
                                    let rep_target_value = RwSignal::new(current_rep_target.to_string());
                                    let core_rt_enable = core_rt.clone();
                                    let core_rt_clear = core_rt.clone();

                                    // Per-entry planned duration
                                    let entry_duration_id = entry.id.clone();
                                    let entry_duration_id_clear = entry.id.clone();
                                    let has_planned_duration = entry.planned_duration_secs.is_some();
                                    let current_duration_mins = entry.planned_duration_secs.map(|s| s / 60).unwrap_or(5);
                                    let duration_value = RwSignal::new(current_duration_mins.to_string());
                                    let core_dur_set = core_dur.clone();
                                    let core_dur_clear = core_dur.clone();

                                    view! {
                                        <DropIndicator visible=drop_before_visible />
                                        <SetlistEntryRow
                                            entry=entry
                                            on_remove=Some(on_remove)
                                            on_move_up=None
                                            on_move_down=None
                                            is_dragging_this=is_dragging_this
                                            on_drag_pointer_down=Some(on_drag_pointer_down)
                                            index=idx
                                        />
                                        // Entry detail controls (intention, reps, duration)
                                        <div class="ml-9 mb-3 space-y-2">
                                            <input
                                                type="text"
                                                class="input-base text-xs"
                                                placeholder="What will you focus on?"
                                                bind:value=entry_intention_value
                                                on:input=move |_| {
                                                    let value = entry_intention_value.get();
                                                    let intention = if value.is_empty() { None } else { Some(value) };
                                                    let event = Event::Session(SessionEvent::SetEntryIntention {
                                                        entry_id: entry_intention_id.clone(),
                                                        intention,
                                                    });
                                                    let core_ref = core_ei.borrow();
                                                    let effects = core_ref.process_event(event);
                                                    process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                                }
                                            />
                                            <div class="flex flex-wrap items-center gap-2">
                                                {if has_rep_target {
                                                    view! {
                                                        <div class="flex items-center gap-1.5 rounded-lg bg-surface-secondary px-2.5 py-1.5">
                                                            <span class="text-xs text-muted">"Reps:"</span>
                                                            <select
                                                                class="input-base text-xs w-14 py-0.5 px-1"
                                                                on:change=move |ev| {
                                                                    let value = leptos::prelude::event_target_value(&ev);
                                                                    rep_target_value.set(value.clone());
                                                                    if let Ok(target) = value.parse::<u8>() {
                                                                        let event = Event::Session(SessionEvent::SetRepTarget {
                                                                            entry_id: entry_rep_target_id.clone(),
                                                                            target: Some(target),
                                                                        });
                                                                        let core_ref = core_rt_enable.borrow();
                                                                        let effects = core_ref.process_event(event);
                                                                        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                                                    }
                                                                }
                                                            >
                                                                {(intrada_core::validation::MIN_REP_TARGET..=intrada_core::validation::MAX_REP_TARGET)
                                                                    .map(|n| {
                                                                        let selected = n == current_rep_target;
                                                                        view! { <option value=n.to_string() selected=selected>{n.to_string()}</option> }
                                                                    }).collect::<Vec<_>>()}
                                                            </select>
                                                            <button
                                                                class="text-xs text-muted hover:text-danger-text motion-safe:transition-colors"
                                                                title="Remove rep target"
                                                                on:click=move |_| {
                                                                    let event = Event::Session(SessionEvent::SetRepTarget {
                                                                        entry_id: entry_rep_target_id_clear.clone(),
                                                                        target: None,
                                                                    });
                                                                    let core_ref = core_rt_clear.borrow();
                                                                    let effects = core_ref.process_event(event);
                                                                    process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                                                }
                                                            >
                                                                <Icon name=IconName::X class="w-3.5 h-3.5" />
                                                            </button>
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <button
                                                            class="rounded-lg border border-border-default px-2.5 py-1.5 text-xs text-muted hover:text-accent-text hover:border-accent-text/30 motion-safe:transition-colors"
                                                            on:click=move |_| {
                                                                let event = Event::Session(SessionEvent::SetRepTarget {
                                                                    entry_id: entry_rep_target_id.clone(),
                                                                    target: Some(intrada_core::validation::DEFAULT_REP_TARGET),
                                                                });
                                                                let core_ref = core_rt_enable.borrow();
                                                                let effects = core_ref.process_event(event);
                                                                process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                                            }
                                                        >"+ Reps"</button>
                                                    }.into_any()
                                                }}
                                                {if has_planned_duration {
                                                    view! {
                                                        <div class="flex items-center gap-1.5 rounded-lg bg-surface-secondary px-2.5 py-1.5">
                                                            <span class="text-xs text-muted">"Duration:"</span>
                                                            <select
                                                                class="input-base text-xs w-18 py-0.5 px-1"
                                                                on:change=move |ev| {
                                                                    let value = leptos::prelude::event_target_value(&ev);
                                                                    duration_value.set(value.clone());
                                                                    if let Ok(mins) = value.parse::<u32>() {
                                                                        let secs = mins * 60;
                                                                        let event = Event::Session(SessionEvent::SetEntryDuration {
                                                                            entry_id: entry_duration_id.clone(),
                                                                            duration_secs: Some(secs),
                                                                        });
                                                                        let core_ref = core_dur_set.borrow();
                                                                        let effects = core_ref.process_event(event);
                                                                        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                                                    }
                                                                }
                                                            >
                                                                {(1u32..=60).map(|n| {
                                                                    let selected = n == current_duration_mins;
                                                                    let label = format!("{n} min");
                                                                    view! { <option value=n.to_string() selected=selected>{label}</option> }
                                                                }).collect::<Vec<_>>()}
                                                            </select>
                                                            <button
                                                                class="text-xs text-muted hover:text-danger-text motion-safe:transition-colors"
                                                                title="Remove planned duration"
                                                                on:click=move |_| {
                                                                    let event = Event::Session(SessionEvent::SetEntryDuration {
                                                                        entry_id: entry_duration_id_clear.clone(),
                                                                        duration_secs: None,
                                                                    });
                                                                    let core_ref = core_dur_clear.borrow();
                                                                    let effects = core_ref.process_event(event);
                                                                    process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                                                }
                                                            >
                                                                <Icon name=IconName::X class="w-3.5 h-3.5" />
                                                            </button>
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <button
                                                            class="rounded-lg border border-border-default px-2.5 py-1.5 text-xs text-muted hover:text-accent-text hover:border-accent-text/30 motion-safe:transition-colors"
                                                            on:click=move |_| {
                                                                let event = Event::Session(SessionEvent::SetEntryDuration {
                                                                    entry_id: entry_duration_id.clone(),
                                                                    duration_secs: Some(5 * 60),
                                                                });
                                                                let core_ref = core_dur_set.borrow();
                                                                let effects = core_ref.process_event(event);
                                                                process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                                            }
                                                        >"+ Duration"</button>
                                                    }.into_any()
                                                }}
                                            </div>
                                        </div>
                                        {if is_last {
                                            Some(view! { <DropIndicator visible=drop_after_visible /> })
                                        } else {
                                            None
                                        }}
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                    _ => {
                        view! {
                            <p class="text-sm text-muted text-center py-4">
                                "Tap items from the library to build your setlist."
                            </p>
                        }.into_any()
                    }
                }
            }}

            // Error display
            {move || {
                view_model.get().error.map(|err| view! {
                    <p class="text-sm text-danger-text">{err}</p>
                })
            }}

            // Action buttons
            <div class="flex gap-3">
                {
                    let core_start = core_actions.clone();
                    let core_cancel = core_actions.clone();
                    view! {
                        <Button
                            variant=ButtonVariant::Primary
                            disabled=setlist_empty
                            on_click=Callback::new(move |_| {
                                let now = chrono::Utc::now();
                                let event = Event::Session(SessionEvent::StartSession { now });
                                let core_ref = core_start.borrow();
                                let effects = core_ref.process_event(event);
                                process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                            })
                        >"Start Session"</Button>
                        <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| {
                            let event = Event::Session(SessionEvent::CancelBuilding);
                            let core_ref = core_cancel.borrow();
                            let effects = core_ref.process_event(event);
                            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                        })>"Cancel"</Button>
                    }
                }
            </div>

            // Save as Routine
            {move || {
                let vm = view_model.get();
                let has_entries = matches!(&vm.building_setlist, Some(setlist) if !setlist.entries.is_empty());
                if has_entries {
                    let core_save = core_routine_save.clone();
                    Some(view! {
                        <RoutineSaveForm on_save=Callback::new(move |name: String| {
                            let event = Event::Routine(RoutineEvent::SaveBuildingAsRoutine { name });
                            let core_ref = core_save.borrow();
                            let effects = core_ref.process_event(event);
                            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                        }) />
                    })
                } else {
                    None
                }
            }}

            // Load saved routines
            <RoutineLoader />
        </div>
    }
}
