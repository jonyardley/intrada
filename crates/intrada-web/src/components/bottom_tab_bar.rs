use intrada_web::haptics;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

/// Mobile bottom tab bar for primary navigation.
///
/// Shows Library, Sessions, and Analytics tabs. Hidden on `sm:` and wider
/// where the header nav is visible instead.
#[component]
pub fn BottomTabBar() -> impl IntoView {
    let location = use_location();
    // Prevents the spring animation firing on initial render when the
    // first tab is already active. Only animate after the user taps.
    let has_tapped = RwSignal::new(false);

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

    let spring = "flex flex-col items-center gap-0.5 text-accent-text tab-spring min-w-[64px] min-h-[44px] justify-center";
    let active = "flex flex-col items-center gap-0.5 text-accent-text min-w-[64px] min-h-[44px] justify-center";
    let inactive = "flex flex-col items-center gap-0.5 text-muted hover:text-secondary motion-safe:transition-colors min-w-[64px] min-h-[44px] justify-center";

    view! {
        <nav
            class="fixed inset-x-0 bottom-0 z-50 h-16 glass-chrome border-t border-border-default pb-safe sm:hidden"
            role="navigation"
            aria-label="Mobile navigation"
        >
            <div class="flex h-full items-center justify-around">
                // Library tab
                <A
                    href="/"
                    attr:class=move || {
                        if is_library_active() {
                            if has_tapped.get() { spring } else { active }
                        } else {
                            inactive
                        }
                    }
                    attr:aria-current=move || if is_library_active() { Some("page") } else { None }
                    on:click=move |_| { has_tapped.set(true); haptics::haptic_selection(); }
                >
                    // Music note icon (SVG)
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="h-5 w-5"
                        viewBox="0 0 20 20"
                        fill="currentColor"
                        aria-hidden="true"
                    >
                        <path d="M18 3a1 1 0 00-1.196-.98l-10 2A1 1 0 006 5v9.114A4.369 4.369 0 005 14c-1.657 0-3 .895-3 2s1.343 2 3 2 3-.895 3-2V7.82l8-1.6v5.894A4.37 4.37 0 0015 12c-1.657 0-3 .895-3 2s1.343 2 3 2 3-.895 3-2V3z" />
                    </svg>
                    <span class="text-xs font-medium">"Library"</span>
                </A>

                // Sessions tab
                <A
                    href="/sessions"
                    attr:class=move || {
                        if is_sessions_active() {
                            if has_tapped.get() { spring } else { active }
                        } else {
                            inactive
                        }
                    }
                    attr:aria-current=move || if is_sessions_active() { Some("page") } else { None }
                    on:click=move |_| { has_tapped.set(true); haptics::haptic_selection(); }
                >
                    // Clock/timer icon (SVG)
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="h-5 w-5"
                        viewBox="0 0 20 20"
                        fill="currentColor"
                        aria-hidden="true"
                    >
                        <path
                            fill-rule="evenodd"
                            d="M10 18a8 8 0 100-16 8 8 0 000 16zm1-12a1 1 0 10-2 0v4a1 1 0 00.293.707l2.828 2.829a1 1 0 101.415-1.415L11 9.586V6z"
                            clip-rule="evenodd"
                        />
                    </svg>
                    <span class="text-xs font-medium">"Practice"</span>
                </A>

                // Routines tab
                <A
                    href="/routines"
                    attr:class=move || {
                        if is_routines_active() {
                            if has_tapped.get() { spring } else { active }
                        } else {
                            inactive
                        }
                    }
                    attr:aria-current=move || if is_routines_active() { Some("page") } else { None }
                    on:click=move |_| { has_tapped.set(true); haptics::haptic_selection(); }
                >
                    // List/template icon (SVG)
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="h-5 w-5"
                        viewBox="0 0 20 20"
                        fill="currentColor"
                        aria-hidden="true"
                    >
                        <path
                            fill-rule="evenodd"
                            d="M3 4a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm0 4a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm0 4a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm0 4a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1z"
                            clip-rule="evenodd"
                        />
                    </svg>
                    <span class="text-xs font-medium">"Routines"</span>
                </A>

                // Analytics tab
                <A
                    href="/analytics"
                    attr:class=move || {
                        if is_analytics_active() {
                            if has_tapped.get() { spring } else { active }
                        } else {
                            inactive
                        }
                    }
                    attr:aria-current=move || if is_analytics_active() { Some("page") } else { None }
                    on:click=move |_| { has_tapped.set(true); haptics::haptic_selection(); }
                >
                    // Chart/bar-chart icon (SVG)
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="h-5 w-5"
                        viewBox="0 0 20 20"
                        fill="currentColor"
                        aria-hidden="true"
                    >
                        <path d="M2 11a1 1 0 011-1h2a1 1 0 011 1v5a1 1 0 01-1 1H3a1 1 0 01-1-1v-5zM8 7a1 1 0 011-1h2a1 1 0 011 1v9a1 1 0 01-1 1H9a1 1 0 01-1-1V7zM14 4a1 1 0 011-1h2a1 1 0 011 1v12a1 1 0 01-1 1h-2a1 1 0 01-1-1V4z" />
                    </svg>
                    <span class="text-xs font-medium">"Analytics"</span>
                </A>
            </div>
        </nav>
    }
}
