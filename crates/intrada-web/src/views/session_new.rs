use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::{Event, SessionEvent, ViewModel};

use crate::components::{BackLink, PageHeading, SetlistBuilder};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::SharedCore;

/// Session creation view: start building a setlist and launch practice.
#[component]
pub fn SessionNewView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let navigate = use_navigate();

    // Dispatch StartBuilding on mount if not already building
    {
        let vm = view_model.get_untracked();
        if vm.session_status != "building" {
            let core_ref = core.borrow();
            let effects = core_ref.process_event(Event::Session(SessionEvent::StartBuilding));
            process_effects(&core_ref, effects, &view_model);
        }
    }

    // Watch session_status to navigate on transitions
    Effect::new(move |_| {
        let vm = view_model.get();
        match vm.session_status.as_str() {
            "active" => {
                navigate(
                    "/sessions/active",
                    NavigateOptions {
                        replace: true,
                        ..Default::default()
                    },
                );
            }
            "idle" => {
                // If building was cancelled, go back to sessions
                // But don't navigate on initial mount — only if we were building
            }
            _ => {}
        }
    });

    view! {
        <div>
            <BackLink label="Back to Sessions" href="/sessions".to_string() />
            <PageHeading text="New Practice Session" />
            <SetlistBuilder />
        </div>
    }
}
