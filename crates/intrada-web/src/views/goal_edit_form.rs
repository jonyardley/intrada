use std::collections::HashMap;

use leptos::ev;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};
use leptos_router::NavigateOptions;

use intrada_core::{Event, GoalEvent, UpdateGoal, ViewModel};

use crate::components::{
    use_toast, BackLink, Button, ButtonSize, ButtonVariant, Card, PageHeading, SkeletonBlock,
    SkeletonLine, TextArea, TextField,
};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

#[component]
pub fn GoalEditFormView(
    #[prop(optional, into)] goal_id: Option<String>,
    #[prop(optional)] in_sheet: bool,
    #[prop(optional, into)] on_dismiss: Option<Callback<()>>,
    #[prop(optional, into)] form_ref: Option<NodeRef<leptos::html::Form>>,
) -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let toast = use_toast();
    let params = use_params_map();
    let id = goal_id.unwrap_or_else(|| params.read_untracked().get("id").unwrap_or_default());
    let navigate = use_navigate();

    let goal = view_model.with_untracked(|vm| {
        vm.current_goal
            .as_ref()
            .filter(|g| g.id == id)
            .cloned()
            .or_else(|| vm.goals.iter().find(|g| g.id == id).cloned())
    });

    if goal.is_none() {
        // Skeleton-then-self-redirect: covers SPA navigation where the
        // goal lands in the view-model a tick after mount. Hard-reload
        // deep-link is intentionally unsupported here (no FetchGoal
        // dispatch on this route — matches Library's EditLibraryItemForm).
        let id_for_loading = id.clone();
        let loading_inner = move || {
            if is_loading.get() {
                view! {
                    <div class="space-y-4 animate-pulse">
                        <SkeletonLine width="w-1/3" height="h-6" />
                        <SkeletonLine width="w-full" height="h-10" />
                        <SkeletonBlock height="h-24" />
                        <SkeletonLine width="w-2/3" height="h-10" />
                    </div>
                }
                .into_any()
            } else {
                let id = id_for_loading.clone();
                let found = view_model.with(|vm| {
                    vm.current_goal.as_ref().is_some_and(|g| g.id == id)
                        || vm.goals.iter().any(|g| g.id == id)
                });
                if found && !in_sheet {
                    let url = format!("/goals/{}/edit", id);
                    let navigate = use_navigate();
                    navigate(
                        &url,
                        NavigateOptions {
                            replace: true,
                            ..Default::default()
                        },
                    );
                    ().into_any()
                } else {
                    view! {
                        <div class="text-center py-8">
                            <p class="text-secondary mb-4">"Goal not found."</p>
                            <A href="/goals" attr:class="text-accent-text hover:text-accent-hover font-medium">
                                "← Back to Goals"
                            </A>
                        </div>
                    }
                    .into_any()
                }
            }
        };
        if in_sheet {
            return view! { <div>{loading_inner}</div> }.into_any();
        }
        return view! {
            <div class="sm:max-w-2xl sm:mx-auto">
                <BackLink label="Cancel" href="/goals".to_string() />
                <PageHeading text="Edit Goal" />
                <Card>{loading_inner}</Card>
            </div>
        }
        .into_any();
    }

    let goal = goal.expect("goal confirmed Some above");
    let goal_id_val = goal.id.clone();
    let back_href = format!("/goals/{}", goal_id_val);

    let title = RwSignal::new(goal.title.clone().unwrap_or_default());
    let notes = RwSignal::new(goal.notes.clone().unwrap_or_default());
    let deadline = RwSignal::new(goal.deadline.clone().unwrap_or_default());
    let target_confidence = RwSignal::new(
        goal.target_confidence
            .map(|c| c.to_string())
            .unwrap_or_default(),
    );
    let errors: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());

    let navigate_cancel = navigate.clone();
    let cancel_href = back_href.clone();

    let form_ref = form_ref.unwrap_or_default();

    let form_body = view! {
        <form
            node_ref=form_ref
            class="space-y-4"
            on:submit={
                let goal_id_val = goal_id_val.clone();
                let back_href = back_href.clone();
                let navigate = navigate.clone();
                move |ev: ev::SubmitEvent| {
                    ev.prevent_default();

                    let title_val = title.get().trim().to_string();
                    let notes_val = notes.get().trim().to_string();
                    let deadline_val = deadline.get().trim().to_string();

                    let tc_str = target_confidence.get();
                    let tc_parsed: Option<u8> = if tc_str.is_empty() {
                        None
                    } else {
                        tc_str.parse().ok()
                    };

                    let input = UpdateGoal {
                        title: Some(if title_val.is_empty() { None } else { Some(title_val) }),
                        date: None,
                        notes: Some(if notes_val.is_empty() { None } else { Some(notes_val) }),
                        deadline: Some(if deadline_val.is_empty() { None } else { Some(deadline_val) }),
                        status: None,
                        target_confidence: Some(tc_parsed),
                    };

                    let core_ref = core.borrow();
                    let effects = core_ref.process_event(Event::Goal(GoalEvent::Update {
                        id: goal_id_val.clone(),
                        input,
                    }));
                    process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);

                    let vm = view_model.get_untracked();
                    if let Some(err) = vm.error {
                        toast.show(&err);
                        return;
                    }
                    errors.set(HashMap::new());
                    toast.show("Goal updated");
                    if let Some(cb) = on_dismiss {
                        cb.run(());
                    } else {
                        navigate(&back_href, NavigateOptions { replace: true, ..Default::default() });
                    }
                }
            }
        >
            <TextField
                id="goal-edit-title"
                label="Title"
                value=title
                field_name="title"
                errors=errors
                placeholder="e.g. Prep for Tuesday lesson"
                input_type="text"
            />

            <TextArea
                id="goal-edit-notes"
                label="Notes"
                value=notes
                rows=4
                field_name="notes"
                errors=errors
            />

            <TextField
                id="goal-edit-deadline"
                label="Deadline"
                value=deadline
                field_name="deadline"
                errors=errors
                input_type="date"
            />

            <div class="space-y-1">
                <label for="goal-edit-target-confidence" class="form-label">
                    "Target confidence"
                </label>
                <select
                    id="goal-edit-target-confidence"
                    class="input-base"
                    prop:value=move || target_confidence.get()
                    on:change=move |ev| target_confidence.set(leptos::prelude::event_target_value(&ev))
                >
                    <option value="">"(no target)"</option>
                    <option value="1">"1 — Just starting"</option>
                    <option value="2">"2"</option>
                    <option value="3">"3 — Comfortable"</option>
                    <option value="4">"4"</option>
                    <option value="5">"5 — Performance ready"</option>
                </select>
                <p class="text-xs text-muted">
                    "Default target for items in this goal. Individual items can override."
                </p>
            </div>

            <div class="flex flex-col pt-2">
                <Button
                    variant=ButtonVariant::Primary
                    button_type="submit"
                    size=ButtonSize::Hero
                    full_width=true
                    loading=Signal::derive(move || is_submitting.get())
                >
                    {move || if is_submitting.get() { "Saving\u{2026}" } else { "Save Changes" }}
                </Button>
                {(!in_sheet).then(|| view! {
                    <div class="mt-3">
                        <Button variant=ButtonVariant::Secondary full_width=true on_click={
                            let cancel_href = cancel_href.clone();
                            let navigate_cancel = navigate_cancel.clone();
                            Callback::new(move |_| {
                                if let Some(cb) = on_dismiss {
                                    cb.run(());
                                } else {
                                    navigate_cancel(&cancel_href, NavigateOptions::default());
                                }
                            })
                        }>"Cancel"</Button>
                    </div>
                })}
            </div>
        </form>
    };

    if in_sheet {
        form_body.into_any()
    } else {
        view! {
            <div class="sm:max-w-2xl sm:mx-auto">
                <BackLink label="Cancel" href=back_href />
                <PageHeading text="Edit Goal" />
                <Card>{form_body}</Card>
            </div>
        }
        .into_any()
    }
}
