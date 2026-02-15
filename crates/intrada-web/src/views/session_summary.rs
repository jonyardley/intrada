use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::ViewModel;

use crate::components::{PageHeading, SessionSummary};
use intrada_web::types::SharedCore;

/// Session summary view: wraps SessionSummary, redirects after save/discard.
#[component]
pub fn SessionSummaryView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let _core = expect_context::<SharedCore>();
    let navigate = use_navigate();

    // Redirect if no summary state
    {
        let vm = view_model.get_untracked();
        if vm.session_status != "summary" {
            navigate(
                "/sessions",
                NavigateOptions {
                    replace: true,
                    ..Default::default()
                },
            );
        }
    }

    // Watch for state transitions (save/discard → idle)
    Effect::new(move |_| {
        let vm = view_model.get();
        if vm.session_status == "idle" {
            navigate(
                "/sessions",
                NavigateOptions {
                    replace: true,
                    ..Default::default()
                },
            );
        }
    });

    view! {
        <div>
            <PageHeading text="Session Summary" />
            <SessionSummary />
        </div>
    }
}
