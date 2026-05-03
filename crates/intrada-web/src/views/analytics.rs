use intrada_core::analytics::{
    AnalyticsView, Direction, NeglectedItem, ScoreChange, WeeklySummary,
};
use intrada_core::ViewModel;
use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::{
    AccentBar, AccentRow, Card, EmptyState, Icon, IconName, LineChart, PageHeading, SectionLabel,
    SkeletonBlock, StatCard, StatTone,
};
use intrada_web::core_bridge::init_core;
use intrada_web::types::{IsLoading, IsSubmitting};

/// Analytics dashboard page — shows practice insights and trends.
#[component]
pub fn AnalyticsPage() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();

    view! {
        <div>
            <PageHeading text="Analytics" subtitle="See how your practice is shaping up with stats, trends, and insights." />

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
                        <EmptyState
                            icon=IconName::BarChart
                            title="No session data yet"
                            body="Complete some sessions to see your analytics. Track your progress, streaks, and most practised items."
                        >
                            <A href="/sessions/new" attr:class="cta-link">
                                "Start a Session"
                            </A>
                        </EmptyState>
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
    // Weekly hours formatted as a single decimal — matches the Pencil
    // "8.5" style. Whole-hour values render as "5", not "5.0".
    let hours_decimal = weekly.total_minutes as f64 / 60.0;
    let hours_display = if hours_decimal.fract() == 0.0 {
        format!("{:.0}", hours_decimal)
    } else {
        format!("{:.1}", hours_decimal)
    };
    let items_display = format!("{}", weekly.items_covered);

    view! {
        <div class="space-y-6">
            // ── Top stat row — three StatCard refresh variants. The
            // tone of the value text (accent purple / warm gold / white)
            // mirrors the gradient bar on the left of each card so a
            // stat's category reads at a glance.
            <div class="grid grid-cols-3 gap-3">
                <StatCard
                    title="Day Streak"
                    value=streak_display
                    bar=AccentBar::Gold
                    tone=StatTone::Accent
                />
                <StatCard
                    title="Hrs This Week"
                    value=hours_display
                    bar=AccentBar::Blue
                    tone=StatTone::WarmAccent
                />
                <StatCard
                    title="Items This Week"
                    value=items_display
                    bar=AccentBar::Gold
                />
            </div>

            // ── Weekly Summary Card ──────────────────────────────
            <Card>
                <SectionLabel text="This Week" />
                <WeekComparisonRow weekly=weekly.clone() />
                // ── Neglected + Score Changes (2-col on desktop, stacked on mobile)
                <div class="mt-4 grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <NeglectedItemsList items=neglected_items />
                    <ScoreChangesList changes=score_changes />
                </div>
            </Card>

            // ── US2: Practice History Chart ──────────────────────
            <Card>
                <SectionLabel text="Practice History (28 days)" />
                {if daily_totals.iter().all(|d| d.minutes == 0) {
                    view! {
                        <p class="text-sm text-muted text-center py-8">
                            "No session data for the past 28 days. "
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
                <SectionLabel text="Most Practised" />
                {if top_items.is_empty() {
                    view! {
                        <p class="text-sm text-muted text-center py-4">
                            "No session data yet. "
                            <A href="/sessions/new" attr:class="text-accent-text hover:text-accent-hover underline">
                                "Start a session"
                            </A>
                            " to track your most practised items."
                        </p>
                    }.into_any()
                } else {
                    // Each row is an AccentRow with bar=None — token
                    // consistency with the rest of the refresh, but no
                    // gold/blue stripe (the list is uniform "top items"
                    // with rank as the differentiator, not type).
                    view! {
                        <ul class="space-y-2 list-none p-0">
                            {top_items.into_iter().enumerate().map(|(i, item)| {
                                let hours = item.total_minutes / 60;
                                let mins = item.total_minutes % 60;
                                let time = if hours > 0 {
                                    format!("{}h {}m", hours, mins)
                                } else {
                                    format!("{}m", mins)
                                };
                                view! {
                                    <li>
                                        <AccentRow bar=AccentBar::None>
                                            <span class="text-xs font-medium text-faint w-5 text-right shrink-0 tabular-nums">
                                                {format!("{}.", i + 1)}
                                            </span>
                                            <div class="flex flex-col flex-1 min-w-0 gap-0.5">
                                                <span class="text-sm font-semibold text-primary truncate">
                                                    {item.item_title.clone()}
                                                </span>
                                                <span class="text-xs text-muted">
                                                    {format!(
                                                        "{} \u{00B7} {} session{}",
                                                        item.item_type,
                                                        item.session_count,
                                                        if item.session_count == 1 { "" } else { "s" }
                                                    )}
                                                </span>
                                            </div>
                                            <span class="text-sm font-medium text-accent-text shrink-0 tabular-nums">
                                                {time}
                                            </span>
                                        </AccentRow>
                                    </li>
                                }
                            }).collect::<Vec<_>>()}
                        </ul>
                    }.into_any()
                }}
            </Card>

            // ── US4: Score Trends ────────────────────────────────
            <Card>
                <SectionLabel text="Score Trends" />
                {if score_trends.is_empty() {
                    view! {
                        <p class="text-sm text-muted text-center py-4">
                            "No scored items yet. During practice, rate your confidence "
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
                                                    _ => "bg-muted/40",
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
    let (icon, color) = match direction {
        Direction::Up => (IconName::ArrowRight, "text-success-text"),
        Direction::Down => (IconName::ArrowRight, "text-muted"),
        Direction::Same => (IconName::ArrowRight, "text-muted"),
    };
    let rotate_class = match direction {
        Direction::Up => "w-3 h-3 inline-block -rotate-45",
        Direction::Down => "w-3 h-3 inline-block rotate-45",
        Direction::Same => "w-3 h-3 inline-block",
    };

    view! {
        <div class="text-center">
            <div class="text-lg font-semibold text-primary">{value}</div>
            <div class=format!("text-xs {color}")>
                {if has_prev {
                    view! {
                        <span class="inline-flex items-center gap-0.5">
                            <Icon name=icon class=rotate_class />
                            {format!("from {prev_value}")}
                        </span>
                    }.into_any()
                } else {
                    view! {
                        <span>"no data last week"</span>
                    }.into_any()
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
            <SkeletonBlock height="h-24" />
            // Weekly summary card skeleton
            <SkeletonBlock height="h-32" />
            // Chart skeleton
            <SkeletonBlock height="h-52" />
            // List skeletons
            <SkeletonBlock height="h-48" />
            <SkeletonBlock height="h-36" />
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
        init_core(&view_model, &is_loading, &is_submitting);
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
