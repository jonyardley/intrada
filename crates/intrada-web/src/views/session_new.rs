use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::{Event, SessionEvent, SessionStatusView, ViewModel};

use crate::components::{BackLink, PageHeading, SetlistBuilder};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Session creation view: goes straight into the setlist builder.
///
/// If an active session already exists (e.g. from crash recovery), redirects
/// to `/sessions/active` without showing a resume/discard prompt.
#[component]
pub fn SessionNewView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let navigate = use_navigate();

    let started_building = RwSignal::new(false);

    // Single Effect handles all session_status transitions:
    // - Idle (first mount): auto-start building
    // - Idle (after cancel/abandon): navigate back to list
    // - Active (crash recovery or just started): navigate to active session
    // - Summary (previous session not yet saved/discarded): redirect to
    //   /sessions/summary so the user resolves it before starting a new one.
    //   Without this branch the view renders an empty page (#503).
    Effect::new({
        let core = core.clone();
        move |_| {
            let vm = view_model.get();
            match vm.session_status {
                SessionStatusView::Active => {
                    navigate(
                        "/sessions/active",
                        NavigateOptions {
                            replace: true,
                            ..Default::default()
                        },
                    );
                }
                SessionStatusView::Summary => {
                    navigate(
                        "/sessions/summary",
                        NavigateOptions {
                            replace: true,
                            ..Default::default()
                        },
                    );
                }
                SessionStatusView::Idle if !started_building.get_untracked() => {
                    let core_ref = core.borrow();
                    let effects =
                        core_ref.process_event(Event::Session(SessionEvent::StartBuilding));
                    process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                    started_building.set(true);
                }
                SessionStatusView::Idle => {
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
        }
    });

    view! {
        <div>
            <BackLink label="Back to Practice" href="/sessions".to_string() />
            <PageHeading text="New Session" />

            <Show when=move || view_model.get().session_status == SessionStatusView::Building>
                <SetlistBuilder />
            </Show>
        </div>
    }
}
