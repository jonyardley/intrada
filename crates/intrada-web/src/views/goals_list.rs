use leptos::prelude::*;
use leptos_router::components::A;

use intrada_core::{GoalStatus, GoalView, ViewModel};

use crate::components::{
    AccentBar, AccentRow, BottomSheet, EmptyState, IconName, PageAddButton, PageHeading,
    SkeletonCardList,
};
use crate::views::GoalFormView;
use intrada_web::haptics::haptic_selection;
use intrada_web::types::{IsLoading, IsSubmitting};

#[component]
pub fn GoalsListView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    let active_tab = RwSignal::new(GoalStatus::Active);
    let add_sheet_open = RwSignal::new(false);
    let open_add_sheet = Callback::new(move |_| add_sheet_open.set(true));
    let close_add_sheet = Callback::new(move |_| add_sheet_open.set(false));

    let filtered_goals = Memo::new(move |_| {
        let vm = view_model.get();
        let tab = active_tab.get();
        vm.goals
            .into_iter()
            .filter(|g| g.status == tab)
            .collect::<Vec<_>>()
    });

    let is_active_tab = move || active_tab.get() == GoalStatus::Active;

    let set_tab = move |status: GoalStatus| {
        if active_tab.get_untracked() == status {
            return;
        }
        haptic_selection();
        active_tab.set(status);
    };

    let active_tab_class = move || {
        if is_active_tab() {
            "tabs-underline-btn tabs-underline-btn--active"
        } else {
            "tabs-underline-btn"
        }
    };
    let completed_tab_class = move || {
        if !is_active_tab() {
            "tabs-underline-btn tabs-underline-btn--active"
        } else {
            "tabs-underline-btn"
        }
    };

    let indicator_style = move || {
        let pct = if is_active_tab() { 0 } else { 100 };
        format!("--tab-count: 2; --thumb-x: {pct}%")
    };

    view! {
        <div class="space-y-4">
            <PageHeading
                text="Goals"
                trailing=Box::new(move || view! {
                    <PageAddButton
                        aria_label="New Goal"
                        on_click=Callback::new(move |_| open_add_sheet.run(()))
                    />
                }.into_any())
            />

            <div
                class="tabs-underline"
                role="tablist"
                aria-label="Filter goals by status"
                style=indicator_style
            >
                <div class="tabs-underline-indicator" aria-hidden="true" />
                <button
                    type="button"
                    role="tab"
                    aria-selected=move || if is_active_tab() { "true" } else { "false" }
                    aria-controls="goals-list"
                    tabindex=move || if is_active_tab() { "0" } else { "-1" }
                    class=active_tab_class
                    on:click=move |_| set_tab(GoalStatus::Active)
                >
                    "Active"
                </button>
                <button
                    type="button"
                    role="tab"
                    aria-selected=move || if !is_active_tab() { "true" } else { "false" }
                    aria-controls="goals-list"
                    tabindex=move || if !is_active_tab() { "0" } else { "-1" }
                    class=completed_tab_class
                    on:click=move |_| set_tab(GoalStatus::Completed)
                >
                    "Completed"
                </button>
            </div>

            <section id="goals-list" aria-label="Goals">
                <Show when=move || is_loading.get()>
                    <SkeletonCardList />
                </Show>

                <Show when=move || !is_loading.get()>
                    {move || {
                        if filtered_goals.with(|g| g.is_empty()) {
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
                                <ul class="space-y-2 list-none p-0" role="list">
                                    <For
                                        each=move || filtered_goals.get()
                                        key=|g| g.id.clone()
                                        let:goal
                                    >
                                        <li><GoalCard goal=goal /></li>
                                    </For>
                                </ul>
                            }.into_any()
                        }
                    }}
                </Show>
            </section>
        </div>

        <AddGoalSheet
            open=add_sheet_open
            on_close=close_add_sheet
            is_submitting=is_submitting
        />
    }
}

#[component]
fn AddGoalSheet(
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
            nav_title="New Goal".to_string()
            nav_action_label="Save".to_string()
            on_nav_action=on_save
            nav_action_disabled=submitting_signal
        >
            <GoalFormView in_sheet=true on_dismiss=on_close form_ref=form_ref />
        </BottomSheet>
    }
}

#[component]
fn GoalCard(goal: GoalView) -> impl IntoView {
    let href = format!("/goals/{}", goal.id);
    let is_completed = goal.status == GoalStatus::Completed;

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

    let title_class = if is_completed {
        "text-sm font-medium text-muted line-through truncate"
    } else if has_title {
        "text-sm font-semibold text-primary truncate"
    } else {
        "text-sm italic text-secondary truncate"
    };

    view! {
        <A href=href attr:class="no-underline">
            <AccentRow bar=AccentBar::None>
                <div class="flex-1 min-w-0 text-left">
                    <div class=title_class>{display_title}</div>
                </div>
                {deadline.map(|d| {
                    let badge_class = if is_overdue {
                        "badge badge--warning"
                    } else {
                        "badge badge--muted"
                    };
                    view! {
                        <span class=badge_class>{format_deadline(&d)}</span>
                    }
                })}
            </AccentRow>
        </A>
    }
}

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
