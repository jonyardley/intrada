use intrada_core::analytics::{
    AnalyticsView, Direction, NeglectedItem, ScoreChange, WeeklySummary,
};
use intrada_core::ViewModel;
use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::{Card, LineChart, PageHeading, StatCard};
use intrada_web::core_bridge::fetch_initial_data;
use intrada_web::types::{IsLoading, IsSubmitting};

/// Analytics dashboard page — shows practice insights and trends.
#[component]
pub fn AnalyticsPage() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();

    view! {
        <div>
            <PageHeading text="Analytics" />

            {move || {
                let loading = is_loading.get();
                let vm = view_model.get();

                if loading && vm.analytics.is_none() {
                    // Skeleton / loading state (FR-012)
                    return view! {
                        <SkeletonDashboard />
                    }.into_any();
                }

                // Error state with retry (FR-013)
                if let Some(ref error_msg) = vm.error {
                    if vm.analytics.is_none() {
                        let msg = error_msg.clone();
                        return view! {
                            <ErrorState message=msg />
                        }.into_any();
                    }
                }

                match vm.analytics {
                    Some(analytics) => view! {
                        <AnalyticsDashboard analytics=analytics />
                    }.into_any(),
                    None => view! {
                        <EmptyState />
                    }.into_any(),
                }
            }}
        </div>
    }
}

/// Full analytics dashboard with all sections.
#[component]
fn AnalyticsDashboard(analytics: AnalyticsView) -> impl IntoView {
    let AnalyticsView {
        weekly_summary: weekly,
        streak,
        daily_totals,
        top_items,
        score_trends,
        neglected_items,
        score_changes,
    } = analytics;

    let streak_display = format!("{}", streak.current_days);

    view! {
        <div class="space-y-6">
            // ── Streak stat card (single, no longer in 3-column grid) ──
            <StatCard
                title="Streak"
                value=streak_display
                subtitle="days"
            />

            // ── Weekly Summary Card ──────────────────────────────
            <Card>
                <h3 class="card-title">"This Week"</h3>
                <WeekComparisonRow weekly=weekly.clone() />
                // ── Neglected + Score Changes (2-col on desktop, stacked on mobile)
                <div class="mt-4 grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <NeglectedItemsList items=neglected_items />
                    <ScoreChangesList changes=score_changes />
                </div>
            </Card>

            // ── US2: Practice History Chart ──────────────────────
            <Card>
                <h3 class="card-title">"Practice History (28 days)"</h3>
                {if daily_totals.iter().all(|d| d.minutes == 0) {
                    view! {
                        <p class="text-sm text-muted text-center py-8">
                            "No practice data for the past 28 days. "
                            <A href="/sessions/new" attr:class="text-accent-text hover:text-accent-hover underline">
                                "Start a session"
                            </A>
                            " to see your progress here."
                        </p>
                    }.into_any()
                } else {
                    view! {
                        <LineChart data=daily_totals />
                    }.into_any()
                }}
            </Card>

            // ── US3: Most Practised Items ────────────────────────
            <Card>
                <h3 class="card-title">"Most Practised"</h3>
                {if top_items.is_empty() {
                    view! {
                        <p class="text-sm text-muted text-center py-4">
                            "No practice data yet. "
                            <A href="/sessions/new" attr:class="text-accent-text hover:text-accent-hover underline">
                                "Start a session"
                            </A>
                            " to track your most practised items."
                        </p>
                    }.into_any()
                } else {
                    view! {
                        <ul class="space-y-2">
                            {top_items.into_iter().enumerate().map(|(i, item)| {
                                let hours = item.total_minutes / 60;
                                let mins = item.total_minutes % 60;
                                let time = if hours > 0 {
                                    format!("{}h {}m", hours, mins)
                                } else {
                                    format!("{}m", mins)
                                };
                                view! {
                                    <li class="flex items-center justify-between py-1.5 border-b border-border-default/50 last:border-0">
                                        <div class="flex items-center gap-2 min-w-0">
                                            <span class="text-xs text-faint w-5 text-right shrink-0">
                                                {format!("{}.", i + 1)}
                                            </span>
                                            <span class="text-sm text-primary truncate">
                                                {item.item_title.clone()}
                                            </span>
                                            <span class="text-xs text-faint shrink-0">
                                                {item.item_type.clone()}
                                            </span>
                                        </div>
                                        <div class="flex items-center gap-3 shrink-0 ml-2">
                                            <span class="text-xs text-muted">
                                                {format!("{} sessions", item.session_count)}
                                            </span>
                                            <span class="text-sm font-medium text-accent-text">
                                                {time}
                                            </span>
                                        </div>
                                    </li>
                                }
                            }).collect::<Vec<_>>()}
                        </ul>
                    }.into_any()
                }}
            </Card>

            // ── US4: Score Trends ────────────────────────────────
            <Card>
                <h3 class="card-title">"Score Trends"</h3>
                {if score_trends.is_empty() {
                    view! {
                        <p class="text-sm text-muted text-center py-4">
                            "No scored items yet. During a practice session, rate your confidence "
                            "on items (1\u{2013}5) to see your progress over time."
                        </p>
                    }.into_any()
                } else {
                    view! {
                        <ul class="space-y-3">
                            {score_trends.into_iter().map(|trend| {
                                view! {
                                    <li class="py-1.5 border-b border-border-default/50 last:border-0">
                                        <div class="flex items-center justify-between mb-1">
                                            <span class="text-sm text-primary truncate">
                                                {trend.item_title.clone()}
                                            </span>
                                            <span class="text-sm font-medium text-accent-text">
                                                {format!("{}/5", trend.latest_score)}
                                            </span>
                                        </div>
                                        <div class="flex items-center gap-1">
                                            {trend.scores.iter().map(|point| {
                                                // Score dots: filled circles scaled by score value
                                                let size = match point.score {
                                                    1 => "w-2 h-2",
                                                    2 => "w-2.5 h-2.5",
                                                    3 => "w-3 h-3",
                                                    4 => "w-3.5 h-3.5",
                                                    5 => "w-4 h-4",
                                                    _ => "w-2 h-2",
                                                };
                                                let color = match point.score {
                                                    1 => "bg-danger/60",
                                                    2 => "bg-warning/40",
                                                    3 => "bg-warning/60",
                                                    4 => "bg-success/60",
                                                    5 => "bg-success/80",
                                                    _ => "bg-gray-400/40",
                                                };
                                                view! {
                                                    <div
                                                        class=format!("{size} {color} rounded-full")
                                                        title=format!("{}: {}/5", point.date, point.score)
                                                        role="img"
                                                        aria-label=format!("Score {}/5 on {}", point.score, point.date)
                                                    ></div>
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    </li>
                                }
                            }).collect::<Vec<_>>()}
                        </ul>
                    }.into_any()
                }}
            </Card>
        </div>
    }
}

/// Renders 3 comparison metrics (time, sessions, items) in a grid.
/// Each metric shows the current value, a directional arrow with comparison text,
/// and a label. When `has_prev_week_data` is false, shows "no data last week".
#[component]
fn WeekComparisonRow(weekly: WeeklySummary) -> impl IntoView {
    let hours = weekly.total_minutes / 60;
    let mins = weekly.total_minutes % 60;
    let time_display = if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    };
    let session_display = format!("{}", weekly.session_count);
    let items_display = format!("{}", weekly.items_covered);

    view! {
        <div class="grid grid-cols-3 gap-3">
            <ComparisonMetric
                value=time_display
                label="Practice Time"
                direction=weekly.time_direction.clone()
                prev_value=format_prev_time(weekly.prev_total_minutes)
                has_prev=weekly.has_prev_week_data
            />
            <ComparisonMetric
                value=session_display
                label="Sessions"
                direction=weekly.sessions_direction.clone()
                prev_value=format!("{}", weekly.prev_session_count)
                has_prev=weekly.has_prev_week_data
            />
            <ComparisonMetric
                value=items_display
                label="Items"
                direction=weekly.items_direction.clone()
                prev_value=format!("{}", weekly.prev_items_covered)
                has_prev=weekly.has_prev_week_data
            />
        </div>
    }
}

/// Single comparison metric: current value + direction arrow + label.
#[component]
fn ComparisonMetric(
    #[prop(into)] value: String,
    #[prop(into)] label: String,
    direction: Direction,
    #[prop(into)] prev_value: String,
    has_prev: bool,
) -> impl IntoView {
    let (arrow, color) = match direction {
        Direction::Up => ("\u{2191}", "text-success-text"), // ↑
        Direction::Down => ("\u{2193}", "text-muted"),      // ↓
        Direction::Same => ("\u{2192}", "text-muted"),      // →
    };

    view! {
        <div class="text-center">
            <div class="text-lg font-semibold text-primary">{value}</div>
            <div class=format!("text-xs {color}")>
                {if has_prev {
                    format!("{arrow} from {prev_value}")
                } else {
                    "no data last week".to_string()
                }}
            </div>
            <div class="field-label mt-1">{label}</div>
        </div>
    }
}

/// Renders up to 5 neglected items with "X days ago" or "never practised" labels.
/// Hidden when the list is empty (FR-007).
#[component]
fn NeglectedItemsList(items: Vec<NeglectedItem>) -> impl IntoView {
    view! {
        {if items.is_empty() {
            None
        } else {
            Some(view! {
                <div>
                    <h4 class="card-title">"Needs attention"</h4>
                    <ul class="space-y-1.5">
                        {items.into_iter().map(|item| {
                            let subtitle = match item.days_since_practice {
                                None => "never practised".to_string(),
                                Some(days) => format!("{days} days ago"),
                            };
                            view! {
                                <li class="flex items-center justify-between">
                                    <span class="text-sm text-primary truncate">{item.item_title}</span>
                                    <span class="text-xs text-muted shrink-0 ml-2">{subtitle}</span>
                                </li>
                            }
                        }).collect::<Vec<_>>()}
                    </ul>
                </div>
            })
        }}
    }
}

/// Renders up to 5 score changes with "X → Y (+N)" or "new" labels.
/// Neutral framing only — no words like "worse" or "declined" (FR-009).
/// Hidden when empty (FR-007).
#[component]
fn ScoreChangesList(changes: Vec<ScoreChange>) -> impl IntoView {
    view! {
        {if changes.is_empty() {
            None
        } else {
            Some(view! {
                <div>
                    <h4 class="card-title">"Improvements"</h4>
                    <ul class="space-y-1.5">
                        {changes.into_iter().map(|change| {
                            let transition = if change.is_new {
                                format!("{}/5", change.current_score)
                            } else {
                                format!(
                                    "{} \u{2192} {}",
                                    change.previous_score.unwrap_or(0),
                                    change.current_score
                                )
                            };
                            let delta_text = if change.is_new {
                                "new".to_string()
                            } else if change.delta > 0 {
                                format!("(+{})", change.delta)
                            } else {
                                format!("({})", change.delta)
                            };
                            view! {
                                <li class="flex items-center justify-between">
                                    <span class="text-sm text-primary truncate">{change.item_title}</span>
                                    <div class="flex items-center gap-2 shrink-0 ml-2">
                                        <span class="text-sm text-secondary">{transition}</span>
                                        <span class="text-sm text-accent-text">{delta_text}</span>
                                    </div>
                                </li>
                            }
                        }).collect::<Vec<_>>()}
                    </ul>
                </div>
            })
        }}
    }
}

/// Format previous week's time display.
fn format_prev_time(minutes: u32) -> String {
    let hours = minutes / 60;
    let mins = minutes % 60;
    if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

/// Skeleton loading state with animated placeholder cards.
#[component]
fn SkeletonDashboard() -> impl IntoView {
    view! {
        <div class="space-y-6 animate-pulse">
            // Streak stat card skeleton
            <div class="bg-surface-secondary rounded-xl h-24"></div>
            // Weekly summary card skeleton
            <div class="bg-surface-secondary rounded-xl h-32"></div>
            // Chart skeleton
            <div class="bg-surface-secondary rounded-xl h-52"></div>
            // List skeletons
            <div class="bg-surface-secondary rounded-xl h-48"></div>
            <div class="bg-surface-secondary rounded-xl h-36"></div>
        </div>
    }
}

/// Error state with retry button when data fetch fails (FR-013).
#[component]
fn ErrorState(#[prop(into)] message: String) -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    let on_retry = move |_| {
        fetch_initial_data(&view_model, &is_loading, &is_submitting);
    };

    view! {
        <div class="text-center py-16">
            <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-16 w-16 mx-auto text-danger/60 mb-4"
                viewBox="0 0 20 20"
                fill="currentColor"
            >
                <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
            </svg>
            <h3 class="text-lg font-semibold text-secondary mb-2">"Something went wrong"</h3>
            <p class="text-sm text-muted mb-6 max-w-sm mx-auto">{message}</p>
            <button
                on:click=on_retry
                class="cta-link"
            >
                "Retry"
            </button>
        </div>
    }
}

/// Empty state when no sessions exist at all.
#[component]
fn EmptyState() -> impl IntoView {
    view! {
        <div class="text-center py-16">
            // Chart icon
            <svg
                xmlns="http://www.w3.org/2000/svg"
                class="h-16 w-16 mx-auto text-faint mb-4"
                viewBox="0 0 20 20"
                fill="currentColor"
            >
                <path d="M2 11a1 1 0 011-1h2a1 1 0 011 1v5a1 1 0 01-1 1H3a1 1 0 01-1-1v-5zM8 7a1 1 0 011-1h2a1 1 0 011 1v9a1 1 0 01-1 1H9a1 1 0 01-1-1V7zM14 4a1 1 0 011-1h2a1 1 0 011 1v12a1 1 0 01-1 1h-2a1 1 0 01-1-1V4z" />
            </svg>
            <h3 class="text-lg font-semibold text-secondary mb-2">"No practice data yet"</h3>
            <p class="text-sm text-muted mb-6 max-w-sm mx-auto">
                "Complete practice sessions to see your analytics. Track your progress, streaks, and most practised items."
            </p>
            <A
                href="/sessions/new"
                attr:class="cta-link"
            >
                "Start a Session"
            </A>
        </div>
    }
}
