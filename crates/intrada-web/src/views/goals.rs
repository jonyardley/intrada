use leptos::prelude::*;
use leptos_router::components::A;

use intrada_core::model::{GoalProgress, GoalView};
use intrada_core::{Event, ViewModel};

use crate::components::{Button, ButtonVariant, Card, PageHeading, SkeletonCardList};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Goals page — lists active goals with progress bars, plus completed/archived history.
#[component]
pub fn GoalsListView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();
    let show_history = RwSignal::new(false);

    view! {
        <div>
            <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 mb-6">
                <PageHeading text="Goals" />
                <A href="/goals/new" attr:class="cta-link">
                    "Set a Goal"
                </A>
            </div>

            {move || {
                if is_loading.get() {
                    return view! {
                        <SkeletonCardList count=3 />
                    }.into_any();
                }

                let vm = view_model.get();
                let active: Vec<GoalView> = vm.goals.iter().filter(|g| g.status == "active").cloned().collect();
                let completed: Vec<GoalView> = vm.goals.iter().filter(|g| g.status == "completed" || g.status == "archived").cloned().collect();

                if active.is_empty() && completed.is_empty() {
                    view! {
                        <div class="text-center py-12 px-4 sm:px-6 lg:px-0">
                            <div class="mb-4 text-4xl">"\u{1F3AF}"</div>
                            <p class="text-secondary font-medium">"No goals yet"</p>
                            <p class="text-sm text-muted mt-2">"Set a goal to track your progress and stay motivated."</p>
                            <div class="mt-6">
                                <A href="/goals/new" attr:class="cta-link">
                                    "Set Your First Goal"
                                </A>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div>
                            // Active goals
                            {if !active.is_empty() {
                                view! {
                                    <div class="space-y-3 mb-8">
                                        <h2 class="section-title">"Active Goals"</h2>
                                        {active.into_iter().map(|goal| {
                                            view! { <GoalCard goal=goal /> }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }}

                            // Completed / Archived goals (collapsible)
                            {if !completed.is_empty() {
                                let count = completed.len();
                                let history_items = completed.into_iter().map(|goal| {
                                    view! { <GoalCard goal=goal /> }
                                }).collect::<Vec<_>>();
                                view! {
                                    <div class="space-y-3">
                                        <button
                                            class="flex items-center gap-2 section-title text-muted cursor-pointer hover:text-secondary transition-colors"
                                            on:click=move |_| show_history.update(|v| *v = !*v)
                                            attr:aria-expanded=move || show_history.get().to_string()
                                        >
                                            <span class="text-xs transition-transform" class:rotate-90=move || show_history.get()>
                                                "\u{25B6}"
                                            </span>
                                            {format!("History ({count})")}
                                        </button>
                                        <div class="space-y-3" style:display=move || if show_history.get() { "block" } else { "none" }>
                                            {history_items}
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! { <div></div> }.into_any()
                            }}
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

/// Icon for a goal type.
fn goal_type_icon(kind_type: &str) -> &'static str {
    match kind_type {
        "session_frequency" => "\u{1F4C5}", // 📅
        "practice_time" => "\u{23F1}",      // ⏱
        "item_mastery" => "\u{2B50}",       // ⭐
        "milestone" => "\u{1F3AF}",         // 🎯
        _ => "\u{1F3AF}",
    }
}

/// A single goal card with progress bar and actions.
#[component]
fn GoalCard(goal: GoalView) -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let confirm_delete = RwSignal::new(false);

    let id_complete = goal.id.clone();
    let id_archive = goal.id.clone();
    let id_reactivate = goal.id.clone();
    let id_delete = goal.id.clone();

    let title = goal.title.clone();
    let kind_label = goal.kind_label.clone();
    let kind_type = goal.kind_type.clone();
    let status = goal.status.clone();
    let progress = goal.progress.clone();
    let deadline = goal.deadline.clone();
    let completed_at = goal.completed_at.clone();
    let item_title = goal.item_title.clone();

    let is_active = status == "active";
    let is_archived = status == "archived";

    // Check if deadline is past (overdue)
    let is_overdue = is_active
        && deadline
            .as_ref()
            .and_then(|d| chrono::DateTime::parse_from_rfc3339(d).ok())
            .map(|dt| dt < chrono::Utc::now())
            .unwrap_or(false);

    let icon = goal_type_icon(&kind_type);

    view! {
        <Card>
            {move || {
                if confirm_delete.get() {
                    let core_del = core.clone();
                    let id_del = id_delete.clone();
                    view! {
                        <div>
                            <p class="text-sm text-danger-text mb-3">"Delete this goal? This cannot be undone."</p>
                            <div class="flex gap-2">
                                <Button
                                    variant=ButtonVariant::Danger
                                    loading=Signal::derive(move || is_submitting.get())
                                    on_click=Callback::new(move |_| {
                                        let event = Event::Goal(intrada_core::domain::goal::GoalEvent::Delete { id: id_del.clone() });
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
                    let title = title.clone();
                    let kind_label = kind_label.clone();
                    let status = status.clone();
                    let progress = progress.clone();
                    let deadline = deadline.clone();
                    let completed_at = completed_at.clone();
                    let item_title = item_title.clone();
                    let id_complete = id_complete.clone();
                    let id_archive = id_archive.clone();
                    let id_reactivate = id_reactivate.clone();
                    let core_actions = core.clone();

                    view! {
                        <div class="space-y-3">
                            // Header: icon + title + status badge
                            <div class="flex items-start gap-3">
                                <span class="text-xl flex-shrink-0">{icon}</span>
                                <div class="flex-1 min-w-0">
                                    <div class="flex flex-wrap items-baseline gap-x-3 gap-y-1">
                                        <span class="text-sm font-medium text-primary">{title}</span>
                                        {match status.as_str() {
                                            "completed" => view! {
                                                <span class="inline-flex items-center rounded-full bg-success-surface px-2 py-0.5 text-xs font-medium text-success-text">
                                                    "Completed"
                                                </span>
                                            }.into_any(),
                                            "archived" => view! {
                                                <span class="inline-flex items-center rounded-full bg-surface-secondary px-2 py-0.5 text-xs font-medium text-muted">
                                                    "Archived"
                                                </span>
                                            }.into_any(),
                                            _ => view! { <span></span> }.into_any(),
                                        }}
                                        {if is_overdue {
                                            view! {
                                                <span class="inline-flex items-center rounded-full bg-warning-surface px-2 py-0.5 text-xs font-medium text-warning-text">
                                                    "Overdue"
                                                </span>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}
                                    </div>
                                    <p class="text-xs text-muted mt-0.5">{kind_label}</p>
                                    // Show linked item title for mastery goals
                                    {item_title.map(|t| view! {
                                        <p class="text-xs text-faint mt-0.5">{format!("Item: {t}")}</p>
                                    })}
                                </div>
                            </div>

                            // Progress bar (active goals only)
                            {progress.map(|p| view! { <GoalProgressBar progress=p /> })}

                            // Deadline / completed date
                            <div class="flex flex-wrap gap-4 text-xs text-faint">
                                {deadline.map(|d| view! {
                                    <span>{format!("Deadline: {}", format_date_short(&d))}</span>
                                })}
                                {completed_at.map(|d| view! {
                                    <span>{format!("Completed: {}", format_date_short(&d))}</span>
                                })}
                            </div>

                            // Action buttons
                            <div class="flex flex-wrap gap-3 pt-1">
                                {if is_active {
                                    let core_complete = core_actions.clone();
                                    let core_archive = core_actions.clone();
                                    view! {
                                        <button
                                            class="text-xs text-success-text hover:text-success-hover font-medium"
                                            on:click=move |_| {
                                                let event = Event::Goal(intrada_core::domain::goal::GoalEvent::Complete { id: id_complete.clone() });
                                                let core_ref = core_complete.borrow();
                                                let effects = core_ref.process_event(event);
                                                process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                            }
                                        >
                                            "Complete"
                                        </button>
                                        <button
                                            class="text-xs text-muted hover:text-secondary font-medium"
                                            on:click=move |_| {
                                                let event = Event::Goal(intrada_core::domain::goal::GoalEvent::Archive { id: id_archive.clone() });
                                                let core_ref = core_archive.borrow();
                                                let effects = core_ref.process_event(event);
                                                process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                            }
                                        >
                                            "Archive"
                                        </button>
                                    }.into_any()
                                } else if is_archived {
                                    let core_reactivate = core_actions.clone();
                                    view! {
                                        <button
                                            class="text-xs text-accent-text hover:text-accent-hover font-medium"
                                            on:click=move |_| {
                                                let event = Event::Goal(intrada_core::domain::goal::GoalEvent::Reactivate { id: id_reactivate.clone() });
                                                let core_ref = core_reactivate.borrow();
                                                let effects = core_ref.process_event(event);
                                                process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                            }
                                        >
                                            "Reactivate"
                                        </button>
                                    }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }}

                                <button
                                    class="text-xs text-danger-text hover:text-danger-hover font-medium"
                                    on:click=move |_| { confirm_delete.set(true); }
                                >
                                    "Delete"
                                </button>
                            </div>
                        </div>
                    }.into_any()
                }
            }}
        </Card>
    }
}

/// Progress bar for a goal.
#[component]
fn GoalProgressBar(progress: GoalProgress) -> impl IntoView {
    let pct = progress.percentage.clamp(0.0, 100.0);
    let width_style = format!("width: {pct:.0}%");

    view! {
        <div class="space-y-1.5">
            <div class="h-2 rounded-full bg-surface-secondary overflow-hidden">
                <div
                    class="h-full rounded-full bg-accent-focus transition-all duration-500"
                    style=width_style
                />
            </div>
            <p class="text-xs text-secondary">{progress.display_text}</p>
        </div>
    }
}

/// Format an ISO date string to a short display format (e.g. "24 Feb 2026").
fn format_date_short(iso: &str) -> String {
    // Parse the ISO date and format nicely, fallback to raw string
    chrono::DateTime::parse_from_rfc3339(iso)
        .map(|dt| dt.format("%d %b %Y").to_string())
        .unwrap_or_else(|_| iso.to_string())
}
