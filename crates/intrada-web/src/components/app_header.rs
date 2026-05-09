use intrada_core::ViewModel;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

use crate::components::ProfileButton;

/// Application header with name, tagline, and navigation.
///
/// Nav links highlight the active section using the same colour
/// (`text-accent-text`) as the mobile bottom tab bar.
#[component]
pub fn AppHeader() -> impl IntoView {
    let location = use_location();
    let view_model = expect_context::<RwSignal<ViewModel>>();

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

    view! {
        <header class="sm:glass-chrome sm:border-b sm:border-border-default" role="banner">
            <div class="max-w-4xl mx-auto px-card sm:px-card-comfortable py-4 flex items-center justify-between">
                <div>
                    <A href="/library" attr:class="flex items-center gap-2.5 no-underline">
                        <svg class="w-5 h-5 text-accent" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M9 9l10.5-3m0 6.553v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 11-.99-3.467l2.31-.66a2.25 2.25 0 001.632-2.163zm0 0V2.25L9 5.25v10.303m0 0v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 01-.99-3.467l2.31-.66A2.25 2.25 0 009 15.553z" />
                        </svg>
                        <span class="text-lg font-bold text-primary font-heading">"Intrada"</span>
                    </A>
                </div>
                <div class="hidden sm:flex items-center gap-4">
                    <nav class="flex items-center gap-4">
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
