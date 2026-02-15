use leptos::prelude::*;

use intrada_core::{Event, SessionEvent, ViewModel};

use crate::components::{Button, ButtonVariant, Card};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::SharedCore;

/// End-of-session summary component: shows results, allows notes, save/discard.
#[component]
pub fn SessionSummary() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();

    let session_notes = RwSignal::new(String::new());

    let core_save = core.clone();
    let core_discard = core.clone();
    let core_entries = core.clone();
    let core_session_notes_outer = core;

    view! {
        <div class="space-y-6">
            {move || {
                let vm = view_model.get();
                match vm.summary {
                    Some(ref summary) => {
                        let core_save = core_save.clone();
                        let core_discard = core_discard.clone();
                        let core_entries = core_entries.clone();
                        let core_session_notes = core_session_notes_outer.clone();
                        let total_duration = summary.total_duration_display.clone();
                        let completion_status = summary.completion_status.clone();
                        let entries = summary.entries.clone();

                        view! {
                            // Summary header
                            <Card>
                                <div class="text-center space-y-2">
                                    <h2 class="text-2xl font-bold text-white">"Session Complete!"</h2>
                                    <p class="text-lg text-gray-300">
                                        {format!("Total: {}", total_duration)}
                                    </p>
                                    {if completion_status == "ended_early" {
                                        Some(view! {
                                            <span class="inline-flex items-center rounded-md bg-amber-500/20 px-2 py-0.5 text-xs font-medium text-amber-300 ring-1 ring-amber-400/20 ring-inset">
                                                "Ended Early"
                                            </span>
                                        })
                                    } else {
                                        None
                                    }}
                                </div>
                            </Card>

                            // Entries breakdown
                            <Card>
                                <h3 class="text-lg font-semibold text-white mb-3">"Items Practiced"</h3>
                                <div class="space-y-3">
                                    {entries.into_iter().map(|entry| {
                                        let entry_id = entry.id.clone();
                                        let entry_notes = RwSignal::new(entry.notes.clone().unwrap_or_default());
                                        let core_notes = core_entries.clone();
                                        let status_label = match entry.status.as_str() {
                                            "completed" => "✓",
                                            "skipped" => "⊘",
                                            _ => "—",
                                        };
                                        let status_color = match entry.status.as_str() {
                                            "completed" => "text-green-400",
                                            "skipped" => "text-amber-400",
                                            _ => "text-gray-500",
                                        };
                                        view! {
                                            <div class="rounded-lg border border-white/10 p-3 space-y-2">
                                                <div class="flex items-center justify-between">
                                                    <div class="flex items-center gap-2">
                                                        <span class={format!("text-sm font-medium {}", status_color)}>{status_label}</span>
                                                        <span class="text-sm font-medium text-white">{entry.item_title}</span>
                                                        <span class="text-xs text-gray-500">{entry.item_type}</span>
                                                    </div>
                                                    <span class="text-sm text-gray-400">{entry.duration_display}</span>
                                                </div>
                                                <div>
                                                    <input
                                                        type="text"
                                                        placeholder="Add notes for this item..."
                                                        class="w-full rounded border border-white/20 bg-white/10 text-white placeholder-gray-400 px-2 py-1 text-sm focus:border-indigo-400 focus:ring-1 focus:ring-indigo-400"
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
                                                            process_effects(&core_ref, effects, &view_model);
                                                        }
                                                    />
                                                </div>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            </Card>

                            // Session notes
                            <Card>
                                <h3 class="text-lg font-semibold text-white mb-3">"Session Notes"</h3>
                                <textarea
                                    rows="3"
                                    placeholder="How did this session go?"
                                    class="w-full rounded-lg border border-white/20 bg-white/10 text-white placeholder-gray-400 px-3 py-2 text-sm focus:border-indigo-400 focus:ring-1 focus:ring-indigo-400"
                                    bind:value=session_notes
                                    on:blur=move |_| {
                                        let notes_val = session_notes.get_untracked();
                                        let notes = if notes_val.is_empty() { None } else { Some(notes_val) };
                                        let event = Event::Session(SessionEvent::UpdateSessionNotes { notes });
                                        let core_ref = core_session_notes.borrow();
                                        let effects = core_ref.process_event(event);
                                        process_effects(&core_ref, effects, &view_model);
                                    }
                                />
                            </Card>

                            // Actions
                            <div class="flex gap-3">
                                <Button variant=ButtonVariant::Primary on_click=Callback::new(move |_| {
                                    let now = chrono::Utc::now();
                                    let event = Event::Session(SessionEvent::SaveSession { now });
                                    let core_ref = core_save.borrow();
                                    let effects = core_ref.process_event(event);
                                    process_effects(&core_ref, effects, &view_model);
                                })>
                                    "Save Session"
                                </Button>
                                <Button variant=ButtonVariant::DangerOutline on_click=Callback::new(move |_| {
                                    let event = Event::Session(SessionEvent::DiscardSession);
                                    let core_ref = core_discard.borrow();
                                    let effects = core_ref.process_event(event);
                                    process_effects(&core_ref, effects, &view_model);
                                })>
                                    "Discard"
                                </Button>
                            </div>

                            // Error display
                            {move || {
                                let vm = view_model.get();
                                vm.error.map(|err| {
                                    view! {
                                        <p class="text-sm text-red-400">{err}</p>
                                    }
                                })
                            }}
                        }.into_any()
                    }
                    None => {
                        view! {
                            <p class="text-sm text-gray-400 text-center py-8">"No session summary available."</p>
                        }.into_any()
                    }
                }
            }}
        </div>
    }
}
