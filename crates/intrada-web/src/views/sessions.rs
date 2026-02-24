use leptos::prelude::*;
use leptos_router::components::A;

use intrada_core::{Event, PracticeSessionView, SessionEvent, ViewModel};

use crate::components::{Button, ButtonVariant, Card, PageHeading};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// All-sessions list view showing every completed practice session.
#[component]
pub fn SessionsListView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();

    view! {
        <div>
            <PageHeading text="Practice Sessions" />

            {move || {
                if is_loading.get() {
                    return view! {
                        <div class="flex justify-center py-12">
                            <div class="animate-spin rounded-full h-8 w-8 border-2 border-accent-focus border-t-transparent"></div>
                        </div>
                    }.into_any();
                }

                let vm = view_model.get();

                if vm.sessions.is_empty() {
                    view! {
                        <div class="text-center py-12 px-4 sm:px-6 lg:px-0">
                            <p class="text-muted">"No practice sessions recorded yet."</p>
                            <p class="text-sm text-faint mt-2">"Start a practice session to begin tracking your progress."</p>
                            <div class="mt-6">
                                <A href="/sessions/new" attr:class="cta-link">
                                    "New Session"
                                </A>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    let core = core.clone();
                    view! {
                        <div class="mb-4">
                            <A href="/sessions/new" attr:class="cta-link">
                                "New Session"
                            </A>
                        </div>
                        <div class="space-y-3">
                            {vm.sessions.iter().map(|session| {
                                view! {
                                    <SessionRow
                                        session=session.clone()
                                        core=core.clone()
                                        view_model=view_model
                                    />
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                        <p class="text-sm text-muted mt-4">
                            {format!("{} session{}", vm.sessions.len(), if vm.sessions.len() == 1 { "" } else { "s" })}
                        </p>
                    }.into_any()
                }
            }}
        </div>
    }
}

/// A completed session row with summary info and delete action.
#[component]
fn SessionRow(
    session: PracticeSessionView,
    core: SharedCore,
    view_model: RwSignal<ViewModel>,
) -> impl IntoView {
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let confirm_delete = RwSignal::new(false);

    let id_for_delete = session.id.clone();
    let started_at = session.started_at.clone();
    let total_duration = session.total_duration_display.clone();
    let completion_status = session.completion_status.clone();
    let session_notes = session.notes.clone();
    let session_intention = session.session_intention.clone();
    let entry_count = session.entries.len();
    let entries = session.entries.clone();

    view! {
        <Card>
            {move || {
                if confirm_delete.get() {
                    let core_del = core.clone();
                    let id_del = id_for_delete.clone();
                    view! {
                        <div>
                            <p class="text-sm text-danger-text mb-3">"Delete this session? This cannot be undone."</p>
                            <div class="flex gap-2">
                                <Button
                                    variant=ButtonVariant::Danger
                                    loading=Signal::derive(move || is_submitting.get())
                                    on_click=Callback::new(move |_| {
                                        let event = Event::Session(SessionEvent::DeleteSession { id: id_del.clone() });
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
                    let started_at = started_at.clone();
                    let total_duration = total_duration.clone();
                    let completion_status = completion_status.clone();
                    let session_notes = session_notes.clone();
                    let session_intention = session_intention.clone();
                    let entries = entries.clone();
                    view! {
                        <div class="space-y-3">
                            <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
                                <div class="flex-1 min-w-0">
                                    <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1">
                                        <span class="text-sm font-medium text-primary">
                                            {total_duration}
                                        </span>
                                        <span class="text-xs text-muted">
                                            {format!("{} item{}", entry_count, if entry_count == 1 { "" } else { "s" })}
                                        </span>
                                        {if completion_status == "ended_early" {
                                            Some(view! {
                                                <span class="inline-flex items-center rounded-md bg-warning-surface px-2 py-0.5 text-xs font-medium text-warning-text ring-1 ring-warning/20 ring-inset">
                                                    "Ended Early"
                                                </span>
                                            })
                                        } else {
                                            None
                                        }}
                                        <span class="text-xs text-faint">{started_at}</span>
                                    </div>
                                    {session_intention.map(|intention| {
                                        view! {
                                            <p class="text-xs text-muted italic mt-1">{intention}</p>
                                        }
                                    })}
                                    {session_notes.map(|n| {
                                        view! {
                                            <p class="text-sm text-secondary mt-1">{n}</p>
                                        }
                                    })}
                                </div>
                                <div class="flex gap-2 sm:ml-4">
                                    <button
                                        class="text-xs text-danger-text hover:text-danger-hover font-medium"
                                        on:click=move |_| { confirm_delete.set(true); }
                                    >
                                        "Delete"
                                    </button>
                                </div>
                            </div>
                            // Entry details with scores
                            <div class="mt-1 pt-2 space-y-1.5">
                                {entries.into_iter().map(|entry| {
                                    let status_label = match entry.status.as_str() {
                                        "completed" => "✓",
                                        "skipped" => "⊘",
                                        _ => "—",
                                    };
                                    let status_color = match entry.status.as_str() {
                                        "completed" => "text-success-text",
                                        "skipped" => "text-warning-text",
                                        _ => "text-faint",
                                    };
                                    let entry_intention = entry.intention.clone();
                                    let entry_rep_target = entry.rep_target;
                                    let entry_rep_count = entry.rep_count;
                                    let entry_rep_reached = entry.rep_target_reached.unwrap_or(false);
                                    let entry_achieved_tempo = entry.achieved_tempo;
                                    view! {
                                        <div class="text-xs">
                                            <div class="flex items-center justify-between">
                                                <div class="flex items-center gap-2 min-w-0">
                                                    <span class={format!("font-medium {}", status_color)}>{status_label}</span>
                                                    <span class="text-primary truncate">{entry.item_title}</span>
                                                    <span class="text-faint shrink-0">{entry.duration_display}</span>
                                                </div>
                                                <div class="flex items-center gap-2 shrink-0 ml-2">
                                                    {entry_rep_target.map(|target| {
                                                        let count = entry_rep_count.unwrap_or(0);
                                                        let (color, bg) = if entry_rep_reached {
                                                            ("text-warm-accent-text", "bg-warm-accent-surface")
                                                        } else {
                                                            ("text-muted", "bg-surface-secondary")
                                                        };
                                                        view! {
                                                            <span class={format!("inline-flex items-center rounded-md px-1.5 py-0.5 text-xs font-mono {} {}", color, bg)}>
                                                                {format!("{}/{}", count, target)}
                                                            </span>
                                                        }
                                                    })}
                                                    {entry_achieved_tempo.map(|tempo| {
                                                        view! {
                                                            <span class="inline-flex items-center rounded-md bg-surface-secondary px-1.5 py-0.5 text-xs font-medium text-muted">
                                                                {format!("\u{266A} {} BPM", tempo)}
                                                            </span>
                                                        }
                                                    })}
                                                    {entry.score.map(|s| {
                                                        view! {
                                                            <span class="inline-flex items-center rounded-md bg-badge-piece-bg px-1.5 py-0.5 text-xs font-medium text-accent-text ring-1 ring-accent-focus/20 ring-inset">
                                                                {format!("{}/5", s)}
                                                            </span>
                                                        }
                                                    })}
                                                    {entry.notes.map(|n| {
                                                        let title = n.clone();
                                                        view! {
                                                            <span class="text-muted truncate max-w-[120px]" title={title}>{n}</span>
                                                        }
                                                    })}
                                                </div>
                                            </div>
                                            {entry_intention.map(|intention| {
                                                view! {
                                                    <p class="text-muted italic ml-5 mt-0.5">{intention}</p>
                                                }
                                            })}
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
