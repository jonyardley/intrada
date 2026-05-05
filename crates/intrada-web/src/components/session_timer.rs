use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use intrada_core::{EntryStatus, Event, ItemKind, SessionEvent, ViewModel};

use crate::app::FocusMode;
use crate::components::{
    Button, ButtonSize, ButtonVariant, Icon, IconName, InlineTypeIndicator, ItemReflectionSheet,
    ItemReflectionTarget, ProgressRing, SectionLabel, SetlistEntryRow, TransitionPrompt,
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
    // Position the user manually dismissed the rep counter at. Tied to
    // position rather than a plain bool so the counter naturally
    // reappears on the next item — fixes the prior bug where a sticky
    // auto-show effect re-revealed the counter immediately after dismiss.
    let rep_dismissed_at_position = RwSignal::new(Option::<usize>::None);
    // Tracks whether the current item's planned duration has elapsed (drives TransitionPrompt)
    let duration_elapsed = RwSignal::new(false);

    // Reflection sheet state — opened on Next / Finish tap, captures the
    // just-completed entry's score/tempo/notes before advancing.
    let reflection_open = RwSignal::new(false);
    let reflection_target = RwSignal::new(Option::<ItemReflectionTarget>::None);
    let reflection_next_title = RwSignal::new(Option::<String>::None);
    let reflection_next_type = RwSignal::new(Option::<ItemKind>::None);
    let reflection_position_label = RwSignal::new(String::new());

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
        // space-y-6 between major zones — hero, optional rep card,
        // primary CTA, sub-actions. The wider gap separates concerns
        // visually so the user can read the screen at a glance.
        <div class="space-y-6">
            {move || {
                let vm = view_model.get();
                match vm.active_session {
                    Some(ref active) => {
                        let core_advance = core.clone();
                        let core_skip = core.clone();
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
                        // Counter is visible when the entry carries rep state AND the
                        // user hasn't dismissed it for this position. The dismissed
                        // position resets on item change so the counter naturally
                        // returns on the next item.
                        let show_counter =
                            has_rep_state && rep_dismissed_at_position.get() != Some(position);

                        let in_focus = focus_mode.get();
                        let session_intention_class = if in_focus {
                            "focus-fade focus-fade--hidden"
                        } else {
                            "focus-fade"
                        };
                        let entry_intention_class = session_intention_class;
                        let completed_class = session_intention_class;

                        view! {
                            // Session intention (above the current item card) —
                            // fades + slides + collapses in focus mode rather
                            // than hard-cutting. Always rendered so the
                            // transition has something to animate.
                            {session_intention.map(|intention| view! {
                                <div class=session_intention_class>
                                    <p class="text-sm text-secondary text-center italic">{intention}</p>
                                </div>
                            })}

                            // Current item — hero block. No Card chrome
                            // here: 2026 refresh leans on type + scale to
                            // anchor the screen rather than a glass surface.
                            <div class="text-center space-y-3 py-2">
                                <p class="text-xs text-muted uppercase tracking-wider">
                                    {format!("Item {} of {}", position + 1, total)}
                                </p>
                                <h2 class="text-2xl font-bold text-primary font-heading">{current_title}</h2>
                                // Entry-level intention — fades with focus mode
                                {current_entry_intention.map(|intention| view! {
                                    <div class=entry_intention_class>
                                        <p class="text-sm text-muted">{intention}</p>
                                    </div>
                                })}
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

                            // Rep counter — its own contained card so it
                            // reads as a distinct contextual module sitting
                            // between the timer hero and the primary CTA.
                            // Header row carries SectionLabel + an X icon-
                            // nav-button (replacing the previous text-styled
                            // "Hide counter" link, which read like body
                            // copy).
                            {if show_counter {
                                let target = rep_target.unwrap_or(0);
                                let count = rep_count.unwrap_or(0);
                                let reached = rep_target_reached.unwrap_or(false);
                                let progress_pct = if target > 0 {
                                    ((count as f64 / target as f64) * 100.0).min(100.0)
                                } else {
                                    0.0
                                };
                                // "Target reached" celebrates with accent
                                // purple — the app's "you achieved
                                // something" colour (Day Streak uses it in
                                // Analytics). Warm-accent gold reads as
                                // warning in the iOS palette and didn't
                                // fit a positive completion moment.
                                let count_class = if reached {
                                    "text-4xl sm:text-5xl font-light tracking-tight tabular-nums text-accent-text"
                                } else {
                                    "text-4xl sm:text-5xl font-light tracking-tight tabular-nums text-primary"
                                };
                                let bar_fill_class = if reached {
                                    "h-full rounded-full bg-accent motion-safe:transition-all motion-safe:duration-300"
                                } else {
                                    "h-full rounded-full bg-success motion-safe:transition-all motion-safe:duration-300"
                                };

                                view! {
                                    <div class="rounded-xl bg-surface-secondary p-4 space-y-3">
                                        // Header row — label left, X close right
                                        <div class="flex items-center justify-between -mt-1 -mr-1">
                                            <span class="section-label" style="margin-bottom:0">"Consecutive Reps"</span>
                                            <button
                                                type="button"
                                                class="icon-nav-button"
                                                aria-label="Hide rep counter"
                                                on:click=move |_| {
                                                    rep_dismissed_at_position.set(Some(position));
                                                }
                                            >
                                                <Icon name=IconName::X class="w-4 h-4" />
                                            </button>
                                        </div>
                                        <div class="text-center space-y-2 pt-1">
                                            <p class=count_class>
                                                {format!("{} / {}", count, target)}
                                            </p>
                                            <div class="w-full h-1.5 rounded-full bg-progress-track overflow-hidden">
                                                <div
                                                    class=bar_fill_class
                                                    style=format!("width: {}%", progress_pct)
                                                />
                                            </div>
                                        </div>

                                        {if reached {
                                            view! {
                                                <p class="text-sm font-medium text-accent-text text-center">"Target reached"</p>
                                            }.into_any()
                                        } else {
                                            // Missed left (de-emphasised),
                                            // Got it right (primary success) —
                                            // iOS "destructive on left,
                                            // primary on right" idiom.
                                            view! {
                                                <div class="flex gap-3 justify-center">
                                                    <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| {
                                                        let event = Event::Session(SessionEvent::RepMissed);
                                                        let core_ref = core_missed.borrow();
                                                        let effects = core_ref.process_event(event);
                                                        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                                    })>
                                                        "Missed"
                                                    </Button>
                                                    <Button variant=ButtonVariant::Success on_click=Callback::new(move |_| {
                                                        let event = Event::Session(SessionEvent::RepGotIt);
                                                        let core_ref = core_got_it.borrow();
                                                        let effects = core_ref.process_event(event);
                                                        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                                    })>
                                                        "Got it"
                                                    </Button>
                                                </div>
                                            }.into_any()
                                        }}
                                    </div>
                                }.into_any()
                            } else {
                                // Counter hidden — show enable/show button
                                view! {
                                    <div class="text-center">
                                        <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| {
                                            // Re-show by clearing the dismissed
                                            // position. If no rep state exists,
                                            // dispatch InitRepCounter to seed it.
                                            rep_dismissed_at_position.set(None);
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
                            // reference. Tap opens the reflection sheet,
                            // which captures self-rating + tempo + notes
                            // before dispatching NextItem / FinishSession.
                            // Skip + End Early stay as secondary /
                            // destructive sized buttons in a row beneath.
                            <div class="space-y-3">
                                {
                                    let entries = active.entries.clone();
                                    let next_title_for_sheet = active.next_item_title.clone();
                                    let on_advance_tap = move || {
                                        // Snapshot the just-completed entry so the
                                        // reflection sheet can pre-populate (and
                                        // dispatch Update* events against the right
                                        // entry id even after Next/Finish lands).
                                        //
                                        // Order: set `target` BEFORE flipping `open`.
                                        // The sheet's seed effect runs on target
                                        // change — flipping open first would let it
                                        // see the previous item's target on first
                                        // render.
                                        if let Some(entry) = entries.get(position) {
                                            reflection_target.set(Some(ItemReflectionTarget {
                                                entry_id: entry.id.clone(),
                                                initial_score: entry.score,
                                                initial_tempo: entry.achieved_tempo,
                                                initial_notes: entry.notes.clone(),
                                            }));
                                        }
                                        reflection_next_title.set(next_title_for_sheet.clone());
                                        // Type of the next item — drives the badge in
                                        // the sheet header. None on the last item.
                                        reflection_next_type.set(
                                            entries.get(position + 1).map(|e| e.item_type.clone())
                                        );
                                        reflection_position_label.set(
                                            format!("Item {} of {}", position + 1, total)
                                        );
                                        reflection_open.set(true);
                                    };
                                    let label = if is_last { "Finish Session" } else { "Next Item" };
                                    view! {
                                        <Button
                                            variant=ButtonVariant::Primary
                                            size=ButtonSize::Hero
                                            full_width=true
                                            on_click=Callback::new(move |_| on_advance_tap())
                                        >
                                            {label}
                                        </Button>
                                    }.into_any()
                                }
                                // Sub-actions — proper Button components
                                // (44px standard size). They're clearly
                                // subordinate to the hero CTA above by
                                // virtue of the hero's heavier presence
                                // (52px / 17px / drop shadow), not by
                                // shrinking these to look like text. End
                                // Early on the leading edge per iOS
                                // convention.
                                <div class="flex items-center justify-center gap-3">
                                    <Button
                                        variant=ButtonVariant::DangerOutline
                                        on_click=Callback::new(move |_| {
                                            let now = chrono::Utc::now();
                                            let event = Event::Session(SessionEvent::EndSessionEarly { now });
                                            let core_ref = core_end.borrow();
                                            let effects = core_ref.process_event(event);
                                            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                            elapsed_secs.set(0);
                                        })
                                    >
                                        "End Early"
                                    </Button>
                                    <Button
                                        variant=ButtonVariant::Secondary
                                        on_click=Callback::new(move |_| {
                                            let now = chrono::Utc::now();
                                            let event = Event::Session(SessionEvent::SkipItem { now });
                                            let core_ref = core_skip.borrow();
                                            let effects = core_ref.process_event(event);
                                            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                            elapsed_secs.set(0);
                                        })
                                    >
                                        "Skip"
                                    </Button>
                                </div>
                            </div>

                            // Completed items — fades + collapses in focus
                            // mode rather than hard-cutting. SectionLabel
                            // matches the rest of the 2026 refresh
                            // grouped-content language.
                            {(!completed_entries.is_empty()).then(|| view! {
                                <div class=completed_class>
                                    <div class="mt-4">
                                        <SectionLabel text="Completed" />
                                        <div class="space-y-1">
                                            {completed_entries.into_iter().map(|entry| {
                                                view! {
                                                    <SetlistEntryRow
                                                        id=entry.id.clone()
                                                        item_title=entry.item_title.clone()
                                                        item_type=entry.item_type.clone()
                                                        duration_display=entry.duration_display.clone()
                                                        position=entry.position
                                                        show_controls=false
                                                    />
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    </div>
                                </div>
                            })}

                            // Reflection sheet — opened by Next/Finish tap.
                            // On Continue: dispatches Update* events for the
                            // just-completed entry, then advances the
                            // session. On Skip: just advances. is_last is
                            // re-read from the view model at advance time so
                            // the dispatch matches the actual session state.
                            {
                                let on_advance = Callback::new(move |_| {
                                    let now = chrono::Utc::now();
                                    let vm = view_model.get_untracked();
                                    let is_last_now = vm.active_session.as_ref().is_some_and(|a| {
                                        a.current_position == a.total_items.saturating_sub(1)
                                    });
                                    let event = if is_last_now {
                                        Event::Session(SessionEvent::FinishSession { now })
                                    } else {
                                        Event::Session(SessionEvent::NextItem { now })
                                    };
                                    let core_ref = core_advance.borrow();
                                    let effects = core_ref.process_event(event);
                                    process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                    elapsed_secs.set(0);
                                    duration_elapsed.set(false);
                                });
                                view! {
                                    <ItemReflectionSheet
                                        open=reflection_open
                                        next_item_title=Signal::derive(move || reflection_next_title.get())
                                        next_item_type=Signal::derive(move || reflection_next_type.get())
                                        target=Signal::derive(move || reflection_target.get())
                                        position_label=Signal::derive(move || reflection_position_label.get())
                                        on_advance=on_advance
                                    />
                                }
                            }
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
