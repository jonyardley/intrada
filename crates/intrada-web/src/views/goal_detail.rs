use std::collections::HashSet;
use std::sync::Arc;

use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};
use leptos_router::NavigateOptions;

use intrada_core::domain::types::UpdateGoalItem;
use intrada_core::{
    Event, GoalEvent, GoalItemView, GoalStatus, GoalView, LibraryItemView, LinkGoalItem,
    SessionEvent, ViewModel,
};

use crate::components::{
    AccentBar, AccentRow, BackLink, BottomSheet, BuilderItemRow, Button, ButtonVariant,
    SkeletonCardList,
};
use crate::views::GoalEditFormView;
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Sheet-state signals owned by `GoalDetailView`, not `GoalDetailContent`.
/// The inner content component is rebuilt on every view-model update
/// (its prop changes when the goal mutates); local signals there would
/// reset mid-interaction. Hoisting them to the outer component keeps
/// in-flight sheets open across optimistic updates.
#[derive(Clone, Copy)]
struct SheetState {
    edit_open: RwSignal<bool>,
    link_open: RwSignal<bool>,
    targets_item_id: RwSignal<Option<String>>,
    show_delete_confirm: RwSignal<bool>,
}

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

    let sheets = SheetState {
        edit_open: RwSignal::new(false),
        link_open: RwSignal::new(false),
        targets_item_id: RwSignal::new(None),
        show_delete_confirm: RwSignal::new(false),
    };

    // Reset sheets when the route id changes (navigating between goals).
    let last_route_id = RwSignal::new(String::new());
    Effect::new(move |_| {
        let route_id = params.with(|p| p.get("id").unwrap_or_default().to_string());
        let prev = last_route_id.get_untracked();
        if !prev.is_empty() && prev != route_id {
            sheets.edit_open.set(false);
            sheets.link_open.set(false);
            sheets.targets_item_id.set(None);
            sheets.show_delete_confirm.set(false);
        }
        last_route_id.set(route_id);
    });

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
                        Some(g) => view! { <GoalDetailContent goal=g sheets=sheets /> }.into_any(),
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
fn GoalDetailContent(goal: GoalView, sheets: SheetState) -> impl IntoView {
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
    let goal_id_for_status = goal.id.clone();
    let is_completed_signal = Signal::derive(move || {
        view_model.with(|vm| {
            vm.goals
                .iter()
                .find(|g| g.id == goal_id_for_status)
                .map(|g| g.status == GoalStatus::Completed)
                .unwrap_or(false)
        })
    });

    let edit_sheet_open = sheets.edit_open;
    let close_edit_sheet = Callback::new(move |_| edit_sheet_open.set(false));
    let link_sheet_open = sheets.link_open;
    let close_link_sheet = Callback::new(move |_| link_sheet_open.set(false));
    let targets_sheet_item_id = sheets.targets_item_id;
    let close_targets_sheet = Callback::new(move |_| targets_sheet_item_id.set(None));
    let show_delete_confirm = sheets.show_delete_confirm;

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
    let core_for_reopen = core.clone();
    let core_for_practice = core.clone();
    let goal_id_for_reopen = goal_id.clone();
    let goal_id_for_practice = goal_id.clone();
    let navigate_practice = use_navigate();
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

    // Reopen stays on the goal detail — user is reverting a decision, not
    // finishing a flow, so no nav back to /goals.
    let on_reopen = move |_: leptos::ev::MouseEvent| {
        let core_ref = core_for_reopen.borrow();
        let effects = core_ref.process_event(Event::Goal(GoalEvent::Reopen {
            id: goal_id_for_reopen.clone(),
        }));
        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
    };

    let on_practice = move |_: leptos::ev::MouseEvent| {
        let core_ref = core_for_practice.borrow();
        let effects = core_ref.process_event(Event::Session(SessionEvent::LoadGoalIntoSetlist {
            goal_id: goal_id_for_practice.clone(),
        }));
        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
        drop(core_ref);
        navigate_practice.clone()("/sessions/new", NavigateOptions::default());
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
                    <span class=move || if is_completed_signal.get() { "badge badge--accent" } else { "badge badge--warm" }>
                        {move || if is_completed_signal.get() { "Completed" } else { "Active" }}
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

            {
                let on_practice = Callback::new(on_practice);
                view! {
                    <Show when=move || {
                        !is_completed_signal.get()
                            && live_linked_items.with(|items| !items.is_empty())
                    }>
                        <Button
                            variant=ButtonVariant::Primary
                            on_click=on_practice
                            full_width=true
                        >
                            "Practice this goal"
                        </Button>
                    </Show>
                }
            }

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
                                {items.into_iter().map(|item| {
                                    let item_id_for_tap = item.item_id.clone();
                                    let item_id_for_library = item.item_id.clone();
                                    view! {
                                        <li>
                                            <AccentRow bar=AccentBar::None>
                                                <button
                                                    type="button"
                                                    class="flex-1 min-w-0 text-left appearance-none bg-transparent p-0"
                                                    on:click=move |_| targets_sheet_item_id.set(Some(item_id_for_tap.clone()))
                                                >
                                                    <div class="text-sm text-primary truncate">
                                                        {item.item_title.clone()}
                                                    </div>
                                                    <GoalItemTargetChips item=item.clone() />
                                                </button>
                                                <A
                                                    href=format!("/library/{}", item_id_for_library)
                                                    attr:class="ml-2 px-2 py-1 text-muted hover:text-accent-text no-underline"
                                                    attr:aria-label="Open in library"
                                                >
                                                    "›"
                                                </A>
                                            </AccentRow>
                                        </li>
                                    }
                                }).collect::<Vec<_>>()}
                            </ul>
                        }.into_any()
                    }
                }}
            </div>

            // Actions
            <div class="space-y-3 pt-4 border-t border-border-default">
                {move || (!is_completed_signal.get() && goal_looks_ready(&live_linked_items.get())).then(|| view! {
                    <div class="rounded-lg p-card-compact bg-surface-secondary text-sm text-accent-text">
                        "🎯 All targeted items meet their confidence target. Looks ready — mark complete?"
                    </div>
                })}
                {
                    let on_complete = Callback::new(on_complete);
                    let on_reopen = Callback::new(on_reopen);
                    view! {
                        <Show
                            when=move || is_completed_signal.get()
                            fallback=move || view! {
                                <Button
                                    variant=ButtonVariant::Primary
                                    on_click=on_complete
                                    full_width=true
                                >
                                    "Mark Complete"
                                </Button>
                            }
                        >
                            <Button
                                variant=ButtonVariant::Primary
                                on_click=on_reopen
                                full_width=true
                            >
                                "Reopen Goal"
                            </Button>
                        </Show>
                    }
                }
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
        <GoalItemTargetsSheet
            goal_id=goal.id.clone()
            item_id=targets_sheet_item_id
            on_close=close_targets_sheet
        />
    }
}

// Untargeted items don't gate the suggestion — see specs/goals-targets.md decision #5.
fn goal_looks_ready(items: &[GoalItemView]) -> bool {
    let mut any_targeted = false;
    for item in items {
        if let Some(target) = item.effective_target_confidence {
            any_targeted = true;
            if !item.latest_score.is_some_and(|s| s >= target) {
                return false;
            }
        }
        if let Some(target) = item.effective_target_tempo {
            any_targeted = true;
            if !item.latest_achieved_tempo.is_some_and(|t| t >= target) {
                return false;
            }
        }
    }
    any_targeted
}

#[component]
fn ItemPracticeInfo(library: Signal<Option<LibraryItemView>>) -> impl IntoView {
    view! {
        {move || library.get().map(|item| {
            let composer = (!item.subtitle.is_empty()).then(|| item.subtitle.clone());
            let practice = item.practice.clone();
            let latest_tempo = item.latest_achieved_tempo;
            view! {
                <div class="space-y-1 text-sm text-muted">
                    {composer.map(|c| view! { <div>{c}</div> })}
                    {practice.as_ref().map(|p| {
                        let count = p.session_count;
                        let score = p.latest_score;
                        view! {
                            <div>
                                {format!("{count} session{} • ", if count == 1 { "" } else { "s" })}
                                {match score {
                                    Some(s) => format!("last score {s}/5"),
                                    None => "no score yet".to_string(),
                                }}
                            </div>
                        }
                    })}
                    {latest_tempo.map(|bpm| view! {
                        <div>{format!("Last achieved tempo: {bpm} bpm")}</div>
                    })}
                    {(practice.is_none() && latest_tempo.is_none() && item.subtitle.is_empty()).then(|| view! {
                        <div class="italic">"No practice yet for this item."</div>
                    })}
                </div>
            }
        })}
    }
}

#[component]
fn GoalItemTargetChips(item: GoalItemView) -> impl IntoView {
    let target_date = item.target_date.clone();
    let effective_confidence = item.effective_target_confidence;
    let effective_tempo = item.effective_target_tempo;
    let latest_score = item.latest_score;
    let latest_tempo = item.latest_achieved_tempo;
    let has_any =
        target_date.is_some() || effective_confidence.is_some() || effective_tempo.is_some();

    if !has_any {
        return view! {
            <div class="text-xs text-muted mt-0.5 italic">"No targets set"</div>
        }
        .into_any();
    }

    view! {
        <div class="flex flex-wrap gap-1 mt-1">
            {target_date.map(|d| view! {
                <span class="badge badge--muted">{format!("📅 {d}")}</span>
            })}
            {effective_confidence.map(|target| {
                let met = latest_score.is_some_and(|s| s >= target);
                let badge_class = if met { "badge badge--accent" } else { "badge badge--muted" };
                let label = match latest_score {
                    Some(score) => format!("Confidence {score}/{target}"),
                    None => format!("Confidence —/{target}"),
                };
                view! { <span class=badge_class>{label}</span> }
            })}
            {effective_tempo.map(|target| {
                let met = latest_tempo.is_some_and(|t| t >= target);
                let badge_class = if met { "badge badge--accent" } else { "badge badge--muted" };
                let label = match latest_tempo {
                    Some(bpm) => format!("Tempo {bpm}/{target} bpm"),
                    None => format!("Tempo —/{target} bpm"),
                };
                view! { <span class=badge_class>{label}</span> }
            })}
        </div>
    }
    .into_any()
}

#[component]
fn GoalItemTargetsSheet(
    goal_id: String,
    item_id: RwSignal<Option<String>>,
    on_close: Callback<()>,
) -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    let open = RwSignal::new(false);
    Effect::new(move |_| {
        open.set(item_id.with(|i| i.is_some()));
    });

    let target_date = RwSignal::new(String::new());
    let target_confidence = RwSignal::new(String::new());
    let target_tempo = RwSignal::new(String::new());

    let goal_id_for_effect = goal_id.clone();
    Effect::new(move |_| {
        let id = item_id.with(|i| i.clone());
        let Some(id) = id else {
            return;
        };
        let item = view_model.with_untracked(|vm| {
            vm.goals
                .iter()
                .find(|g| g.id == goal_id_for_effect)
                .and_then(|g| g.items.iter().find(|i| i.item_id == id).cloned())
        });
        if let Some(item) = item {
            target_date.set(item.target_date.clone().unwrap_or_default());
            target_confidence.set(
                item.target_confidence
                    .map(|c| c.to_string())
                    .unwrap_or_default(),
            );
            target_tempo.set(item.target_tempo.map(|t| t.to_string()).unwrap_or_default());
        }
    });

    let goal_id_for_view = goal_id.clone();
    let item_view = Signal::derive(move || {
        let id = item_id.with(|i| i.clone())?;
        view_model.with(|vm| {
            vm.goals
                .iter()
                .find(|g| g.id == goal_id_for_view)
                .and_then(|g| g.items.iter().find(|i| i.item_id == id).cloned())
        })
    });

    let library_item = Signal::derive(move || {
        let id = item_id.with(|i| i.clone())?;
        view_model.with(|vm| vm.items.iter().find(|i| i.id == id).cloned())
    });

    let goal_id_for_save = goal_id;
    // Full overwrite on Save: both fields are initialised from the item on
    // open, so wrapping each in Some(_) sends the form's current state
    // (including explicit clears as Some(None)). No per-field dirty
    // tracking — form is the single source of truth.
    let on_save = Callback::new(move |_| {
        let Some(id) = item_id.with_untracked(|i| i.clone()) else {
            return;
        };
        let td = target_date.get_untracked();
        let tc = target_confidence.get_untracked();
        let tt = target_tempo.get_untracked();

        let target_date_val = if td.is_empty() { None } else { Some(td) };
        let target_confidence_val: Option<u8> = if tc.is_empty() { None } else { tc.parse().ok() };
        let target_tempo_val: Option<u16> = if tt.is_empty() { None } else { tt.parse().ok() };

        let input = UpdateGoalItem {
            target_date: Some(target_date_val),
            target_confidence: Some(target_confidence_val),
            target_tempo: Some(target_tempo_val),
        };

        let core_ref = core.borrow();
        let effects = core_ref.process_event(Event::Goal(GoalEvent::UpdateGoalItemTargets {
            goal_id: goal_id_for_save.clone(),
            item_id: id,
            input,
        }));
        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
        on_close.run(());
    });

    let submitting_signal = Signal::derive(move || is_submitting.get());

    view! {
        <BottomSheet
            open=open
            on_close=on_close
            nav_title="Item Targets".to_string()
            nav_action_label="Save".to_string()
            on_nav_action=on_save
            nav_action_disabled=submitting_signal
        >
            {move || {
                let item = item_view.get();
                match item {
                    None => view! { <div></div> }.into_any(),
                    Some(item) => view! {
                        <div class="space-y-4">
                            <div class="space-y-1">
                                <p class="field-label">"Item"</p>
                                <p class="text-base text-primary">{item.item_title.clone()}</p>
                                <ItemPracticeInfo library=library_item />
                            </div>

                            <div class="space-y-1">
                                <label for="goal-item-target-date" class="form-label">
                                    "Target date"
                                </label>
                                <input
                                    id="goal-item-target-date"
                                    type="date"
                                    class="input-base"
                                    bind:value=target_date
                                />
                                <p class="text-xs text-muted">
                                    "When you want this piece ready. Overrides the goal's deadline for this item."
                                </p>
                            </div>

                            <div class="space-y-1">
                                <label for="goal-item-target-confidence" class="form-label">
                                    "Target confidence"
                                </label>
                                <select
                                    id="goal-item-target-confidence"
                                    class="input-base"
                                    prop:value=move || target_confidence.get()
                                    on:change=move |ev| target_confidence.set(leptos::prelude::event_target_value(&ev))
                                >
                                    <option value="">"(use goal default)"</option>
                                    <option value="1">"1 — Just starting"</option>
                                    <option value="2">"2"</option>
                                    <option value="3">"3 — Comfortable"</option>
                                    <option value="4">"4"</option>
                                    <option value="5">"5 — Performance ready"</option>
                                </select>
                            </div>

                            <div class="space-y-1">
                                <label for="goal-item-target-tempo" class="form-label">
                                    "Target tempo (BPM)"
                                </label>
                                <input
                                    id="goal-item-target-tempo"
                                    class="input-base"
                                    type="number"
                                    inputmode="numeric"
                                    min="20"
                                    max="400"
                                    placeholder="(use goal default)"
                                    prop:value=move || target_tempo.get()
                                    on:input=move |ev| target_tempo.set(leptos::prelude::event_target_value(&ev))
                                />
                            </div>
                        </div>
                    }.into_any()
                }
            }}
        </BottomSheet>
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
                        target_date: None,
                        target_confidence: None,
                        target_tempo: None,
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
