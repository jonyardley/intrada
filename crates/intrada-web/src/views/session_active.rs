use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::{SessionStatusView, ViewModel};

use crate::app::FocusMode;
use crate::components::{PageHeading, SessionTimer};

/// Active session view: wraps the SessionTimer, redirects when session state changes.
#[component]
pub fn SessionActiveView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let focus_mode = expect_context::<FocusMode>();
    let navigate = use_navigate();

    // Enter focus mode on mount, exit on unmount
    focus_mode.set(true);
    on_cleanup(move || {
        focus_mode.set(false);
    });

    // Redirect if no active session
    {
        let vm = view_model.get_untracked();
        if vm.session_status != SessionStatusView::Active {
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
        match vm.session_status {
            SessionStatusView::Summary => {
                navigate(
                    "/sessions/summary",
                    NavigateOptions {
                        replace: true,
                        ..Default::default()
                    },
                );
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
    });

    view! {
        <div>
            <Show when=move || !focus_mode.get()>
                <PageHeading text="Practice" />
            </Show>
            <SessionTimer />
        </div>
    }
}
