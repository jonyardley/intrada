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

    let is_library_active = move || {
        let path = location.pathname.get();
        path == "/" || path.starts_with("/library")
    };

    let is_sessions_active = move || {
        let path = location.pathname.get();
        path.starts_with("/sessions")
    };

    let is_analytics_active = move || {
        let path = location.pathname.get();
        path.starts_with("/analytics")
    };

    view! {
        <header class="glass-chrome border-b border-border-default" role="banner">
            <div class="max-w-4xl mx-auto px-card sm:px-card-comfortable py-card sm:py-card-comfortable flex items-center justify-between">
                <div>
                    <A href="/" attr:class="no-underline">
                        <h1 class="text-2xl sm:text-3xl font-bold tracking-tight text-primary">"Intrada"</h1>
                    </A>
                </div>
                <div class="flex items-center gap-4">
                    <nav class="hidden sm:flex items-center gap-4">
                        <A
                            href="/"
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
                            href="/sessions"
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
