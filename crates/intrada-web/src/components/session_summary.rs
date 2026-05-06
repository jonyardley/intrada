use leptos::prelude::*;

use intrada_core::{CompletionStatus, EntryStatus, Event, SessionEvent, SetEvent, ViewModel};
use intrada_web::validation::validate_achieved_tempo_input;

use crate::components::{
    AccentBar, Button, ButtonVariant, Icon, IconName, RatingChips, SetSaveForm, StatCard, StatTone,
};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// End-of-session summary component: shows results, allows notes, save/discard.
#[component]
pub fn SessionSummary() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    let session_notes = RwSignal::new(String::new());

    let core_save = core.clone();
    let core_discard = core.clone();
    let core_entries = core.clone();
    let core_score = core.clone();
    let core_tempo = core.clone();
    let core_set_save = core.clone();
    let core_session_notes_outer = core;

    view! {
        // pb-32 reserves space for the sticky .action-bar at the bottom so
        // the last section (Save-as-Set button) isn't occluded.
        <div class="space-y-6 pb-32">
            {move || {
                let vm = view_model.get();
                match vm.summary {
                    Some(ref summary) => {
                        let core_save = core_save.clone();
                        let core_discard = core_discard.clone();
                        let core_entries = core_entries.clone();
                        let core_score = core_score.clone();
                        let core_tempo = core_tempo.clone();
                        let core_session_notes = core_session_notes_outer.clone();
                        let total_duration = summary.total_duration_display.clone();
                        let completion_status = summary.completion_status.clone();
                        let session_intention = summary.session_intention.clone();
                        let entries = summary.entries.clone();

                        // Stat row figures: items practiced (any non-NotAttempted)
                        // and average confidence across scored entries.
                        let items_practiced = entries
                            .iter()
                            .filter(|e| e.status != EntryStatus::NotAttempted)
                            .count();
                        let scored: Vec<u8> = entries
                            .iter()
                            .filter_map(|e| e.score)
                            .collect();
                        let avg_score_display = if scored.is_empty() {
                            "—".to_string()
                        } else {
                            let sum: u32 = scored.iter().map(|s| *s as u32).sum();
                            let avg = sum as f64 / scored.len() as f64;
                            format!("{avg:.1}")
                        };

                        view! {
                            // Hero header — flat, no Card chrome (matches the
                            // builder / review-sheet vocabulary). page-title
                            // for the celebratory beat, subtitle for context.
                            <div class="text-center space-y-2">
                                <h2 class="page-title">"Session Complete"</h2>
                                <p class="text-sm text-secondary">
                                    "Great work! Review your session below."
                                </p>
                                {if completion_status == CompletionStatus::EndedEarly {
                                    Some(view! {
                                        <span class="badge badge--warning">"Ended Early"</span>
                                    })
                                } else {
                                    None
                                }}
                                {session_intention.map(|intention| view! {
                                    <p class="text-sm text-secondary italic">{intention}</p>
                                })}
                            </div>

                            // Stat row — Duration / Items Practiced / Avg
                            // Confidence. Uses the StatCard refresh variant
                            // with inset accent bars per Pencil.
                            <div class="grid grid-cols-3 gap-3">
                                <StatCard
                                    title="Duration"
                                    value=total_duration
                                    bar=AccentBar::Gold
                                />
                                <StatCard
                                    title="Items"
                                    value=items_practiced.to_string()
                                    bar=AccentBar::Blue
                                    tone=StatTone::Accent
                                />
                                <StatCard
                                    title="Avg"
                                    value=avg_score_display
                                    subtitle="out of 5"
                                    bar=AccentBar::Gold
                                    tone=StatTone::WarmAccent
                                />
                            </div>

                            // Items practiced — flat section with header,
                            // each entry is its own surface-secondary card.
                            <div class="space-y-3">
                                <h3 class="section-title">"Items Practiced"</h3>
                                    {entries.into_iter().map(|entry| {
                                        let entry_id = entry.id.clone();
                                        let entry_id_for_score = entry.id.clone();
                                        let entry_id_for_tempo = entry.id.clone();
                                        let entry_notes = RwSignal::new(entry.notes.clone().unwrap_or_default());
                                        let entry_tempo_str = RwSignal::new(
                                            entry.achieved_tempo.map(|t| t.to_string()).unwrap_or_default()
                                        );
                                        let tempo_error = RwSignal::new(Option::<String>::None);
                                        let core_notes = core_entries.clone();
                                        let core_score_inner = core_score.clone();
                                        let core_tempo_inner = core_tempo.clone();
                                        let is_completed = entry.status == EntryStatus::Completed;
                                        let current_score = entry.score;
                                        let entry_intention = entry.intention.clone();
                                        let entry_rep_target = entry.rep_target;
                                        let entry_rep_count = entry.rep_count;
                                        let entry_rep_reached = entry.rep_target_reached.unwrap_or(false);
                                        let notes_label = format!("Notes for {}", entry.item_title);
                                        let notes_input_id = format!("entry-notes-{}", entry.id);
                                        let (status_icon, status_color) = match entry.status {
                                            EntryStatus::Completed => (IconName::Check, "text-success-text"),
                                            EntryStatus::Skipped => (IconName::Ban, "text-warning-text"),
                                            EntryStatus::NotAttempted => (IconName::Minus, "text-faint"),
                                        };
                                        view! {
                                            <div class="rounded-lg bg-surface-secondary p-3 space-y-2">
                                                <div class="flex items-center justify-between">
                                                    <div class="flex items-center gap-2">
                                                        <span class={format!("text-sm font-medium {}", status_color)}>
                                                            <Icon name=status_icon class="w-4 h-4" />
                                                        </span>
                                                        <span class="text-sm font-medium text-primary">{entry.item_title}</span>
                                                        <span class="text-xs text-faint">{entry.item_type.to_string()}</span>
                                                    </div>
                                                    <span class="text-sm text-muted">{entry.duration_display}</span>
                                                </div>
                                                {entry_intention.map(|intention| view! {
                                                    <p class="text-xs text-muted italic">{intention}</p>
                                                })}
                                                {entry_rep_target.map(|target| {
                                                    let count = entry_rep_count.unwrap_or(0);
                                                    let color = if entry_rep_reached {
                                                        "text-warm-accent-text"
                                                    } else {
                                                        "text-muted"
                                                    };
                                                    let label = if entry_rep_reached {
                                                        format!("Reps: {} / {} ✓", count, target)
                                                    } else {
                                                        format!("Reps: {} / {}", count, target)
                                                    };
                                                    // Show attempt count when history is non-empty
                                                    let attempt_suffix = entry.rep_history.as_ref().and_then(|history| {
                                                        let attempts = history.len();
                                                        if attempts > 0 {
                                                            Some(format!(" · {} attempts", attempts))
                                                        } else {
                                                            None
                                                        }
                                                    });
                                                    view! {
                                                        <p class={format!("text-xs font-mono {}", color)}>
                                                            {label}
                                                            {attempt_suffix.map(|suffix| view! {
                                                                <span class="text-muted">{suffix}</span>
                                                            })}
                                                        </p>
                                                    }
                                                })}
                                                <div>
                                                    <label class="sr-only" for=notes_input_id.clone()>
                                                        {notes_label}
                                                    </label>
                                                    <input
                                                        type="text"
                                                        id=notes_input_id
                                                        placeholder="Add notes for this item..."
                                                        class="input-base"
                                                        bind:value=entry_notes
                                                        on:blur=move |_| {
                                                            let notes_val = entry_notes.get_untracked();
                                                            let notes = if notes_val.is_empty() { None } else { Some(notes_val) };
                                                            let event = Event::Session(SessionEvent::UpdateEntryNotes {
                                                                entry_id: entry_id.clone(),
                                                                notes,
                                                            });
                                                            let core_ref = core_notes.borrow();
                                                            let effects = core_ref.process_event(event);
                                                            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                                        }
                                                    />
                                                </div>
                                                // Score chips — only for completed entries.
                                                // Toggle + dispatch logic lives in the parent
                                                // because the source of truth is the view model
                                                // entry, not a local signal.
                                                {if is_completed {
                                                    let entry_id_score = entry_id_for_score.clone();
                                                    let core_score_btns = core_score_inner.clone();
                                                    let on_change = Callback::new(move |new_score: Option<u8>| {
                                                        let event = Event::Session(SessionEvent::UpdateEntryScore {
                                                            entry_id: entry_id_score.clone(),
                                                            score: new_score,
                                                        });
                                                        let core_ref = core_score_btns.borrow();
                                                        let effects = core_ref.process_event(event);
                                                        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                                    });
                                                    Some(view! {
                                                        <div class="flex items-center gap-2">
                                                            <span class="text-xs text-muted mr-1">"Confidence:"</span>
                                                            <RatingChips
                                                                selected=Signal::derive(move || current_score)
                                                                on_change=on_change
                                                                aria_label_prefix="Rate confidence"
                                                            />
                                                        </div>
                                                    })
                                                } else {
                                                    None
                                                }}
                                                // Achieved tempo input — only for completed entries
                                                {if is_completed {
                                                    let entry_id_tempo = entry_id_for_tempo.clone();
                                                    let tempo_input_id = format!("achieved-tempo-{}", entry_id_tempo);
                                                    Some(view! {
                                                        <div>
                                                            <label class="text-xs text-muted" for=tempo_input_id.clone()>
                                                                "Achieved tempo (BPM)"
                                                            </label>
                                                            <input
                                                                type="number"
                                                                inputmode="numeric"
                                                                id=tempo_input_id
                                                                placeholder="1\u{2013}500"
                                                                class="input-base mt-1"
                                                                bind:value=entry_tempo_str
                                                                on:blur=move |_| {
                                                                    let val = entry_tempo_str.get_untracked();
                                                                    // Validate client-side
                                                                    if let Some(err) = validate_achieved_tempo_input(&val) {
                                                                        tempo_error.set(Some(err));
                                                                        return;
                                                                    }
                                                                    tempo_error.set(None);
                                                                    let tempo = if val.trim().is_empty() {
                                                                        None
                                                                    } else {
                                                                        val.trim().parse::<u16>().ok()
                                                                    };
                                                                    let event = Event::Session(SessionEvent::UpdateEntryTempo {
                                                                        entry_id: entry_id_tempo.clone(),
                                                                        tempo,
                                                                    });
                                                                    let core_ref = core_tempo_inner.borrow();
                                                                    let effects = core_ref.process_event(event);
                                                                    process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                                                }
                                                            />
                                                            {move || {
                                                                tempo_error.get().map(|err| view! {
                                                                    <p class="text-xs text-danger-text mt-1">{err}</p>
                                                                })
                                                            }}
                                                        </div>
                                                    })
                                                } else {
                                                    None
                                                }}
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                            </div>

                            // Session Notes — flat, no Card chrome.
                            <div class="space-y-3">
                                <h3 class="section-title">"Session Notes"</h3>
                                <textarea
                                    rows="3"
                                    placeholder="How did this session go?"
                                    class="input-base"
                                    bind:value=session_notes
                                    on:blur=move |_| {
                                        let notes_val = session_notes.get_untracked();
                                        let notes = if notes_val.is_empty() { None } else { Some(notes_val) };
                                        let event = Event::Session(SessionEvent::UpdateSessionNotes { notes });
                                        let core_ref = core_session_notes.borrow();
                                        let effects = core_ref.process_event(event);
                                        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                    }
                                />
                            </div>

                            // Save as Set — kept in place for now;
                            // re-evaluate position when the sets flow
                            // is reworked (see #390).
                            {
                                let core_save_set = core_set_save.clone();
                                view! {
                                    <SetSaveForm on_save=Callback::new(move |name: String| {
                                        let event = Event::Set(SetEvent::SaveSummaryAsSet { name });
                                        let core_ref = core_save_set.borrow();
                                        let effects = core_ref.process_event(event);
                                        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                    }) />
                                }
                            }

                            // Error display
                            {move || {
                                let vm = view_model.get();
                                vm.error.map(|err| {
                                    view! {
                                        <p class="text-sm text-danger-text">{err}</p>
                                    }
                                })
                            }}

                            // Sticky bottom action bar — Save Session
                            // (primary, fills) + Discard (danger outline,
                            // sized to its label). Same chrome as the
                            // builder toolbar (.action-bar utility).
                            <div class="action-bar" role="toolbar" aria-label="Session actions">
                                <div class="flex-1">
                                    <Button
                                        variant=ButtonVariant::Primary
                                        full_width=true
                                        on_click=Callback::new(move |_| {
                                            let now = chrono::Utc::now();
                                            let event = Event::Session(SessionEvent::SaveSession { now });
                                            let core_ref = core_save.borrow();
                                            let effects = core_ref.process_event(event);
                                            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                        })
                                    >
                                        "Save Session"
                                    </Button>
                                </div>
                                <Button
                                    variant=ButtonVariant::DangerOutline
                                    on_click=Callback::new(move |_| {
                                        let event = Event::Session(SessionEvent::DiscardSession);
                                        let core_ref = core_discard.borrow();
                                        let effects = core_ref.process_event(event);
                                        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                    })
                                >
                                    "Discard"
                                </Button>
                            </div>
                        }.into_any()
                    }
                    None => {
                        view! {
                            <p class="text-sm text-muted text-center py-8">"No session summary available."</p>
                        }.into_any()
                    }
                }
            }}
        </div>
    }
}
