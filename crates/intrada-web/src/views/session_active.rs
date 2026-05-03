use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::{SessionStatusView, ViewModel};

use crate::app::FocusMode;
use crate::components::{Icon, IconName, SessionTimer};

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

    let focus_signal = focus_mode.0;
    let title_class = move || {
        if focus_signal.get() {
            "focus-fade focus-fade--hidden"
        } else {
            "focus-fade"
        }
    };
    let toggle_aria = move || {
        if focus_signal.get() {
            "Exit focus mode"
        } else {
            "Enter focus mode"
        }
    };

    view! {
        <div>
            // Top row: page title (fades in focus mode) + persistent
            // focus-toggle icon button on the trailing edge. The row's
            // min-height keeps the button anchored even when the title
            // collapses, so the user always has a way back out of focus.
            <div class="flex items-center justify-between gap-3 mb-6 min-h-[44px]">
                <div class=title_class>
                    <h2 class="page-title">"Practice"</h2>
                </div>
                <button
                    type="button"
                    class="icon-nav-button"
                    aria-label=toggle_aria
                    on:click=move |_| focus_signal.set(!focus_signal.get_untracked())
                >
                    {move || if focus_signal.get() {
                        view! { <Icon name=IconName::ChevronDown class="w-5 h-5" /> }
                    } else {
                        view! { <Icon name=IconName::ChevronUp class="w-5 h-5" /> }
                    }}
                </button>
            </div>
            <SessionTimer />
        </div>
    }
}
