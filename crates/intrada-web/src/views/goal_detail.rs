use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use leptos_router::NavigateOptions;

use intrada_core::{Event, GoalEvent, GoalStatus, GoalView, ViewModel};

use crate::components::{BackLink, Button, ButtonVariant, SkeletonCardList};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Detail view for a single goal.
///
/// Reached via `/goals/:id`. Shows full goal info including notes, photos,
/// linked items, and actions (complete, delete).
#[component]
pub fn GoalDetailView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    let params = use_params_map();

    // Fetch goal on mount
    Effect::new(move |_| {
        let id = params.with(|p| p.get("id").unwrap_or_default().to_string());
        if !id.is_empty() {
            let core_ref = core.borrow();
            let effects = core_ref.process_event(Event::Goal(GoalEvent::FetchGoal { id }));
            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
        }
    });

    // Read current_goal from ViewModel
    let goal = Signal::derive(move || view_model.with(|vm| vm.current_goal.clone()));

    view! {
        <div class="space-y-4">
            <BackLink label="Back to Goals" href="/goals".to_string() />

            <Show when=move || is_loading.get()>
                <SkeletonCardList />
            </Show>

            <Show when=move || !is_loading.get()>
                {move || {
                    match goal.get() {
                        Some(g) => view! { <GoalDetailContent goal=g /> }.into_any(),
                        None => view! {
                            <p class="text-muted text-sm">"Goal not found."</p>
                        }.into_any(),
                    }
                }}
            </Show>
        </div>
    }
}

/// Inner content for a loaded goal.
#[component]
fn GoalDetailContent(goal: GoalView) -> impl IntoView {
    let core = expect_context::<SharedCore>();
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let navigate = use_navigate();
    let navigate_delete = use_navigate();

    let goal_id = goal.id.clone();
    let goal_id_delete = goal.id.clone();
    let is_completed = goal.status == GoalStatus::Completed;

    let core_for_delete = core.clone();
    let on_complete = move |_: leptos::ev::MouseEvent| {
        let core_ref = core.borrow();
        let effects = core_ref.process_event(Event::Goal(GoalEvent::Complete {
            id: goal_id.clone(),
        }));
        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
        let nav = navigate.clone();
        nav("/goals", NavigateOptions::default());
    };

    let on_delete = move |_: leptos::ev::MouseEvent| {
        let core_ref = core_for_delete.borrow();
        let effects = core_ref.process_event(Event::Goal(GoalEvent::Delete {
            id: goal_id_delete.clone(),
        }));
        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
        let nav = navigate_delete.clone();
        nav("/goals", NavigateOptions::default());
    };

    let title_display = goal
        .title
        .clone()
        .unwrap_or_else(|| "Untitled goal".to_string());

    view! {
        <div class="space-y-6">
            // Title + status
            <div class="space-y-2">
                <h2 class="page-title">{title_display}</h2>
                <div class="flex items-center gap-2 flex-wrap">
                    // Status badge
                    <span class=if is_completed {
                        "text-xs px-2 py-0.5 rounded bg-emerald-500/15 text-emerald-400 font-medium"
                    } else {
                        "text-xs px-2 py-0.5 rounded bg-amber-500/15 text-amber-500 font-medium"
                    }>
                        {if is_completed { "Completed" } else { "Active" }}
                    </span>
                    // Deadline badge
                    {goal.deadline.clone().map(|d| {
                        let badge_class = if goal.is_overdue {
                            "text-xs px-2 py-0.5 rounded bg-red-500/15 text-red-400"
                        } else {
                            "text-xs px-2 py-0.5 rounded bg-surface-secondary text-muted"
                        };
                        view! {
                            <span class=badge_class>
                                {d}
                            </span>
                        }
                    })}
                </div>
            </div>

            // Notes
            {goal.notes.clone().map(|notes| view! {
                <div class="space-y-1">
                    <p class="field-label">"Notes"</p>
                    <p class="text-sm text-secondary whitespace-pre-wrap">{notes}</p>
                </div>
            })}

            // Photos
            {(!goal.photos.is_empty()).then(|| {
                let photos = goal.photos.clone();
                view! {
                    <div class="space-y-2">
                        <p class="field-label">"Photos"</p>
                        <div class="flex gap-2 flex-wrap">
                            {photos.into_iter().map(|photo| view! {
                                <img
                                    src=photo.url
                                    class="w-16 h-16 rounded-lg object-cover"
                                    alt="Goal photo"
                                />
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                }
            })}

            // Linked items
            {(!goal.items.is_empty()).then(|| {
                let items = goal.items.clone();
                view! {
                    <div class="space-y-2">
                        <p class="field-label">"Linked items"</p>
                        <div class="space-y-1">
                            {items.into_iter().map(|item| view! {
                                <a
                                    href=format!("/library/{}", item.item_id)
                                    class="block rounded-lg bg-surface-secondary p-card-compact text-sm text-secondary hover:text-primary transition-colors"
                                >
                                    {item.item_title}
                                </a>
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                }
            })}

            // Actions
            <div class="space-y-3 pt-4 border-t border-border-default">
                {(!is_completed).then(|| view! {
                    <Button
                        variant=ButtonVariant::Primary
                        on_click=Callback::new(on_complete)
                        full_width=true
                    >
                        "Mark Complete"
                    </Button>
                })}
                <Button
                    variant=ButtonVariant::Danger
                    on_click=Callback::new(on_delete)
                    full_width=true
                >
                    "Delete Goal"
                </Button>
            </div>
        </div>
    }
}
