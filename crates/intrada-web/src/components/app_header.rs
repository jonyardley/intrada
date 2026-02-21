use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

use intrada_web::clerk_bindings;

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

    let is_routines_active = move || {
        let path = location.pathname.get();
        path.starts_with("/routines")
    };

    let is_analytics_active = move || {
        let path = location.pathname.get();
        path.starts_with("/analytics")
    };

    view! {
        <header class="glass-chrome border-b border-border-default" role="banner">
            <div class="max-w-4xl mx-auto px-4 sm:px-6 py-4 sm:py-5 flex items-center justify-between">
                <div>
                    <A href="/" attr:class="no-underline">
                        <h1 class="text-2xl sm:text-3xl font-bold tracking-tight text-primary">"Intrada"</h1>
                    </A>
                    <p class="text-sm text-muted mt-0.5">"Your music practice companion"</p>
                </div>
                <nav class="hidden sm:flex items-center gap-4">
                    <A
                        href="/"
                        attr:class=move || {
                            if is_library_active() {
                                "text-sm font-medium text-accent-text motion-safe:transition-colors"
                            } else {
                                "text-sm font-medium text-secondary hover:text-white motion-safe:transition-colors"
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
                                "text-sm font-medium text-secondary hover:text-white motion-safe:transition-colors"
                            }
                        }
                        attr:aria-current=move || if is_sessions_active() { Some("page") } else { None }
                    >
                        "Sessions"
                    </A>
                    <A
                        href="/routines"
                        attr:class=move || {
                            if is_routines_active() {
                                "text-sm font-medium text-accent-text motion-safe:transition-colors"
                            } else {
                                "text-sm font-medium text-secondary hover:text-white motion-safe:transition-colors"
                            }
                        }
                        attr:aria-current=move || if is_routines_active() { Some("page") } else { None }
                    >
                        "Routines"
                    </A>
                    <A
                        href="/analytics"
                        attr:class=move || {
                            if is_analytics_active() {
                                "text-sm font-medium text-accent-text motion-safe:transition-colors"
                            } else {
                                "text-sm font-medium text-secondary hover:text-white motion-safe:transition-colors"
                            }
                        }
                        attr:aria-current=move || if is_analytics_active() { Some("page") } else { None }
                    >
                        "Analytics"
                    </A>
                    <Show when=move || clerk_bindings::is_signed_in()>
                        <button
                            on:click=move |_| {
                                leptos::task::spawn_local(async move {
                                    clerk_bindings::sign_out().await;
                                });
                            }
                            class="text-sm font-medium text-muted hover:text-white motion-safe:transition-colors ml-2"
                            aria-label="Sign out"
                        >
                            "Sign out"
                        </button>
                    </Show>
                </nav>
            </div>
        </header>
    }
}
