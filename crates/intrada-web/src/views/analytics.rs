use intrada_core::analytics::AnalyticsView;
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
    } = analytics;

    // Format weekly time display
    let hours = weekly.total_minutes / 60;
    let mins = weekly.total_minutes % 60;
    let time_display = if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    };

    let streak_display = format!("{}", streak.current_days);
    let session_display = format!("{}", weekly.session_count);

    view! {
        <div class="space-y-6">
            // ── US1: Overview Stats ──────────────────────────────
            <div class="grid grid-cols-3 gap-3">
                <StatCard
                    title="This Week"
                    value=time_display
                    subtitle="practice time"
                />
                <StatCard
                    title="Sessions"
                    value=session_display
                    subtitle="this week"
                />
                <StatCard
                    title="Streak"
                    value=streak_display
                    subtitle="days"
                />
            </div>

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

/// Skeleton loading state with animated placeholder cards.
#[component]
fn SkeletonDashboard() -> impl IntoView {
    view! {
        <div class="space-y-6 animate-pulse">
            // Stat card skeletons
            <div class="grid grid-cols-3 gap-3">
                <div class="bg-surface-secondary rounded-xl h-24"></div>
                <div class="bg-surface-secondary rounded-xl h-24"></div>
                <div class="bg-surface-secondary rounded-xl h-24"></div>
            </div>
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
