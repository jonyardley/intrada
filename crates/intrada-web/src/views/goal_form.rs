use std::collections::HashMap;

use leptos::ev;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::{CreateGoal, Event, GoalEvent, ViewModel};

use crate::components::{
    use_toast, BackLink, Button, ButtonSize, ButtonVariant, Card, PageHeading, TextArea, TextField,
};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Goal creation form.
///
/// Mirrors `AddLibraryItemForm`: renders the same body in both a standalone
/// `/goals/new` route (with back-link + page heading + card chrome) and
/// inside a `BottomSheet` from the goals list (the sheet supplies its own
/// nav chrome).
#[component]
pub fn GoalFormView(
    /// When rendered inside a BottomSheet (vs as a standalone route), drop
    /// the back-link / page heading / card chrome — the sheet provides its
    /// own. Cancel + Save call `on_dismiss` instead of navigating.
    #[prop(optional)]
    in_sheet: bool,
    /// Fired when the user successfully saves or cancels. Required when
    /// `in_sheet` is true; ignored otherwise (route mode navigates instead).
    #[prop(optional, into)]
    on_dismiss: Option<Callback<()>>,
    /// Optional ref to the underlying `<form>` element so the sheet's
    /// nav-bar Save can trigger `requestSubmit()` on it.
    #[prop(optional, into)]
    form_ref: Option<NodeRef<leptos::html::Form>>,
) -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let toast = use_toast();
    let navigate = use_navigate();
    let navigate_cancel = navigate.clone();

    let title = RwSignal::new(String::new());
    let notes = RwSignal::new(String::new());
    let deadline = RwSignal::new(String::new());
    let errors: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());

    let dismiss_save = on_dismiss;
    let dismiss_cancel = on_dismiss;

    let form_ref = form_ref.unwrap_or_default();

    let form_body = view! {
        <form
            node_ref=form_ref
            class="space-y-4"
            on:submit=move |ev: ev::SubmitEvent| {
                ev.prevent_default();

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
                    title: if title_val.is_empty() { None } else { Some(title_val) },
                    notes: if notes_val.is_empty() { None } else { Some(notes_val) },
                    deadline: if deadline_val.is_empty() { None } else { Some(deadline_val) },
                };

                let core_ref = core.borrow();
                let effects = core_ref.process_event(Event::Goal(GoalEvent::Add(input)));
                process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);

                let vm = view_model.get_untracked();
                if let Some(err) = vm.error {
                    errors.set(HashMap::from([("title".to_string(), err)]));
                    return;
                }
                errors.set(HashMap::new());
                toast.show("Goal added");
                if let Some(cb) = dismiss_save {
                    cb.run(());
                } else {
                    navigate("/goals", NavigateOptions { replace: true, ..Default::default() });
                }
            }
        >
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

            <div class="flex flex-col pt-2">
                <Button
                    variant=ButtonVariant::Primary
                    button_type="submit"
                    size=ButtonSize::Hero
                    full_width=true
                    loading=Signal::derive(move || is_submitting.get())
                >
                    {move || if is_submitting.get() { "Saving\u{2026}" } else { "Save Goal" }}
                </Button>
                {(!in_sheet).then(|| view! {
                    <div class="mt-3">
                        <Button variant=ButtonVariant::Secondary full_width=true on_click=Callback::new(move |_| {
                            if let Some(cb) = dismiss_cancel {
                                cb.run(());
                            } else {
                                navigate_cancel("/goals", NavigateOptions::default());
                            }
                        })>"Cancel"</Button>
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
                <BackLink label="Cancel" href="/goals".to_string() />
                <PageHeading text="New Goal" />
                <Card>{form_body}</Card>
            </div>
        }
        .into_any()
    }
}
