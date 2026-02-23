use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use intrada_core::{Event, SessionEvent, ViewModel};

use crate::components::{Button, ButtonVariant, Card, SetlistEntryRow, TypeBadge};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Active session timer: shows current item, elapsed time, progress, and controls.
#[component]
pub fn SessionTimer() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    let elapsed_secs = RwSignal::new(0u32);
    let interval_id: RwSignal<Option<i32>> = RwSignal::new(None);
    // UI-only visibility signal for the rep counter (does not affect domain state)
    let rep_counter_visible = RwSignal::new(false);

    // Start the display timer
    {
        let closure = Closure::<dyn Fn()>::new(move || {
            elapsed_secs.update(|s| *s += 1);
        });
        if let Some(window) = web_sys::window() {
            if let Ok(id) = window.set_interval_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                1000,
            ) {
                interval_id.set(Some(id));
            }
        }
        closure.forget();
    }

    on_cleanup(move || {
        if let Some(id) = interval_id.get_untracked() {
            if let Some(window) = web_sys::window() {
                window.clear_interval_with_handle(id);
            }
        }
    });

    view! {
        <div class="space-y-4">
            {move || {
                let vm = view_model.get();
                match vm.active_session {
                    Some(ref active) => {
                        let core_next = core.clone();
                        let core_skip = core.clone();
                        let core_finish = core.clone();
                        let core_end = core.clone();
                        let core_got_it = core.clone();
                        let core_missed = core.clone();
                        let core_init_rep = core.clone();
                        let current_title = active.current_item_title.clone();
                        let current_type = active.current_item_type.clone();
                        let position = active.current_position;
                        let total = active.total_items;
                        let is_last = position == total - 1;
                        let current_entry_intention = active.entries
                            .get(position)
                            .and_then(|e| e.intention.clone());
                        let session_intention = active.session_intention.clone();
                        let completed_entries: Vec<_> = active.entries.iter()
                            .filter(|e| e.status == "completed" || e.status == "skipped")
                            .cloned()
                            .collect();

                        // Rep counter state for the current item
                        let rep_target = active.current_rep_target;
                        let rep_count = active.current_rep_count;
                        let rep_target_reached = active.current_rep_target_reached;
                        let has_rep_state = rep_target.is_some();
                        // Auto-show counter when entry has rep state from building phase;
                        // auto-hide when navigating to an item without rep state.
                        if has_rep_state && !rep_counter_visible.get_untracked() {
                            rep_counter_visible.set(true);
                        } else if !has_rep_state && rep_counter_visible.get_untracked() {
                            rep_counter_visible.set(false);
                        }
                        let show_counter = rep_counter_visible.get_untracked() || has_rep_state;

                        view! {
                            // Session intention (above the current item card)
                            {session_intention.map(|intention| view! {
                                <p class="text-sm text-secondary text-center italic">{intention}</p>
                            })}

                            // Current item card
                            <Card>
                                <div class="text-center space-y-3">
                                    <p class="text-xs text-muted uppercase tracking-wider">
                                        {format!("Item {} of {}", position + 1, total)}
                                    </p>
                                    <h2 class="text-2xl font-bold text-primary">{current_title}</h2>
                                    // Entry-level intention (below the item title)
                                    {current_entry_intention.map(|intention| view! {
                                        <p class="text-sm text-muted">{intention}</p>
                                    })}
                                    <TypeBadge item_type=current_type />
                                    <p class="text-4xl sm:text-6xl font-mono font-bold text-primary mt-4">
                                        {move || {
                                            let secs = elapsed_secs.get();
                                            format!("{:02}:{:02}", secs / 60, secs % 60)
                                        }}
                                    </p>
                                </div>
                            </Card>

                            // Rep counter section
                            {if show_counter {
                                let target = rep_target.unwrap_or(0);
                                let count = rep_count.unwrap_or(0);
                                let reached = rep_target_reached.unwrap_or(false);
                                let progress_pct = if target > 0 {
                                    ((count as f64 / target as f64) * 100.0).min(100.0)
                                } else {
                                    0.0
                                };

                                view! {
                                    <Card>
                                        <div class="space-y-4">
                                            // Counter display + progress bar
                                            <div class="text-center space-y-2">
                                                <p class="text-xs text-muted uppercase tracking-wider">"Consecutive Reps"</p>
                                                {if reached {
                                                    view! {
                                                        <p class="text-4xl font-mono font-bold text-warm-accent-text">
                                                            {format!("{} / {}", count, target)}
                                                        </p>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <p class="text-4xl font-mono font-bold text-primary">
                                                            {format!("{} / {}", count, target)}
                                                        </p>
                                                    }.into_any()
                                                }}
                                                // Progress bar
                                                <div class="w-full h-2 rounded-full bg-surface-secondary overflow-hidden">
                                                    <div
                                                        class={if reached {
                                                            "h-full rounded-full bg-warm-accent motion-safe:transition-all motion-safe:duration-300"
                                                        } else {
                                                            "h-full rounded-full bg-success motion-safe:transition-all motion-safe:duration-300"
                                                        }}
                                                        style=format!("width: {}%", progress_pct)
                                                    />
                                                </div>
                                            </div>

                                            {if reached {
                                                // Achievement state — target reached
                                                view! {
                                                    <p class="text-sm font-semibold text-warm-accent-text text-center">"Target reached!"</p>
                                                }.into_any()
                                            } else {
                                                // Active counting buttons
                                                view! {
                                                    <div class="flex gap-3 justify-center">
                                                        <Button variant=ButtonVariant::Success on_click=Callback::new(move |_| {
                                                            let event = Event::Session(SessionEvent::RepGotIt);
                                                            let core_ref = core_got_it.borrow();
                                                            let effects = core_ref.process_event(event);
                                                            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                                        })>
                                                            "Got it"
                                                        </Button>
                                                        <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| {
                                                            let event = Event::Session(SessionEvent::RepMissed);
                                                            let core_ref = core_missed.borrow();
                                                            let effects = core_ref.process_event(event);
                                                            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                                        })>
                                                            "Missed"
                                                        </Button>
                                                    </div>
                                                }.into_any()
                                            }}

                                            // Hide counter link (UI-only toggle, preserves rep state)
                                            <div class="text-center">
                                                <button
                                                    class="text-xs text-muted hover:text-secondary motion-safe:transition-colors"
                                                    on:click=move |_| {
                                                        rep_counter_visible.set(false);
                                                    }
                                                >
                                                    "Hide counter"
                                                </button>
                                            </div>
                                        </div>
                                    </Card>
                                }.into_any()
                            } else {
                                // Counter hidden — show enable/show button
                                view! {
                                    <div class="text-center">
                                        <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| {
                                            rep_counter_visible.set(true);
                                            // Only dispatch InitRepCounter when no rep state exists yet
                                            if !has_rep_state {
                                                let event = Event::Session(SessionEvent::InitRepCounter);
                                                let core_ref = core_init_rep.borrow();
                                                let effects = core_ref.process_event(event);
                                                process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                            }
                                        })>
                                            "🔄 Rep Counter"
                                        </Button>
                                    </div>
                                }.into_any()
                            }}

                            // Controls
                            <div class="flex flex-wrap gap-3 justify-center">
                                {if is_last {
                                    view! {
                                        <Button variant=ButtonVariant::Primary on_click=Callback::new(move |_| {
                                            let now = chrono::Utc::now();
                                            let event = Event::Session(SessionEvent::FinishSession { now });
                                            let core_ref = core_finish.borrow();
                                            let effects = core_ref.process_event(event);
                                            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                            elapsed_secs.set(0);
                                        })>
                                            "Finish Session"
                                        </Button>
                                    }.into_any()
                                } else {
                                    view! {
                                        <Button variant=ButtonVariant::Primary on_click=Callback::new(move |_| {
                                            let now = chrono::Utc::now();
                                            let event = Event::Session(SessionEvent::NextItem { now });
                                            let core_ref = core_next.borrow();
                                            let effects = core_ref.process_event(event);
                                            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                            elapsed_secs.set(0);
                                        })>
                                            "Next Item"
                                        </Button>
                                    }.into_any()
                                }}
                                <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| {
                                    let now = chrono::Utc::now();
                                    let event = Event::Session(SessionEvent::SkipItem { now });
                                    let core_ref = core_skip.borrow();
                                    let effects = core_ref.process_event(event);
                                    process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                    elapsed_secs.set(0);
                                })>
                                    "Skip"
                                </Button>
                                <Button variant=ButtonVariant::DangerOutline on_click=Callback::new(move |_| {
                                    let now = chrono::Utc::now();
                                    let event = Event::Session(SessionEvent::EndSessionEarly { now });
                                    let core_ref = core_end.borrow();
                                    let effects = core_ref.process_event(event);
                                    process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                    elapsed_secs.set(0);
                                })>
                                    "End Early"
                                </Button>
                            </div>

                            // Completed items
                            {if !completed_entries.is_empty() {
                                Some(view! {
                                    <div class="mt-4">
                                        <h4 class="card-title">"Completed"</h4>
                                        <div class="space-y-1">
                                            {completed_entries.into_iter().map(|entry| {
                                                view! {
                                                    <SetlistEntryRow entry=entry show_controls=false />
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    </div>
                                })
                            } else {
                                None
                            }}
                        }.into_any()
                    }
                    None => {
                        view! {
                            <p class="text-sm text-muted text-center py-8">"No active session."</p>
                        }.into_any()
                    }
                }
            }}
        </div>
    }
}
