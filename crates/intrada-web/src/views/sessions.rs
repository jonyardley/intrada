use std::collections::HashSet;

use chrono::NaiveDate;
use leptos::prelude::*;
use leptos_router::components::A;

use intrada_core::{
    CompletionStatus, EntryStatus, Event, PracticeSessionView, SessionEvent, ViewModel,
};

use crate::components::{
    ContextMenu, ContextMenuAction, EmptyState, GroupedList, GroupedListRow, Icon, IconName,
    PageHeading, SkeletonCardList, SwipeActions, WeekStrip,
};
use intrada_web::core_bridge::process_effects_with_core;
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

    // Derived: which dates in the current week have sessions (used for auto-select)
    let session_dates_in_week =
        Signal::derive(move || sessions_for_week(&grouped_sessions.get(), week_start.get()));

    // Derived: session dates across prev/current/next weeks. The week strip
    // renders all 3 pages side-by-side for the iOS Calendar peek gesture, so
    // it needs practice dots for the adjacent weeks too — otherwise dots
    // would only pop in after the snap completes.
    let session_dates_three_weeks = Signal::derive(move || {
        let grouped = grouped_sessions.get();
        let ws = week_start.get();
        let prev = sessions_for_week(&grouped, ws - chrono::Duration::days(7));
        let curr = sessions_for_week(&grouped, ws);
        let next = sessions_for_week(&grouped, ws + chrono::Duration::days(7));
        let mut all = HashSet::new();
        all.extend(prev);
        all.extend(curr);
        all.extend(next);
        all
    });

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

    // When navigating weeks via swipe or chevron, keep the selected
    // day-of-week (e.g. Mon → Mon of the new week) so the highlight stays
    // visually anchored — matches iOS Calendar and avoids a "selection
    // pop" right after the swipe completes.
    let on_prev_week = Callback::new(move |()| {
        week_offset.update(|o| *o -= 1);
        if let Some(sel) = selected_date.get_untracked() {
            selected_date.set(Some(sel - chrono::Duration::days(7)));
        }
    });

    let on_next_week = Callback::new(move |()| {
        week_offset.update(|o| *o += 1);
        if let Some(sel) = selected_date.get_untracked() {
            selected_date.set(Some(sel + chrono::Duration::days(7)));
        }
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

            // Week strip navigator. Pass signals (not values) so WeekStrip
            // stays mounted across week changes — its internal track
            // animation depends on persistent component state, and a remount
            // on every navigation would make the post-snap reset to centred
            // visibly bounce (see week_strip.rs for the full picture).
            <div class="mb-6">
                <WeekStrip
                    week_start=week_start
                    selected_date=selected_date.into()
                    session_dates=session_dates_three_weeks
                    on_day_click=on_day_click
                    on_prev_week=on_prev_week
                    on_next_week=on_next_week
                    on_today=on_today
                    is_current_week=Signal::derive(move || week_offset.get() == 0)
                />
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
                        <EmptyState
                            icon=IconName::CalendarDays
                            title="No sessions on this day"
                            body="Start a practice session to see it here."
                        >
                            <A href="/sessions/new" attr:class="cta-link">
                                "New Session"
                            </A>
                        </EmptyState>
                    }.into_any()
                } else {
                    let core = core.clone();
                    let session_count = sessions.len();
                    view! {
                        <GroupedList aria_label="Practice sessions">
                            {sessions.into_iter().map(|session| {
                                // Per-row clone — the GroupedListRow's
                                // Children closure is FnOnce and would
                                // otherwise try to move `core` out of the
                                // surrounding FnMut map closure.
                                let core_for_row = core.clone();
                                view! {
                                    <GroupedListRow>
                                        <SessionRow
                                            session=session.clone()
                                            core=core_for_row
                                            view_model=view_model
                                        />
                                    </GroupedListRow>
                                }
                            }).collect::<Vec<_>>()}
                        </GroupedList>
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

    let id_for_swipe = session.id.clone();
    let id_for_menu_delete = session.id.clone();
    let started_at = session.started_at.clone();
    let total_duration = session.total_duration_display.clone();
    let completion_status = session.completion_status.clone();
    let session_notes = session.notes.clone();
    let session_intention = session.session_intention.clone();
    let entry_count = session.entries.len();
    let entries = session.entries.clone();

    // Direct-delete used by the iOS swipe gesture and the long-press
    // context menu's Delete action — skips the in-row confirmation banner.
    // The gesture itself is the deliberate confirmation, matching native
    // UISwipeActionsConfiguration / UIContextMenuInteraction behaviour.
    let core_for_gesture = core.clone();
    let direct_delete = Callback::new(move |session_id: String| {
        let event = Event::Session(SessionEvent::DeleteSession { id: session_id });
        let effects = {
            let core_ref = core_for_gesture.borrow();
            core_ref.process_event(event)
        };
        process_effects_with_core(
            &core_for_gesture,
            effects,
            &view_model,
            &is_loading,
            &is_submitting,
        );
    });

    let menu_actions = vec![ContextMenuAction {
        label: "Delete".to_string(),
        destructive: true,
        on_select: Callback::new(move |_| {
            direct_delete.run(id_for_menu_delete.clone());
        }),
    }];

    view! {
        // Delete is reachable via swipe-left (mobile) and long-press → context
        // menu. No inline confirmation banner — the gesture itself is the
        // deliberate confirmation, matching native UISwipeActionsConfiguration
        // / UIContextMenuInteraction behaviour and the Routines pattern.
        <ContextMenu actions=menu_actions>
            <SwipeActions on_delete=Callback::new(move |_| {
                direct_delete.run(id_for_swipe.clone());
                        })>
                            <div class="p-card sm:p-card-comfortable space-y-3">
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
                                                    <span class="badge badge--warning">"Ended Early"</span>
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
                                    // Delete is reachable via swipe-left (mobile) and
                                    // long-press → context menu (mobile + desktop).
                                    // The previous inline text-button on the row was a
                                    // duplicate affordance; the gesture-based ones match
                                    // the iOS pattern used elsewhere (Routines, Library).
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
                                                        let class = if entry_rep_reached {
                                                            "badge badge--mono badge--warm"
                                                        } else {
                                                            "badge badge--mono badge--muted"
                                                        };
                                                        view! {
                                                            <span class=class>
                                                                {format!("{}/{}", count, target)}
                                                            </span>
                                                        }
                                                    })}
                                                    {entry_achieved_tempo.map(|tempo| {
                                                        view! {
                                                            <span class="badge badge--muted">
                                                                {format!("\u{266A} {} BPM", tempo)}
                                                            </span>
                                                        }
                                                    })}
                                                    {entry.score.map(|s| {
                                                        view! {
                                                            <span class="badge badge--accent">
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
        </SwipeActions>
        </ContextMenu>
    }
}
