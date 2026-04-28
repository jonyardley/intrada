use intrada_web::haptics;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

/// Mobile bottom tab bar for primary navigation.
///
/// Shows Library / Practice / Routines / Analytics tabs. Hidden on `sm:`
/// and wider where the header nav is visible instead.
///
/// Icons follow the iOS convention: outline (stroke) for the inactive
/// tabs, solid (fill) for the active one. Sizing matches the iOS tab
/// bar (~24px icon, 12px label).
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
                // Library tab — music note
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
                    {move || if is_library_active() {
                        view! { <MusicNoteIconSolid /> }.into_any()
                    } else {
                        view! { <MusicNoteIconOutline /> }.into_any()
                    }}
                    <span class="text-xs font-medium">"Library"</span>
                </A>

                // Practice tab — clock
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
                    {move || if is_sessions_active() {
                        view! { <ClockIconSolid /> }.into_any()
                    } else {
                        view! { <ClockIconOutline /> }.into_any()
                    }}
                    <span class="text-xs font-medium">"Practice"</span>
                </A>

                // Routines tab — list
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
                    {move || if is_routines_active() {
                        view! { <RoutinesIconSolid /> }.into_any()
                    } else {
                        view! { <RoutinesIconOutline /> }.into_any()
                    }}
                    <span class="text-xs font-medium">"Routines"</span>
                </A>

                // Analytics tab — bar chart
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
                    {move || if is_analytics_active() {
                        view! { <ChartIconSolid /> }.into_any()
                    } else {
                        view! { <ChartIconOutline /> }.into_any()
                    }}
                    <span class="text-xs font-medium">"Analytics"</span>
                </A>
            </div>
        </nav>
    }
}

// ════════════════════════════════════════════════════════════════════════
// Tab bar icons
//
// Heroicons v2 outline + solid pairs at 24×24, sized via h-6 w-6 to match
// the iOS tab bar's ~24px icon convention. Outline = stroke 1.5, no fill;
// solid = fill currentColor. Active vs inactive state swaps the variant.
// ════════════════════════════════════════════════════════════════════════

#[component]
fn MusicNoteIconOutline() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-6 w-6"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            aria-hidden="true"
        >
            <path stroke-linecap="round" stroke-linejoin="round" d="M9 9l10.5-3m0 6.553v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 11-.99-3.467l2.31-.66a2.25 2.25 0 001.632-2.163zm0 0V2.25L9 5.25v10.303m0 0v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 01-.99-3.467l2.31-.66A2.25 2.25 0 009 15.553z" />
        </svg>
    }
}

#[component]
fn MusicNoteIconSolid() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-6 w-6"
            viewBox="0 0 24 24"
            fill="currentColor"
            aria-hidden="true"
        >
            <path fill-rule="evenodd" d="M19.952 1.651a.75.75 0 01.298.599V16.303a3 3 0 01-2.176 2.884l-1.32.377a2.553 2.553 0 11-1.402-4.911l2.32-.662A1.5 1.5 0 0018.75 12.55v-2.66l-9 2.571v4.69a3 3 0 01-2.176 2.884l-1.32.377a2.553 2.553 0 11-1.402-4.911l2.32-.662A1.5 1.5 0 008.25 13.054V5.25a.75.75 0 01.544-.721l10.5-3a.75.75 0 01.658.122z" clip-rule="evenodd" />
        </svg>
    }
}

#[component]
fn ClockIconOutline() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-6 w-6"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            aria-hidden="true"
        >
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
    }
}

#[component]
fn ClockIconSolid() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-6 w-6"
            viewBox="0 0 24 24"
            fill="currentColor"
            aria-hidden="true"
        >
            <path fill-rule="evenodd" d="M12 2.25c-5.385 0-9.75 4.365-9.75 9.75s4.365 9.75 9.75 9.75 9.75-4.365 9.75-9.75S17.385 2.25 12 2.25zM12.75 6a.75.75 0 00-1.5 0v6c0 .414.336.75.75.75h4.5a.75.75 0 000-1.5h-3.75V6z" clip-rule="evenodd" />
        </svg>
    }
}

#[component]
fn RoutinesIconOutline() -> impl IntoView {
    // Queue-list shape: a top "header" rule with three rows below — reads
    // unambiguously as "saved routine / template" rather than a generic
    // hamburger menu.
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-6 w-6"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            aria-hidden="true"
        >
            <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 12h16.5m-16.5 3.75h16.5M3.75 19.5h16.5M5.625 4.5h12.75a1.875 1.875 0 010 3.75H5.625a1.875 1.875 0 010-3.75z" />
        </svg>
    }
}

#[component]
fn RoutinesIconSolid() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-6 w-6"
            viewBox="0 0 24 24"
            fill="currentColor"
            aria-hidden="true"
        >
            <path d="M5.625 3.75a2.625 2.625 0 100 5.25h12.75a2.625 2.625 0 000-5.25H5.625zM3.75 11.25a.75.75 0 000 1.5h16.5a.75.75 0 000-1.5H3.75zM3 15.75a.75.75 0 01.75-.75h16.5a.75.75 0 010 1.5H3.75a.75.75 0 01-.75-.75zM3.75 18.75a.75.75 0 000 1.5h16.5a.75.75 0 000-1.5H3.75z" />
        </svg>
    }
}

#[component]
fn ChartIconOutline() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-6 w-6"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="1.5"
            aria-hidden="true"
        >
            <path stroke-linecap="round" stroke-linejoin="round" d="M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 013 19.875v-6.75zM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V8.625zM16.5 4.125c0-.621.504-1.125 1.125-1.125h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V4.125z" />
        </svg>
    }
}

#[component]
fn ChartIconSolid() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-6 w-6"
            viewBox="0 0 24 24"
            fill="currentColor"
            aria-hidden="true"
        >
            <path d="M18.375 2.25c-1.035 0-1.875.84-1.875 1.875v15.75c0 1.035.84 1.875 1.875 1.875h.75c1.035 0 1.875-.84 1.875-1.875V4.125c0-1.036-.84-1.875-1.875-1.875h-.75zM9.75 8.625c0-1.036.84-1.875 1.875-1.875h.75c1.036 0 1.875.84 1.875 1.875v11.25c0 1.035-.84 1.875-1.875 1.875h-.75a1.875 1.875 0 01-1.875-1.875V8.625zM3 13.125c0-1.036.84-1.875 1.875-1.875h.75c1.036 0 1.875.84 1.875 1.875v6.75c0 1.035-.84 1.875-1.875 1.875h-.75A1.875 1.875 0 013 19.875v-6.75z" />
        </svg>
    }
}
