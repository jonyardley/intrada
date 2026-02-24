use leptos::prelude::*;

use intrada_core::{Event, RoutineEvent, SessionEvent, ViewModel};

use crate::components::{
    Button, ButtonVariant, Card, DropIndicator, RoutineLoader, RoutineSaveForm, SetlistEntryRow,
};
use intrada_web::core_bridge::process_effects;
use intrada_web::hooks::use_drag_reorder;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Setlist builder component: shows library items to add, current setlist, and controls.
#[component]
pub fn SetlistBuilder() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    let core_setlist = core.clone();
    let core_actions = core.clone();
    let core_library = core.clone();
    let core_routine_save = core.clone();
    let core_drag = core.clone();
    let core_session_intention = core.clone();

    // --- Drag-and-drop setup ---
    let setlist_container_ref = NodeRef::<leptos::html::Div>::new();

    let item_count = Signal::derive(move || {
        let vm = view_model.get();
        vm.building_setlist
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

    // Session intention signal — local UI state, dispatches to core on change
    let session_intention_value = RwSignal::new(String::new());

    view! {
        <div class="space-y-6">
            // Session-level intention
            <Card>
                <div>
                    <label class="form-label" for="session-intention">
                        "Session Intention"
                    </label>
                    <p class="hint-text">"Optional — set a focus for your practice session"</p>
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
            </Card>

            // Current setlist
            <Card>
                <h3 class="section-title">"Your Setlist"</h3>
                {move || {
                    let vm = view_model.get();
                    match vm.building_setlist {
                        Some(ref setlist) if !setlist.entries.is_empty() => {
                            let core_remove = core_setlist.clone();
                            let core_up = core.clone();
                            let core_down = core.clone();
                            let entries = setlist.entries.clone();
                            let entry_count = entries.len();
                            let core_entry_intention = core.clone();
                            let core_rep_target = core.clone();
                            let core_duration = core.clone();
                            view! {
                                <div node_ref=setlist_container_ref aria-roledescription="sortable">
                                    {entries.into_iter().enumerate().map(|(idx, entry)| {
                                        let core_r = core_remove.clone();
                                        let core_u = core_up.clone();
                                        let core_d = core_down.clone();
                                        let core_ei = core_entry_intention.clone();
                                        let core_rt = core_rep_target.clone();
                                        let core_dur = core_duration.clone();
                                        let on_remove = Callback::new(move |entry_id: String| {
                                            let event = Event::Session(SessionEvent::RemoveFromSetlist { entry_id });
                                            let core_ref = core_r.borrow();
                                            let effects = core_ref.process_event(event);
                                            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                        });
                                        let on_move_up = if idx > 0 {
                                            let core_mu = core_u.clone();
                                            Some(Callback::new(move |entry_id: String| {
                                                let event = Event::Session(SessionEvent::ReorderSetlist { entry_id, new_position: idx - 1 });
                                                let core_ref = core_mu.borrow();
                                                let effects = core_ref.process_event(event);
                                                process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                            }))
                                        } else {
                                            None
                                        };
                                        let on_move_down = if idx < entry_count - 1 {
                                            let core_md = core_d.clone();
                                            Some(Callback::new(move |entry_id: String| {
                                                let event = Event::Session(SessionEvent::ReorderSetlist { entry_id, new_position: idx + 1 });
                                                let core_ref = core_md.borrow();
                                                let effects = core_ref.process_event(event);
                                                process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                            }))
                                        } else {
                                            None
                                        };

                                        // Drag state for this entry
                                        let eid = entry.id.clone();
                                        let is_dragging_this = Signal::derive(move || {
                                            dragged_id.get().as_deref() == Some(eid.as_str())
                                        });

                                        // Drop indicator before this entry (visible when hover_index == idx)
                                        let drop_before_visible = Signal::derive(move || {
                                            drag_hover_index.get() == Some(idx)
                                        });

                                        // Drop indicator after the last entry
                                        let is_last = idx == entry_count - 1;
                                        let drop_after_visible = Signal::derive(move || {
                                            is_last && drag_hover_index.get() == Some(entry_count)
                                        });

                                        // Per-entry intention signal
                                        let entry_intention_id = entry.id.clone();
                                        let entry_intention_value = RwSignal::new(
                                            entry.intention.clone().unwrap_or_default()
                                        );

                                        // Per-entry rep target state
                                        let entry_rep_target_id = entry.id.clone();
                                        let entry_rep_target_id_clear = entry.id.clone();
                                        let has_rep_target = entry.rep_target.is_some();
                                        let current_rep_target = entry.rep_target.unwrap_or(intrada_core::validation::DEFAULT_REP_TARGET);
                                        let rep_target_value = RwSignal::new(current_rep_target.to_string());
                                        let core_rt_enable = core_rt.clone();
                                        let core_rt_clear = core_rt.clone();

                                        // Per-entry planned duration state
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
                                                on_move_up=on_move_up
                                                on_move_down=on_move_down
                                                is_dragging_this=is_dragging_this
                                                on_drag_pointer_down=Some(on_drag_pointer_down)
                                                index=idx
                                            />
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
                                                // Entry options: rep target + duration controls
                                                <div class="flex flex-wrap items-center gap-2">
                                                    // Rep target control
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
                                                                            view! {
                                                                                <option value=n.to_string() selected=selected>{n.to_string()}</option>
                                                                            }
                                                                        })
                                                                        .collect::<Vec<_>>()}
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
                                                                    "✕"
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
                                                            >
                                                                "+ Reps"
                                                            </button>
                                                        }.into_any()
                                                    }}
                                                    // Planned duration control
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
                                                                    {(1u32..=60)
                                                                        .map(|n| {
                                                                            let selected = n == current_duration_mins;
                                                                            let label = format!("{n} min");
                                                                            view! {
                                                                                <option value=n.to_string() selected=selected>{label}</option>
                                                                            }
                                                                        })
                                                                        .collect::<Vec<_>>()}
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
                                                                    "✕"
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
                                                            >
                                                                "+ Duration"
                                                            </button>
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
                                    "No items added yet. Select items from your library below."
                                </p>
                            }.into_any()
                        }
                    }
                }}
            </Card>

            // Error display (above buttons so it's visible without scrolling)
            {move || {
                let vm = view_model.get();
                vm.error.map(|err| {
                    view! {
                        <p class="text-sm text-danger-text">{err}</p>
                    }
                })
            }}

            // Action buttons
            <div class="flex gap-3">
                {
                    let core_start = core_actions.clone();
                    let core_cancel = core_actions.clone();
                    let setlist_empty = Signal::derive(move || {
                        let vm = view_model.get();
                        match &vm.building_setlist {
                            Some(setlist) => setlist.entries.is_empty(),
                            None => true,
                        }
                    });
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
                        >
                            "Start Session"
                        </Button>
                        <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| {
                            let event = Event::Session(SessionEvent::CancelBuilding);
                            let core_ref = core_cancel.borrow();
                            let effects = core_ref.process_event(event);
                            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                        })>
                            "Cancel"
                        </Button>
                    }
                }
            </div>

            // Save as Routine (only when setlist has entries)
            {move || {
                let vm = view_model.get();
                let has_entries = matches!(&vm.building_setlist, Some(setlist) if !setlist.entries.is_empty());
                if has_entries {
                    let core_save_routine = core_routine_save.clone();
                    Some(view! {
                        <RoutineSaveForm on_save=Callback::new(move |name: String| {
                            let event = Event::Routine(RoutineEvent::SaveBuildingAsRoutine { name });
                            let core_ref = core_save_routine.borrow();
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

            // Library items to add (T013: whole row is clickable)
            <Card>
                <h3 class="section-title">"Library Items"</h3>
                {move || {
                    let vm = view_model.get();
                    if vm.items.is_empty() {
                        view! {
                            <p class="text-sm text-muted">"No library items available."</p>
                        }.into_any()
                    } else {
                        let core_add = core_library.clone();
                        view! {
                            <div class="space-y-2">
                                {vm.items.iter().map(|item| {
                                    let item_id = item.id.clone();
                                    let title = item.title.clone();
                                    let item_type = item.item_type.clone();
                                    let core_a = core_add.clone();
                                    view! {
                                        <div
                                            class="flex items-center justify-between rounded-lg bg-surface-secondary px-3 py-2 hover:bg-surface-hover cursor-pointer"
                                            on:click=move |_| {
                                                let event = Event::Session(SessionEvent::AddToSetlist { item_id: item_id.clone() });
                                                let core_ref = core_a.borrow();
                                                let effects = core_ref.process_event(event);
                                                process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                            }
                                        >
                                            <div class="flex items-center gap-2">
                                                <span class="text-sm text-primary">{title}</span>
                                                <span class="text-xs text-faint">{item_type}</span>
                                            </div>
                                            <span class="text-xs font-medium text-accent-text">
                                                "+ Add"
                                            </span>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </Card>
        </div>
    }
}
