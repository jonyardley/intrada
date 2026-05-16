use leptos::prelude::*;

use intrada_core::{Event, GoalEvent, GoalStatus, GoalView, ViewModel};

use crate::components::{EmptyState, IconName, PageHeading, SkeletonCardList};
use intrada_web::core_bridge::process_effects_with_core;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Goals list view with Active/Completed toggle tabs.
///
/// Fetches goals on mount and displays them filtered by status. A floating
/// action button links to the goal creation form.
#[component]
pub fn GoalsListView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let core = expect_context::<SharedCore>();

    // Active/Completed filter tab state
    let active_tab = RwSignal::new(GoalStatus::Active);

    // Fetch goals on mount
    Effect::new(move |_| {
        let effects = {
            let core_ref = core.borrow();
            core_ref.process_event(Event::Goal(GoalEvent::FetchGoals))
        };
        process_effects_with_core(&core, effects, &view_model, &is_loading, &is_submitting);
    });

    // Filtered goals memo
    let filtered_goals = Memo::new(move |_| {
        let vm = view_model.get();
        let tab = active_tab.get();
        vm.goals
            .into_iter()
            .filter(|g| g.status == tab)
            .collect::<Vec<_>>()
    });

    let is_active_tab = move || active_tab.get() == GoalStatus::Active;

    view! {
        <div class="space-y-4">
            <PageHeading text="Goals" />

            // Active / Completed toggle tabs
            <div class="flex gap-2">
                <button
                    class=move || if is_active_tab() {
                        "px-4 py-2 rounded-lg text-sm font-medium bg-amber-500/15 text-amber-500"
                    } else {
                        "px-4 py-2 rounded-lg text-sm font-medium text-muted hover:text-secondary"
                    }
                    on:click=move |_| active_tab.set(GoalStatus::Active)
                >
                    "Active"
                </button>
                <button
                    class=move || if !is_active_tab() {
                        "px-4 py-2 rounded-lg text-sm font-medium bg-amber-500/15 text-amber-500"
                    } else {
                        "px-4 py-2 rounded-lg text-sm font-medium text-muted hover:text-secondary"
                    }
                    on:click=move |_| active_tab.set(GoalStatus::Completed)
                >
                    "Completed"
                </button>
            </div>

            // Loading skeleton
            <Show when=move || is_loading.get()>
                <SkeletonCardList />
            </Show>

            // Goal cards
            <Show when=move || !is_loading.get()>
                {move || {
                    let goals = filtered_goals.get();
                    if goals.is_empty() {
                        let (title, body) = if is_active_tab() {
                            ("No active goals yet", "Set a goal to focus your practice")
                        } else {
                            ("No completed goals", "Goals you complete will appear here")
                        };
                        view! {
                            <EmptyState
                                icon=IconName::Star
                                title=title
                                body=body
                            />
                        }.into_any()
                    } else {
                        view! {
                            <div class="space-y-3">
                                {goals.into_iter().map(|goal| {
                                    view! { <GoalCard goal=goal /> }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </Show>

            // FAB — create new goal
            <a
                href="/goals/new"
                class="fixed bottom-20 right-4 z-40 flex h-14 w-14 items-center justify-center rounded-full bg-gradient-to-br from-amber-500 to-amber-600 text-black text-2xl font-light shadow-lg sm:bottom-6"
            >
                "+"
            </a>
        </div>
    }
}

/// Individual goal card.
#[component]
fn GoalCard(goal: GoalView) -> impl IntoView {
    let href = format!("/goals/{}", goal.id);
    let is_completed = goal.status == GoalStatus::Completed;

    // Title or notes preview
    let display_title = if let Some(ref title) = goal.title {
        title.clone()
    } else if !goal.notes_preview.is_empty() {
        goal.notes_preview.clone()
    } else {
        "Untitled goal".to_string()
    };

    let has_title = goal.title.is_some();
    let deadline = goal.deadline.clone();
    let is_overdue = goal.is_overdue;
    let has_photos = goal.has_photos;
    let completed_at = goal.completed_at.clone();

    view! {
        <a
            href=href
            class="block rounded-lg bg-surface-secondary p-card hover:bg-surface-secondary/80 transition-colors"
        >
            <div class="flex items-center justify-between gap-3">
                // Left: title/preview
                <div class="flex-1 min-w-0">
                    <p class=move || {
                        if is_completed {
                            "text-sm font-medium text-muted line-through"
                        } else if has_title {
                            "text-sm font-semibold text-primary"
                        } else {
                            "text-sm italic text-secondary"
                        }
                    }>
                        {display_title.clone()}
                    </p>
                    // Photo count indicator
                    {has_photos.then(|| view! {
                        <span class="text-xs text-muted mt-0.5 inline-flex items-center gap-1">
                            <PhotoIcon />
                        </span>
                    })}
                    // Completed date
                    {completed_at.clone().map(|date| view! {
                        <span class="text-xs text-emerald-500 mt-0.5 inline-flex items-center gap-1">
                            <CheckIcon />
                            {format_completed_date(&date)}
                        </span>
                    })}
                </div>

                // Right: deadline badge
                {deadline.clone().map(|d| {
                    let badge_class = if is_overdue {
                        "text-xs px-2 py-0.5 rounded bg-red-500/15 text-red-400"
                    } else {
                        "text-xs px-2 py-0.5 rounded bg-amber-500/15 text-amber-500"
                    };
                    view! {
                        <span class=badge_class>
                            {format_deadline(&d)}
                        </span>
                    }
                })}
            </div>
        </a>
    }
}

/// Small check icon for completed goals.
#[component]
fn CheckIcon() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-3 w-3"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2".to_string()
            aria-hidden="true"
        >
            <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
        </svg>
    }
}

/// Small photo icon.
#[component]
fn PhotoIcon() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-3 w-3"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2".to_string()
            aria-hidden="true"
        >
            <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 15.75l5.159-5.159a2.25 2.25 0 013.182 0l5.159 5.159m-1.5-1.5l1.409-1.409a2.25 2.25 0 013.182 0l2.909 2.909M3.75 21h16.5A2.25 2.25 0 0022.5 18.75V5.25A2.25 2.25 0 0020.25 3H3.75A2.25 2.25 0 001.5 5.25v13.5A2.25 2.25 0 003.75 21z" />
        </svg>
    }
}

/// Format a deadline date string (YYYY-MM-DD) for display.
fn format_deadline(deadline: &str) -> String {
    if let Ok(date) = chrono::NaiveDate::parse_from_str(deadline, "%Y-%m-%d") {
        let today = chrono::Utc::now().date_naive();
        let diff = (date - today).num_days();
        if diff == 0 {
            "Today".to_string()
        } else if diff == 1 {
            "Tomorrow".to_string()
        } else if diff > 0 && diff <= 7 {
            date.format("Due %a").to_string()
        } else {
            date.format("%b %d").to_string()
        }
    } else {
        deadline.to_string()
    }
}

/// Format a completed_at date (RFC3339) for display.
fn format_completed_date(date_str: &str) -> String {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
        dt.format("%b %d").to_string()
    } else {
        "Completed".to_string()
    }
}
