use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::{Event, SessionEvent, SessionStatusView, ViewModel};

use crate::components::{BackLink, Button, ButtonVariant, Card, PageHeading, SetlistBuilder};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Session creation view: start building a setlist and launch practice.
///
/// If an active session already exists (e.g. from crash recovery or navigating
/// away), shows a banner with Resume / Discard options instead of the builder.
#[component]
pub fn SessionNewView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let navigate = use_navigate();

    // Track whether we entered building state on this mount.
    // When true, an "active" status means StartSession was just clicked and we
    // should navigate to the active view. When false, an "active" status means
    // there was a pre-existing session and we show recovery UI instead.
    let started_building = RwSignal::new(false);

    // Dispatch StartBuilding on mount — but only from idle state.
    // If an active session exists we leave it alone and show recovery UI.
    // Note: ViewModel::default() has session_status "" (empty string) which
    // occurs when navigating directly to this page before async data loads.
    // The underlying Model defaults to SessionStatus::Idle, so both "" and
    // "idle" are safe to treat as idle here.
    {
        let vm = view_model.get_untracked();
        if vm.session_status == SessionStatusView::Idle {
            let core_ref = core.borrow();
            let effects = core_ref.process_event(Event::Session(SessionEvent::StartBuilding));
            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
            started_building.set(true);
        }
    }

    // Navigate on state transitions
    Effect::new(move |_| {
        let vm = view_model.get();
        match vm.session_status {
            SessionStatusView::Active if started_building.get_untracked() => {
                navigate(
                    "/sessions/active",
                    NavigateOptions {
                        replace: true,
                        ..Default::default()
                    },
                );
            }
            SessionStatusView::Idle if started_building.get_untracked() => {
                // CancelBuilding or AbandonSession completed — go back
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

    let core_abandon = core.clone();
    let navigate_resume = use_navigate();

    view! {
        <div>
            <BackLink label="Back to Practice" href="/sessions".to_string() />
            <PageHeading text="New Practice" />

            {move || {
                let vm = view_model.get();
                if vm.session_status == SessionStatusView::Active {
                    // Active session exists — show recovery banner
                    let core_a = core_abandon.clone();
                    let nav = navigate_resume.clone();
                    Some(view! {
                        <Card>
                            <div class="space-y-3">
                                <p class="text-sm text-secondary">
                                    "You have a practice in progress."
                                </p>
                                <div class="flex gap-3">
                                    <Button
                                        variant=ButtonVariant::Primary
                                        on_click=Callback::new(move |_| {
                                            nav(
                                                "/sessions/active",
                                                NavigateOptions {
                                                    replace: true,
                                                    ..Default::default()
                                                },
                                            );
                                        })
                                    >
                                        "Resume Practice"
                                    </Button>
                                    <Button
                                        variant=ButtonVariant::Danger
                                        on_click=Callback::new(move |_| {
                                            let event = Event::Session(SessionEvent::AbandonSession);
                                            let core_ref = core_a.borrow();
                                            let effects = core_ref.process_event(event);
                                            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                        })
                                    >
                                        "Discard Practice"
                                    </Button>
                                </div>
                            </div>
                        </Card>
                    })
                } else {
                    None
                }
            }}

            // Only show the setlist builder when in building state
            <Show when=move || view_model.get().session_status == SessionStatusView::Building>
                <SetlistBuilder />
            </Show>
        </div>
    }
}
