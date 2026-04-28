use std::collections::HashSet;

use chrono::NaiveDate;
use leptos::prelude::*;
use leptos_router::components::A;

use intrada_core::{
    CompletionStatus, EntryStatus, Event, PracticeSessionView, SessionEvent, ViewModel,
};

use crate::components::{
    Button, ButtonVariant, Card, Icon, IconName, PageHeading, SkeletonCardList, WeekStrip,
};
use intrada_web::core_bridge::process_effects;
use intrada_web::helpers::{
    auto_select_day, format_time_short, get_week_start, group_sessions_by_date, sessions_for_week,
};
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Practice page with week strip navigator.
///
/// Shows a weekly calendar strip with Mon–Sun and dot indicators for days
/// that have practices. Tapping a day shows that day's practice cards below.
#[component]
pub fn SessionsListView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();

    // Ephemeral UI state — week offset and selected date (Leptos signals per architecture rules)
    let week_offset = RwSignal::new(0_i32);
    let selected_date = RwSignal::new(None::<NaiveDate>);

    // Today's date — computed once on mount
    let today = chrono::Utc::now().date_naive();

    // Derived: week start from offset
    let week_start = Signal::derive(move || get_week_start(today, week_offset.get()));

    // Derived: group all sessions by date
    let grouped_sessions = Signal::derive(move || {
        let vm = view_model.get();
        group_sessions_by_date(&vm.sessions)
    });

    // Derived: which dates in the current week have sessions
    let session_dates_in_week =
        Signal::derive(move || sessions_for_week(&grouped_sessions.get(), week_start.get()));

    // Auto-select effect: when selected_date is None (initial load or week change),
    // pick the best day based on auto-select logic.
    Effect::new(move || {
        let ws = week_start.get();
        let dates = session_dates_in_week.get();
        if selected_date.get_untracked().is_none() {
            let best = auto_select_day(ws, today, &dates);
            selected_date.set(Some(best));
        }
    });

    // Derived: sessions for the selected day, sorted chronologically
    let day_sessions = Signal::derive(move || -> Vec<PracticeSessionView> {
        let sel = selected_date.get();
        let grouped = grouped_sessions.get();
        match sel {
            Some(date) => grouped.get(&date).cloned().unwrap_or_default(),
            None => Vec::new(),
        }
    });

    // Callbacks for WeekStrip
    let on_day_click = Callback::new(move |date: NaiveDate| {
        selected_date.set(Some(date));
    });

    let on_prev_week = Callback::new(move |()| {
        week_offset.update(|o| *o -= 1);
        selected_date.set(None); // triggers auto-select
    });

    let on_next_week = Callback::new(move |()| {
        week_offset.update(|o| *o += 1);
        selected_date.set(None); // triggers auto-select
    });

    let on_today = Callback::new(move |()| {
        week_offset.set(0);
        selected_date.set(Some(today));
    });

    view! {
        <div>
            // PageHeading owns the title-row layout — title on left, the
            // "New Session" trailing action on right, subtitle below
            // both. The cta-link's icon/label children are CSS-swapped
            // per platform: web shows the "New Session" pill, iOS shows
            // the "+" icon-only nav action.
            <PageHeading
                text="Practice"
                subtitle="Review your session history and track how your sessions build over time."
                trailing=Box::new(|| view! {
                    <A
                        href="/sessions/new"
                        attr:class="cta-link cta-link--page-add shrink-0"
                        attr:aria-label="New Session"
                    >
                        <Icon name=IconName::Plus class="cta-link-icon" />
                        <span class="cta-link-label">"New Session"</span>
                    </A>
                }.into_any())
            />

            // Week strip navigator
            <div class="mb-6">
                {move || {
                    let ws = week_start.get();
                    let sel = selected_date.get();
                    let dates: HashSet<NaiveDate> = session_dates_in_week.get();
                    let is_current = week_offset.get() == 0;
                    view! {
                        <WeekStrip
                            week_start=ws
                            selected_date=sel
                            session_dates=dates
                            on_day_click=on_day_click
                            on_prev_week=on_prev_week
                            on_next_week=on_next_week
                            on_today=on_today
                            is_current_week=is_current
                        />
                    }
                }}
            </div>

            // Session cards for selected day
            {move || {
                if is_loading.get() {
                    return view! {
                        <SkeletonCardList count=3 />
                    }.into_any();
                }

                let sessions = day_sessions.get();

                if sessions.is_empty() {
                    view! {
                        <p class="empty-text">"No sessions on this day"</p>
                    }.into_any()
                } else {
                    let core = core.clone();
                    let session_count = sessions.len();
                    view! {
                        <div class="space-y-3">
                            {sessions.into_iter().map(|session| {
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
                            {format!("{} session{}", session_count, if session_count == 1 { "" } else { "s" })}
                        </p>
                    }.into_any()
                }
            }}

            // "Show all sessions" link
            <div class="mt-6 text-center">
                <A href="/sessions/all" attr:class="action-link text-muted hover:text-accent-text">
                    "Show all sessions →"
                </A>
            </div>
        </div>
    }
}

/// A completed session row with summary info and delete action.
#[component]
pub(crate) fn SessionRow(
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
                                        {if completion_status == CompletionStatus::EndedEarly {
                                            Some(view! {
                                                <span class="inline-flex items-center rounded-md bg-warning-surface px-2 py-0.5 text-xs font-medium text-warning-text ring-1 ring-warning/20 ring-inset">
                                                    "Ended Early"
                                                </span>
                                            })
                                        } else {
                                            None
                                        }}
                                        <span class="text-xs text-faint">{format_time_short(&started_at)}</span>
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
                                    let (status_icon, status_color) = match entry.status {
                                        EntryStatus::Completed => (IconName::Check, "text-success-text"),
                                        EntryStatus::Skipped => (IconName::Ban, "text-warning-text"),
                                        EntryStatus::NotAttempted => (IconName::Minus, "text-faint"),
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
                                                    <span class={format!("font-medium {}", status_color)}>
                                                        <Icon name=status_icon class="w-3.5 h-3.5" />
                                                    </span>
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
