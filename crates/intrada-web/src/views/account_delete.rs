use std::collections::HashMap;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::{AccountEvent, Event, ViewModel};
use intrada_web::clerk_bindings;
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

use crate::components::{Button, ButtonVariant, TextField};

const CONFIRM_PHRASE: &str = "delete my account";

/// Full-screen typed-confirmation flow for permanent account deletion.
///
/// Lists what will be erased, requires an exact-string confirmation, then
/// dispatches `AccountEvent::DeleteAccount`. Sign-out + navigation are
/// triggered by an `Effect` watching the model's `account_deleted` flag,
/// which only flips on the `AccountDeleted` event — i.e. after the
/// server has confirmed the deletion, not on dispatch.
#[component]
pub fn AccountDeleteView() -> impl IntoView {
    let core = expect_context::<SharedCore>();
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    let confirmation = RwSignal::new(String::new());
    let errors = RwSignal::new(HashMap::<String, String>::new());

    let confirmation_matches = Signal::derive(move || confirmation.get() == CONFIRM_PHRASE);
    let delete_in_flight = Signal::derive(move || view_model.get().delete_in_flight);

    // Watch the model's terminal `account_deleted` flag — set ONLY in the
    // `AccountDeleted` handler, never on dispatch — and run sign-out +
    // navigation once when it flips true. A local guard prevents the
    // spawned closure firing twice if the effect re-runs.
    let navigated = RwSignal::new(false);
    let navigate = use_navigate();
    Effect::new(move |_| {
        let vm = view_model.get();
        if let Some(err) = vm.error.clone() {
            if !err.is_empty() {
                errors.set(HashMap::from([("confirmation".to_string(), err)]));
            }
        }
        if !vm.account_deleted || navigated.get_untracked() {
            return;
        }
        navigated.set(true);
        let navigate = navigate.clone();
        leptos::task::spawn_local(async move {
            clerk_bindings::sign_out().await;
            navigate(
                "/",
                NavigateOptions {
                    replace: true,
                    ..Default::default()
                },
            );
        });
    });

    let core_for_delete = core.clone();
    let on_delete = Callback::new(move |_: leptos::ev::MouseEvent| {
        if !confirmation_matches.get() {
            return;
        }
        errors.set(HashMap::new());
        let core_ref = core_for_delete.borrow();
        let effects = core_ref.process_event(Event::Account(AccountEvent::DeleteAccount));
        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
    });

    let cancel_navigate = use_navigate();
    let on_cancel = move |_| {
        cancel_navigate("/", NavigateOptions::default());
    };

    view! {
        <div class="max-w-md mx-auto py-comfortable space-y-comfortable pb-[env(safe-area-inset-bottom)]">
            <h1 class="page-title">"Delete your account?"</h1>

            <p class="text-sm text-secondary">
                "This is permanent. All of the following will be erased:"
            </p>

            <ul class="list-disc pl-card text-sm text-secondary space-y-1">
                <li>"Your library (pieces and exercises)"</li>
                <li>"Your practice sessions and analytics"</li>
                <li>"Your routines and sets"</li>
                <li>"Your lessons and lesson photos"</li>
                <li>"Your sign-in account"</li>
            </ul>

            <p class="text-sm font-medium text-danger-text">"This cannot be undone."</p>

            <div class="space-y-card">
                <TextField
                    id="confirm-delete"
                    label="Type 'delete my account' to confirm"
                    value=confirmation
                    field_name="confirmation"
                    errors=errors
                    placeholder="delete my account"
                    input_type="text"
                />
            </div>

            <div class="flex flex-col gap-card-compact">
                <Button
                    variant=ButtonVariant::Danger
                    on_click=on_delete
                    disabled=Signal::derive(move || !confirmation_matches.get())
                    loading=delete_in_flight
                    full_width=true
                >
                    "Delete account"
                </Button>
                <Button
                    variant=ButtonVariant::Secondary
                    on_click=Callback::new(on_cancel)
                    full_width=true
                >
                    "Cancel"
                </Button>
            </div>
        </div>
    }
}
