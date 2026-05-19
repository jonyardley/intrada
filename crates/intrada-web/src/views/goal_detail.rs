use std::collections::HashSet;
use std::sync::Arc;

use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use leptos_router::NavigateOptions;

use intrada_core::{Event, GoalEvent, GoalStatus, GoalView, LinkGoalItem, ViewModel};

use crate::components::{
    AccentBar, AccentRow, BackLink, BottomSheet, BuilderItemRow, Button, ButtonVariant,
    SkeletonCardList,
};
use crate::views::GoalEditFormView;
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

#[component]
pub fn GoalDetailView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    let params = use_params_map();

    // Sticky so a Delete that wipes the goal from the model doesn't flash
    // "not found" before the route transition unmounts the view.
    let goal_snapshot = RwSignal::new(None::<GoalView>);

    // Untracked view-model read: only `params` should re-fire this Effect,
    // otherwise the route transition out of the view re-triggers FetchGoal
    // for the just-deleted id.
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
            <Show when=move || is_loading.get()>
                <BackLink label="Back to Goals" href="/goals".to_string() />
                <SkeletonCardList />
            </Show>

            <Show when=move || !is_loading.get()>
                {move || {
                    match goal.get() {
                        Some(g) => view! { <GoalDetailContent goal=g /> }.into_any(),
                        None => view! {
                            <BackLink label="Back to Goals" href="/goals".to_string() />
                            <p class="text-muted text-sm">"Goal not found."</p>
                        }.into_any(),
                    }
                }}
            </Show>
        </div>
    }
}

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
    let goal_id_for_section = goal.id.clone();
    let goal_id_for_edit_sheet = goal.id.clone();
    let goal_id_for_link_sheet = goal.id.clone();
    let is_completed = goal.status == GoalStatus::Completed;

    let edit_sheet_open = RwSignal::new(false);
    let close_edit_sheet = Callback::new(move |_| edit_sheet_open.set(false));
    let link_sheet_open = RwSignal::new(false);
    let close_link_sheet = Callback::new(move |_| link_sheet_open.set(false));
    let show_delete_confirm = RwSignal::new(false);

    let live_linked_items = {
        let goal_id_for_section = goal_id_for_section.clone();
        Signal::derive(move || {
            view_model.with(|vm| {
                vm.goals
                    .iter()
                    .find(|g| g.id == goal_id_for_section)
                    .map(|g| g.items.clone())
                    .unwrap_or_default()
            })
        })
    };

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

    let title_display = goal
        .title
        .clone()
        .unwrap_or_else(|| "Untitled goal".to_string());

    view! {
        <div class="flex items-center justify-between -mb-2">
            <BackLink label="Back to Goals" href="/goals".to_string() />
            <button
                type="button"
                class="text-sm font-medium text-accent-text hover:text-accent-hover"
                on:click=move |_| edit_sheet_open.set(true)
            >
                "Edit"
            </button>
        </div>

        <div class="space-y-6">
            {move || show_delete_confirm.get().then(|| {
                let goal_id_del = goal_id_delete.clone();
                let core_del = core_for_delete.clone();
                let navigate_del = navigate_delete.clone();
                view! {
                    <div class="danger-callout" role="alert">
                        <p class="text-sm text-danger-text mb-3">
                            "Are you sure you want to delete this goal? This action cannot be undone."
                        </p>
                        <div class="flex gap-3">
                            <Button
                                variant=ButtonVariant::Danger
                                loading=Signal::derive(move || is_submitting.get())
                                on_click=Callback::new(move |_| {
                                    navigate_del("/goals", NavigateOptions { replace: true, ..Default::default() });
                                    let core_ref = core_del.borrow();
                                    let effects = core_ref.process_event(Event::Goal(GoalEvent::Delete {
                                        id: goal_id_del.clone(),
                                    }));
                                    process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                })
                            >
                                {move || if is_submitting.get() { "Deleting\u{2026}" } else { "Confirm Delete" }}
                            </Button>
                            <Button
                                variant=ButtonVariant::Secondary
                                on_click=Callback::new(move |_| { show_delete_confirm.set(false); })
                            >
                                "Cancel"
                            </Button>
                        </div>
                    </div>
                }
            })}

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

            <div class="space-y-2">
                <div class="flex items-center justify-between">
                    <p class="field-label">"Linked items"</p>
                    <button
                        type="button"
                        class="text-sm font-medium text-accent-text hover:text-accent-hover"
                        on:click=move |_| link_sheet_open.set(true)
                    >
                        {move || if live_linked_items.with(|items| items.is_empty()) { "+ Link" } else { "Edit" }}
                    </button>
                </div>
                {move || {
                    let items = live_linked_items.get();
                    if items.is_empty() {
                        view! {
                            <p class="text-sm text-muted italic">"No items linked."</p>
                        }.into_any()
                    } else {
                        view! {
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
                        }.into_any()
                    }
                }}
            </div>

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
                    on_click=Callback::new(move |_| { show_delete_confirm.set(true); })
                    full_width=true
                >
                    "Delete Goal"
                </Button>
            </div>
        </div>

        <EditGoalSheet
            goal_id=goal_id_for_edit_sheet
            open=edit_sheet_open
            on_close=close_edit_sheet
            is_submitting=is_submitting
        />
        <LinkItemsSheet
            goal_id=goal_id_for_link_sheet
            open=link_sheet_open
            on_close=close_link_sheet
        />
    }
}

#[component]
fn LinkItemsSheet(goal_id: String, open: RwSignal<bool>, on_close: Callback<()>) -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    let linked_set: Signal<Arc<HashSet<String>>> = {
        let goal_id = goal_id.clone();
        Signal::derive(move || {
            Arc::new(view_model.with(|vm| {
                vm.goals
                    .iter()
                    .find(|g| g.id == goal_id)
                    .map(|g| g.items.iter().map(|i| i.item_id.clone()).collect())
                    .unwrap_or_default()
            }))
        })
    };

    let on_toggle = {
        let goal_id = goal_id.clone();
        Callback::new(move |item_id: String| {
            let item =
                view_model.with_untracked(|vm| vm.items.iter().find(|i| i.id == item_id).cloned());
            let Some(item) = item else {
                return;
            };

            let is_linked = linked_set.with_untracked(|set| set.contains(&item_id));

            let event = if is_linked {
                Event::Goal(GoalEvent::UnlinkItem {
                    goal_id: goal_id.clone(),
                    item_id: item_id.clone(),
                })
            } else {
                Event::Goal(GoalEvent::LinkItem {
                    goal_id: goal_id.clone(),
                    item: LinkGoalItem {
                        item_id: item.id.clone(),
                        item_title: item.title.clone(),
                        item_type: item.item_type,
                    },
                })
            };

            let core_ref = core.borrow();
            let effects = core_ref.process_event(event);
            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
        })
    };

    view! {
        <BottomSheet
            open=open
            on_close=on_close
            nav_title="Link Items".to_string()
            nav_action_label="Done".to_string()
            on_nav_action=Callback::new(move |_| on_close.run(()))
        >
            {move || {
                let items = view_model.with(|vm| vm.items.clone());
                if items.is_empty() {
                    view! {
                        <p class="text-sm text-muted text-center py-8">"No library items available. Add a piece or exercise first."</p>
                    }.into_any()
                } else {
                    view! {
                        <div class="space-y-2">
                            {items.into_iter().map(|item| {
                                let item_id = item.id.clone();
                                let is_selected = Signal::derive(move || {
                                    linked_set.with(|set| set.contains(&item_id))
                                });
                                view! {
                                    <BuilderItemRow
                                        item=item
                                        is_selected=is_selected
                                        on_toggle=on_toggle
                                    />
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }.into_any()
                }
            }}
        </BottomSheet>
    }
}

#[component]
fn EditGoalSheet(
    goal_id: String,
    open: RwSignal<bool>,
    on_close: Callback<()>,
    is_submitting: IsSubmitting,
) -> impl IntoView {
    let form_ref = NodeRef::<leptos::html::Form>::new();
    let on_save = Callback::new(move |_| {
        if let Some(form) = form_ref.get() {
            let _ = form.request_submit();
        }
    });
    let submitting_signal = Signal::derive(move || is_submitting.get());
    view! {
        <BottomSheet
            open=open
            on_close=on_close
            nav_title="Edit Goal".to_string()
            nav_action_label="Save".to_string()
            on_nav_action=on_save
            nav_action_disabled=submitting_signal
        >
            <GoalEditFormView
                goal_id=goal_id.clone()
                in_sheet=true
                on_dismiss=on_close
                form_ref=form_ref
            />
        </BottomSheet>
    }
}
