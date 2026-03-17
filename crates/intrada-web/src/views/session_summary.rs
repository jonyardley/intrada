use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::{SessionStatusView, ViewModel};

use crate::components::{PageHeading, SessionSummary};

/// Session summary view: wraps SessionSummary, redirects after save/discard.
#[component]
pub fn SessionSummaryView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let navigate = use_navigate();

    // Redirect if no summary state
    {
        let vm = view_model.get_untracked();
        if vm.session_status != SessionStatusView::Summary {
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
        if vm.session_status == SessionStatusView::Idle {
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
