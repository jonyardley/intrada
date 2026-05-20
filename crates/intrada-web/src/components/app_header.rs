use intrada_core::ViewModel;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

use crate::components::{BrandMark, ProfileButton};

/// Application header with name, tagline, and navigation.
///
/// Nav links highlight the active section using the same colour
/// (`text-accent-text`) as the mobile bottom tab bar.
#[component]
pub fn AppHeader() -> impl IntoView {
    let location = use_location();
    let view_model = expect_context::<RwSignal<ViewModel>>();

    let is_goals_active = move || {
        let path = location.pathname.get();
        path.starts_with("/goals")
    };

    let is_library_active = move || {
        let path = location.pathname.get();
        path.starts_with("/library")
    };

    let is_sessions_active = move || {
        let path = location.pathname.get();
        path.starts_with("/sessions")
    };

    let is_analytics_active = move || {
        let path = location.pathname.get();
        path.starts_with("/analytics")
    };

    // Tapping Practice jumps straight to whatever's already in flight so
    // the user can't accidentally start a second session on top of an
    // active one. Live > building > idle.
    let practice_href = move || {
        view_model.with(|vm| {
            if vm.active_session.is_some() {
                "/sessions/active".to_string()
            } else if vm.building_setlist.is_some() {
                "/sessions/new".to_string()
            } else {
                "/sessions".to_string()
            }
        })
    };

    let goals_enabled =
        Signal::derive(move || view_model.with(|vm| vm.features.as_ref().is_some_and(|f| f.goals)));

    view! {
        <header class="sm:glass-chrome sm:border-b sm:border-border-default" role="banner">
            <div class="max-w-4xl mx-auto px-card sm:px-card-comfortable py-4 flex items-center justify-between">
                <div>
                    <A href="/library" attr:class="no-underline">
                        <BrandMark />
                    </A>
                </div>
                <div class="hidden sm:flex items-center gap-4">
                    <nav class="flex items-center gap-4">
                        <Show when=move || goals_enabled.get()>
                            <A
                                href="/goals"
                                attr:class=move || {
                                    if is_goals_active() {
                                        "text-sm font-medium text-accent-text motion-safe:transition-colors"
                                    } else {
                                        "text-sm font-medium text-secondary hover:text-primary motion-safe:transition-colors"
                                    }
                                }
                                attr:aria-current=move || if is_goals_active() { Some("page") } else { None }
                            >
                                "Goals"
                            </A>
                        </Show>
                        <A
                            href="/library"
                            attr:class=move || {
                                if is_library_active() {
                                    "text-sm font-medium text-accent-text motion-safe:transition-colors"
                                } else {
                                    "text-sm font-medium text-secondary hover:text-primary motion-safe:transition-colors"
                                }
                            }
                            attr:aria-current=move || if is_library_active() { Some("page") } else { None }
                        >
                            "Library"
                        </A>
                        <A
                            href=practice_href
                            attr:class=move || {
                                if is_sessions_active() {
                                    "text-sm font-medium text-accent-text motion-safe:transition-colors"
                                } else {
                                    "text-sm font-medium text-secondary hover:text-primary motion-safe:transition-colors"
                                }
                            }
                            attr:aria-current=move || if is_sessions_active() { Some("page") } else { None }
                        >
                            "Practice"
                        </A>
                        <A
                            href="/analytics"
                            attr:class=move || {
                                if is_analytics_active() {
                                    "text-sm font-medium text-accent-text motion-safe:transition-colors"
                                } else {
                                    "text-sm font-medium text-secondary hover:text-primary motion-safe:transition-colors"
                                }
                            }
                            attr:aria-current=move || if is_analytics_active() { Some("page") } else { None }
                        >
                            "Analytics"
                        </A>
                    </nav>
                    <ProfileButton />
                </div>
            </div>
        </header>
    }
}
