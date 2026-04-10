use leptos::ev;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::{Event, SessionEvent, SessionStatusView, ViewModel};

use crate::components::{BackLink, Button, ButtonVariant, Card, PageHeading, SetlistBuilder};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Session creation view: start building a setlist and launch practice.
///
/// Shows preset duration buttons (10/15/20/30 min) and a custom session option.
/// If an active session already exists (e.g. from crash recovery), shows
/// Resume / Discard options instead.
#[component]
pub fn SessionNewView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let navigate = use_navigate();

    // Track whether we entered building state on this mount.
    let started_building = RwSignal::new(false);

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
    let core_presets = core.clone();
    let core_custom = core.clone();
    let navigate_resume = use_navigate();

    let make_preset_cb = |mins: u32, core: SharedCore| {
        Callback::new(move |_: ev::MouseEvent| {
            let core_ref = core.borrow();
            let effects =
                core_ref.process_event(Event::Session(SessionEvent::StartBuildingWithTarget {
                    target_duration_mins: mins,
                }));
            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
            started_building.set(true);
        })
    };
    let preset_10 = make_preset_cb(10, core_presets.clone());
    let preset_15 = make_preset_cb(15, core_presets.clone());
    let preset_20 = make_preset_cb(20, core_presets.clone());
    let preset_30 = make_preset_cb(30, core_presets);

    let start_custom = Callback::new(move |_: ev::MouseEvent| {
        let core_ref = core_custom.borrow();
        let effects = core_ref.process_event(Event::Session(SessionEvent::StartBuilding));
        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
        started_building.set(true);
    });

    view! {
        <div>
            <BackLink label="Back to Practice" href="/sessions".to_string() />
            <PageHeading text="New Session" />

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
                                    "You have a session in progress."
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
                                        "Resume Session"
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
                                        "Discard Session"
                                    </Button>
                                </div>
                            </div>
                        </Card>
                    })
                } else {
                    None
                }
            }}

            // Preset buttons — shown when idle (before building starts)
            <Show when=move || view_model.get().session_status == SessionStatusView::Idle>
                <Card>
                    <div class="space-y-4">
                        <p class="field-label">
                            "Quick Start"
                        </p>
                        <div class="flex gap-3">
                            <Button
                                variant=ButtonVariant::Secondary
                                on_click=Callback::new(move |e: ev::MouseEvent| preset_10.run(e))
                            >
                                "10 min"
                            </Button>
                            <Button
                                variant=ButtonVariant::Secondary
                                on_click=Callback::new(move |e: ev::MouseEvent| preset_15.run(e))
                            >
                                "15 min"
                            </Button>
                            <Button
                                variant=ButtonVariant::Secondary
                                on_click=Callback::new(move |e: ev::MouseEvent| preset_20.run(e))
                            >
                                "20 min"
                            </Button>
                            <Button
                                variant=ButtonVariant::Secondary
                                on_click=Callback::new(move |e: ev::MouseEvent| preset_30.run(e))
                            >
                                "30 min"
                            </Button>
                        </div>
                        <div class="flex justify-center">
                            <Button
                                variant=ButtonVariant::Secondary
                                on_click=start_custom
                            >
                                "Custom Session"
                            </Button>
                        </div>
                    </div>
                </Card>
            </Show>

            // Only show the setlist builder when in building state
            <Show when=move || view_model.get().session_status == SessionStatusView::Building>
                <SetlistBuilder />
            </Show>
        </div>
    }
}
