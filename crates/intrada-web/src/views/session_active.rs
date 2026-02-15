use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::ViewModel;

use crate::components::{PageHeading, SessionTimer};
use intrada_web::types::SharedCore;

/// Active session view: wraps the SessionTimer, redirects when session state changes.
#[component]
pub fn SessionActiveView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let _core = expect_context::<SharedCore>();
    let navigate = use_navigate();

    // Redirect if no active session
    {
        let vm = view_model.get_untracked();
        if vm.session_status != "active" {
            navigate(
                "/sessions/new",
                NavigateOptions {
                    replace: true,
                    ..Default::default()
                },
            );
        }
    }

    // Watch for state transitions
    Effect::new(move |_| {
        let vm = view_model.get();
        match vm.session_status.as_str() {
            "summary" => {
                navigate(
                    "/sessions/summary",
                    NavigateOptions {
                        replace: true,
                        ..Default::default()
                    },
                );
            }
            "idle" => {
                navigate(
                    "/sessions",
                    NavigateOptions {
                        replace: true,
                        ..Default::default()
                    },
                );
            }
            _ => {}
        }
    });

    view! {
        <div>
            <PageHeading text="Practice Session" />
            <SessionTimer />
        </div>
    }
}
