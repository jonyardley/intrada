use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use intrada_core::{EntryStatus, Event, ItemKind, SessionEvent, ViewModel};

use crate::app::FocusMode;
use crate::components::{
    Button, ButtonSize, ButtonVariant, Card, Icon, IconName, InlineTypeIndicator, ProgressRing,
    SetlistEntryRow, TransitionPrompt,
};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, ItemType, SharedCore};

/// Map an `ItemKind` from core into the `ItemType` enum used by
/// `<InlineTypeIndicator>` (the two enums are duplicated for FFI/typegen
/// reasons; see `crates/intrada-web/src/types.rs`).
fn item_kind_to_type(kind: ItemKind) -> ItemType {
    match kind {
        ItemKind::Piece => ItemType::Piece,
        ItemKind::Exercise => ItemType::Exercise,
    }
}

/// Active session timer: shows current item, elapsed time, progress, and controls.
#[component]
pub fn SessionTimer() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let focus_mode = expect_context::<FocusMode>();

    let elapsed_secs = RwSignal::new(0u32);
    let interval_id: RwSignal<Option<i32>> = RwSignal::new(None);
    // UI-only visibility signal for the rep counter (does not affect domain state)
    let rep_counter_visible = RwSignal::new(false);
    // Tracks whether the current item's planned duration has elapsed (drives TransitionPrompt)
    let duration_elapsed = RwSignal::new(false);

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
                            .filter(|e| e.status == EntryStatus::Completed || e.status == EntryStatus::Skipped)
                            .cloned()
                            .collect();

                        // Planned duration for the current item (drives ProgressRing + TransitionPrompt)
                        let planned_duration = active.current_planned_duration_secs;
                        let next_item_title = active.next_item_title.clone();

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

                        let in_focus = focus_mode.get();

                        view! {
                            // Session intention (above the current item card) — hidden in focus mode
                            {if !in_focus {
                                session_intention.map(|intention| view! {
                                    <p class="text-sm text-secondary text-center italic">{intention}</p>
                                })
                            } else {
                                None
                            }}

                            // Current item — hero block. No Card chrome
                            // here: 2026 refresh leans on type + scale to
                            // anchor the screen rather than a glass surface.
                            <div class="text-center space-y-3 py-2">
                                <p class="text-xs text-muted uppercase tracking-wider">
                                    {format!("Item {} of {}", position + 1, total)}
                                </p>
                                <h2 class="text-2xl font-bold text-primary font-heading">{current_title}</h2>
                                // Entry-level intention (below the item title) — hidden in focus mode
                                {if !in_focus {
                                    current_entry_intention.map(|intention| view! {
                                        <p class="text-sm text-muted">{intention}</p>
                                    })
                                } else {
                                    None
                                }}
                                <div class="flex justify-center">
                                    <InlineTypeIndicator item_type=item_kind_to_type(current_type) />
                                </div>
                                // Timer: progress ring when planned duration exists,
                                // bare digital otherwise. The digital variant uses Inter
                                // weight 300 (light) at 48px/56px — the elegant practice-
                                // timer look from the Pencil reference rather than the
                                // alarm-clock font-mono bold of the previous design.
                                {match planned_duration {
                                    Some(planned_secs) => view! {
                                        <div class="mt-4">
                                            <ProgressRing
                                                elapsed_secs=elapsed_secs
                                                planned_duration_secs=planned_secs
                                            />
                                        </div>
                                    }.into_any(),
                                    None => view! {
                                        <p class="mt-4 text-5xl sm:text-6xl font-light tracking-tight text-primary tabular-nums">
                                            {move || {
                                                let secs = elapsed_secs.get();
                                                format!("{:02}:{:02}", secs / 60, secs % 60)
                                            }}
                                        </p>
                                    }.into_any(),
                                }}
                            </div>

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
                                            <span class="inline-flex items-center gap-1.5">
                                                <Icon name=IconName::RotateCcw class="w-4 h-4" />
                                                "Rep Counter"
                                            </span>
                                        </Button>
                                    </div>
                                }.into_any()
                            }}

                            // Transition prompt — shown when planned duration has elapsed
                            {move || {
                                if let Some(planned) = planned_duration {
                                    let elapsed = elapsed_secs.get();
                                    if elapsed >= planned {
                                        if !duration_elapsed.get_untracked() {
                                            duration_elapsed.set(true);
                                        }
                                        return Some(view! {
                                            <TransitionPrompt next_item_title=next_item_title.clone() />
                                        });
                                    }
                                }
                                None
                            }}

                            // Controls — primary action (Next / Finish) is
                            // a full-width hero CTA matching the Pencil
                            // reference. Skip + End Early stay as secondary
                            // / destructive sized buttons in a row beneath.
                            <div class="space-y-3">
                                {if is_last {
                                    view! {
                                        <Button
                                            variant=ButtonVariant::Primary
                                            size=ButtonSize::Hero
                                            attr:class="w-full"
                                            on_click=Callback::new(move |_| {
                                                let now = chrono::Utc::now();
                                                let event = Event::Session(SessionEvent::FinishSession { now });
                                                let core_ref = core_finish.borrow();
                                                let effects = core_ref.process_event(event);
                                                process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                                elapsed_secs.set(0);
                                                duration_elapsed.set(false);
                                            })
                                        >
                                            "Finish Session"
                                        </Button>
                                    }.into_any()
                                } else {
                                    view! {
                                        <Button
                                            variant=ButtonVariant::Primary
                                            size=ButtonSize::Hero
                                            attr:class="w-full"
                                            on_click=Callback::new(move |_| {
                                                let now = chrono::Utc::now();
                                                let event = Event::Session(SessionEvent::NextItem { now });
                                                let core_ref = core_next.borrow();
                                                let effects = core_ref.process_event(event);
                                                process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                                elapsed_secs.set(0);
                                                duration_elapsed.set(false);
                                            })
                                        >
                                            "Next Item"
                                        </Button>
                                    }.into_any()
                                }}
                                <div class="flex flex-wrap gap-3 justify-center">
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
                            </div>

                            // Focus mode toggle — reveals/hides nav, intentions, completed items
                            <div class="text-center">
                                <button
                                    class="inline-flex items-center gap-1 text-xs text-muted hover:text-secondary motion-safe:transition-colors"
                                    on:click=move |_| {
                                        focus_mode.set(!focus_mode.get_untracked());
                                    }
                                    aria-label=move || if in_focus { "Show more details" } else { "Return to focused view" }
                                >
                                    {if in_focus {
                                        view! {
                                            // Down chevron — "show more"
                                            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
                                                <path stroke-linecap="round" stroke-linejoin="round" d="m19.5 8.25-7.5 7.5-7.5-7.5" />
                                            </svg>
                                            <span>"Show more"</span>
                                        }.into_any()
                                    } else {
                                        view! {
                                            // Up chevron — "focus"
                                            <svg class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor">
                                                <path stroke-linecap="round" stroke-linejoin="round" d="m4.5 15.75 7.5-7.5 7.5 7.5" />
                                            </svg>
                                            <span>"Focus"</span>
                                        }.into_any()
                                    }}
                                </button>
                            </div>

                            // Completed items — hidden in focus mode
                            {if !in_focus && !completed_entries.is_empty() {
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
                            <p class="text-sm text-muted text-center py-8">"No session in progress."</p>
                        }.into_any()
                    }
                }
            }}
        </div>
    }
}
