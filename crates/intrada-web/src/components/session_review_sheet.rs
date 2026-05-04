use leptos::prelude::*;

use intrada_core::{Event, RoutineEvent, SessionEvent, ViewModel};

use crate::components::{
    BottomSheet, Button, ButtonVariant, DropIndicator, Icon, IconName, RoutineSaveForm,
    SetlistEntryRow,
};
use intrada_web::core_bridge::process_effects;
use intrada_web::hooks::use_drag_reorder;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Bottom sheet that opens from the Review Session CTA in the builder.
/// Holds the editable view of the setlist: intention input, drag-reorderable
/// rows with per-entry rep / duration controls, total minutes, and the
/// Start Session CTA.
///
/// Library item selection happens on the page behind the sheet — this sheet
/// is purely about reviewing and tuning what the user already chose.
#[component]
pub fn SessionReviewSheet(open: Signal<bool>, on_close: Callback<()>) -> impl IntoView {
    view! {
        <BottomSheet open=open on_close=on_close nav_title="Review session".to_string()>
            <ReviewSheetBody />
        </BottomSheet>
    }
}

/// Body of the review sheet rendered inside [`BottomSheet`]'s `ChildrenFn`.
/// Lives in its own component so all the per-entry move-closures don't
/// have to satisfy `Fn` for the BottomSheet's children prop — the Leptos
/// component boundary breaks the closure-trait dependency chain.
#[component]
fn ReviewSheetBody() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    let core_intention = core.clone();
    let core_remove = core.clone();
    let core_drag = core.clone();
    let core_start = core.clone();
    let core_save_routine = core.clone();
    let core_entry_intention = core.clone();
    let core_rep_target = core.clone();
    let core_duration = core.clone();

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

    // Local intention signal seeded from VM each open.
    let session_intention_value = RwSignal::new(String::new());
    Effect::new(move |_| {
        let vm = view_model.get();
        let current = vm
            .building_setlist
            .as_ref()
            .and_then(|s| s.session_intention.clone())
            .unwrap_or_default();
        if session_intention_value.get_untracked() != current {
            session_intention_value.set(current);
        }
    });

    view! {
        <div class="flex flex-col gap-5 pb-6">
            <div class="flex items-center justify-between">
                <h3 class="card-title">"Your setlist"</h3>
                {move || {
                    let vm = view_model.get();
                    let totals = vm.building_setlist.as_ref().map(|s| {
                        let total: u32 = s.entries.iter()
                            .map(|e| e.planned_duration_secs.unwrap_or(intrada_core::validation::DEFAULT_PLANNED_DURATION_SECS))
                            .sum::<u32>() / 60;
                        (total, s.target_duration_mins)
                    });
                    totals.map(|(total, target)| match target {
                        Some(t) => view! {
                            <span class="text-xs font-medium text-muted">
                                {format!("{total} / {t} min")}
                            </span>
                        }.into_any(),
                        None => view! {
                            <span class="text-xs font-medium text-muted">
                                {format!("Total: {total} min")}
                            </span>
                        }.into_any(),
                    })
                }}
            </div>

            <div>
                <label class="form-label" for="session-intention">"Session intention"</label>
                <input
                    id="session-intention"
                    type="text"
                    class="input-base"
                    placeholder="e.g. Focus on dynamics"
                    bind:value=session_intention_value
                    on:input=move |_| {
                        let value = session_intention_value.get();
                        let intention = if value.is_empty() { None } else { Some(value) };
                        let event = Event::Session(SessionEvent::SetSessionIntention { intention });
                        let core_ref = core_intention.borrow();
                        let effects = core_ref.process_event(event);
                        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                    }
                />
            </div>

            {move || {
                let vm = view_model.get();
                match vm.building_setlist {
                    Some(ref setlist) if !setlist.entries.is_empty() => {
                        let entries = setlist.entries.clone();
                        let entry_count = entries.len();
                        let core_r = core_remove.clone();
                        let core_ei = core_entry_intention.clone();
                        let core_rt = core_rep_target.clone();
                        let core_dur = core_duration.clone();
                        view! {
                            <div node_ref=setlist_container_ref aria-roledescription="sortable" class="space-y-3">
                                {entries.into_iter().enumerate().map(|(idx, entry)| {
                                    let core_r2 = core_r.clone();
                                    let core_ei2 = core_ei.clone();
                                    let core_rt_set = core_rt.clone();
                                    let core_rt_clear = core_rt.clone();
                                    let core_dur_set = core_dur.clone();
                                    let core_dur_clear = core_dur.clone();

                                    let on_remove = Callback::new(move |entry_id: String| {
                                        let event = Event::Session(SessionEvent::RemoveFromSetlist { entry_id });
                                        let core_ref = core_r2.borrow();
                                        let effects = core_ref.process_event(event);
                                        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                    });

                                    let eid = entry.id.clone();
                                    let is_dragging_this = Signal::derive(move || {
                                        dragged_id.get().as_deref() == Some(eid.as_str())
                                    });
                                    let drop_before_visible = Signal::derive(move || drag_hover_index.get() == Some(idx));
                                    let is_last = idx == entry_count - 1;
                                    let drop_after_visible = Signal::derive(move || is_last && drag_hover_index.get() == Some(entry_count));

                                    let entry_intention_id = entry.id.clone();
                                    let entry_intention_value = RwSignal::new(entry.intention.clone().unwrap_or_default());

                                    let entry_rep_target_id = entry.id.clone();
                                    let entry_rep_target_id_clear = entry.id.clone();
                                    let has_rep_target = entry.rep_target.is_some();
                                    let current_rep_target = entry.rep_target.unwrap_or(intrada_core::validation::DEFAULT_REP_TARGET);
                                    let rep_target_value = RwSignal::new(current_rep_target.to_string());

                                    let entry_duration_id = entry.id.clone();
                                    let entry_duration_id_clear = entry.id.clone();
                                    let has_planned_duration = entry.planned_duration_secs.is_some();
                                    let current_duration_mins = entry.planned_duration_secs.map(|s| s / 60).unwrap_or(5);
                                    let duration_value = RwSignal::new(current_duration_mins.to_string());

                                    view! {
                                        <DropIndicator visible=drop_before_visible />
                                        <div>
                                            <SetlistEntryRow
                                                entry=entry
                                                on_remove=Some(on_remove)
                                                show_controls=true
                                                is_dragging_this=is_dragging_this
                                                on_drag_pointer_down=Some(on_drag_pointer_down)
                                                index=idx
                                            />
                                            <div class="ml-9 mt-2 flex flex-wrap items-center gap-2">
                                                <input
                                                    type="text"
                                                    class="input-base text-xs flex-1 min-w-32"
                                                    placeholder="What will you focus on?"
                                                    bind:value=entry_intention_value
                                                    on:input=move |_| {
                                                        let value = entry_intention_value.get();
                                                        let intention = if value.is_empty() { None } else { Some(value) };
                                                        let event = Event::Session(SessionEvent::SetEntryIntention {
                                                            entry_id: entry_intention_id.clone(),
                                                            intention,
                                                        });
                                                        let core_ref = core_ei2.borrow();
                                                        let effects = core_ref.process_event(event);
                                                        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                                    }
                                                />
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
                                                                        let core_ref = core_rt_set.borrow();
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
                                                                let core_ref = core_rt_set.borrow();
                                                                let effects = core_ref.process_event(event);
                                                                process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                                            }
                                                        >
                                                            "+ Reps"
                                                        </button>
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
                                                                    duration_secs: Some(intrada_core::validation::DEFAULT_PLANNED_DURATION_SECS),
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
                    _ => view! {
                        <p class="text-sm text-muted text-center py-6">
                            "No items in your setlist yet. Tap items in the library to add them."
                        </p>
                    }.into_any(),
                }
            }}

            {move || {
                let vm = view_model.get();
                vm.error.map(|err| view! {
                    <p class="text-sm text-danger-text">{err}</p>
                })
            }}

            {
                let setlist_empty = Signal::derive(move || {
                    view_model.get().building_setlist
                        .map(|s| s.entries.is_empty())
                        .unwrap_or(true)
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
                }
            }

            {move || {
                let vm = view_model.get();
                let has_entries = matches!(&vm.building_setlist, Some(s) if !s.entries.is_empty());
                if has_entries {
                    let core_save = core_save_routine.clone();
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
        </div>
    }
}
