use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use leptos_router::NavigateOptions;

use intrada_core::{Event, GoalEvent, GoalStatus, GoalView, ViewModel};

use crate::components::{AccentBar, AccentRow, BackLink, Button, ButtonVariant, SkeletonCardList};
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

    // Sticky snapshot of the displayed goal. Updated whenever a matching
    // goal appears in the view model (current_goal or goals list), but
    // never cleared by the view model — so a Delete that wipes the goal
    // from the model doesn't trigger a "Goal not found" flash before the
    // route transition unmounts the view. Same trick keeps the row stable
    // when an optimistic-create's client ulid is replaced by the server's
    // ulid (the row disappears from `vm.goals` for one frame).
    let goal_snapshot = RwSignal::new(None::<GoalView>);

    // Fetch the goal when the route id changes. Untracked-read the view
    // model so the Effect's only reactive dependency is `params` — that
    // way the route transition out of this view (e.g. after Delete) can't
    // re-fire FetchGoal for the just-deleted id. Also skip the fetch when
    // the goal is already loaded — either as `current_goal` from a prior
    // fetch, or in the `goals` list from the initial app-load fetch (or
    // an optimistic create the server hasn't acknowledged yet).
    Effect::new(move |_| {
        let id = params.with(|p| p.get("id").unwrap_or_default().to_string());
        if id.is_empty() {
            return;
        }
        let already_loaded = view_model.with_untracked(|vm| {
            vm.current_goal.as_ref().is_some_and(|g| g.id == id)
                || vm.goals.iter().any(|g| g.id == id)
        });
        if already_loaded {
            return;
        }
        let core_ref = core.borrow();
        let effects = core_ref.process_event(Event::Goal(GoalEvent::FetchGoal { id }));
        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
    });

    // Watch the view model for a matching goal and keep `goal_snapshot`
    // updated. Update when a matching goal appears; keep the existing
    // snapshot when its id still matches the route (Delete / refetch
    // briefly remove the goal — let the route transition dismiss the
    // view, don't flash "not found"); clear when the route id moves on
    // to a different goal so we don't render stale content from the
    // previous route.
    Effect::new(move |_| {
        let route_id = params.with(|p| p.get("id").unwrap_or_default().to_string());
        let candidate = view_model.with(|vm| {
            if let Some(g) = vm.current_goal.as_ref() {
                if g.id == route_id {
                    return Some(g.clone());
                }
            }
            vm.goals.iter().find(|g| g.id == route_id).cloned()
        });
        if candidate.is_some() {
            goal_snapshot.set(candidate);
            return;
        }
        let stale_for_other_route =
            goal_snapshot.with_untracked(|s| s.as_ref().is_some_and(|g| g.id != route_id));
        if stale_for_other_route {
            goal_snapshot.set(None);
        }
    });

    let goal = Signal::derive(move || goal_snapshot.get());

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
        let nav = navigate.clone();
        nav(
            "/goals",
            NavigateOptions {
                replace: true,
                ..Default::default()
            },
        );
        let core_ref = core.borrow();
        let effects = core_ref.process_event(Event::Goal(GoalEvent::Complete {
            id: goal_id.clone(),
        }));
        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
    };

    let on_delete = move |_: leptos::ev::MouseEvent| {
        let nav = navigate_delete.clone();
        nav(
            "/goals",
            NavigateOptions {
                replace: true,
                ..Default::default()
            },
        );
        let core_ref = core_for_delete.borrow();
        let effects = core_ref.process_event(Event::Goal(GoalEvent::Delete {
            id: goal_id_delete.clone(),
        }));
        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
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
                    <span class=if is_completed { "badge badge--accent" } else { "badge badge--warm" }>
                        {if is_completed { "Completed" } else { "Active" }}
                    </span>
                    {goal.deadline.clone().map(|d| {
                        let badge_class = if goal.is_overdue {
                            "badge badge--warning"
                        } else {
                            "badge badge--muted"
                        };
                        view! {
                            <span class=badge_class>{d}</span>
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
                        <ul class="space-y-2 list-none p-0" role="list">
                            {items.into_iter().map(|item| view! {
                                <li>
                                    <a
                                        href=format!("/library/{}", item.item_id)
                                        class="no-underline"
                                    >
                                        <AccentRow bar=AccentBar::None>
                                            <div class="flex-1 min-w-0 text-sm text-primary truncate">
                                                {item.item_title}
                                            </div>
                                        </AccentRow>
                                    </a>
                                </li>
                            }).collect::<Vec<_>>()}
                        </ul>
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
