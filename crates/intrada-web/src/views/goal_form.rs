use std::collections::HashMap;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::{CreateGoal, Event, GoalEvent, ViewModel};

use crate::components::{BackLink, Button, ButtonVariant, PageHeading, TextArea, TextField};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Goal creation form.
///
/// Renders fields for title (optional), notes, and deadline, then dispatches
/// `GoalEvent::Add` on submit. Navigates back to `/goals` on success.
#[component]
pub fn GoalFormView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let navigate = use_navigate();

    // Form state
    let title = RwSignal::new(String::new());
    let notes = RwSignal::new(String::new());
    let deadline = RwSignal::new(String::new());
    let errors: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        // Build today's date for the `date` field
        let today = js_sys::Date::new_0();
        let date = format!(
            "{}-{:02}-{:02}",
            today.get_full_year(),
            today.get_month() + 1,
            today.get_date()
        );

        let title_val = title.get();
        let notes_val = notes.get();
        let deadline_val = deadline.get();

        let input = CreateGoal {
            date,
            title: if title_val.is_empty() {
                None
            } else {
                Some(title_val)
            },
            notes: if notes_val.is_empty() {
                None
            } else {
                Some(notes_val)
            },
            deadline: if deadline_val.is_empty() {
                None
            } else {
                Some(deadline_val)
            },
        };

        let core_ref = core.borrow();
        let effects = core_ref.process_event(Event::Goal(GoalEvent::Add(input)));
        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);

        // Check for errors from validation
        let vm = view_model.get_untracked();
        if let Some(err) = vm.error {
            errors.set(HashMap::from([("title".to_string(), err)]));
        } else {
            let nav = navigate.clone();
            nav("/goals", NavigateOptions::default());
        }
    };

    view! {
        <div class="space-y-4">
            <BackLink label="Back to Goals" href="/goals".to_string() />
            <PageHeading text="New Goal" />

            <form class="space-y-4" on:submit=on_submit>
                <TextField
                    id="goal-title"
                    label="Title"
                    value=title
                    field_name="title"
                    errors=errors
                    placeholder="e.g. Prep for Tuesday lesson"
                    input_type="text"
                />

                <TextArea
                    id="goal-notes"
                    label="Notes"
                    value=notes
                    rows=4
                    field_name="notes"
                    errors=errors
                />

                <TextField
                    id="goal-deadline"
                    label="Deadline"
                    value=deadline
                    field_name="deadline"
                    errors=errors
                    input_type="date"
                />

                <Button
                    variant=ButtonVariant::Primary
                    full_width=true
                    button_type="submit"
                >
                    "Save Goal"
                </Button>
            </form>
        </div>
    }
}
