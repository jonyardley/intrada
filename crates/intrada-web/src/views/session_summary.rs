use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::{SessionStatusView, ViewModel};

use crate::components::SessionSummary;

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

    // No <PageHeading> wrapper here — the SessionSummary component renders
    // its own "Session Complete" hero header (Pencil intent: a single
    // celebratory anchor rather than a generic page title above it).
    view! { <SessionSummary /> }
}
