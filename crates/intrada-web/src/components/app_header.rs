use leptos::prelude::*;
use leptos_router::components::A;

/// Application header with name, tagline, navigation, and version badge.
#[component]
pub fn AppHeader() -> impl IntoView {
    view! {
        <header class="bg-gray-900/60 supports-backdrop:backdrop-blur-md border-b border-white/10" role="banner">
            <div class="max-w-4xl mx-auto px-4 sm:px-6 py-4 sm:py-5 flex items-center justify-between">
                <div>
                    <A href="/" attr:class="no-underline">
                        <h1 class="text-2xl sm:text-3xl font-bold tracking-tight text-white">"Intrada"</h1>
                    </A>
                    <p class="text-sm text-gray-400 mt-0.5">"Your music practice companion"</p>
                </div>
                <nav class="hidden sm:flex items-center gap-4">
                    <A href="/" attr:class="text-sm font-medium text-gray-300 hover:text-white motion-safe:transition-colors">"Library"</A>
                    <A href="/sessions" attr:class="text-sm font-medium text-gray-300 hover:text-white motion-safe:transition-colors">"Sessions"</A>
                    <span
                        class="inline-flex items-center rounded-full bg-amber-900/40 px-3 py-1 text-xs font-medium text-amber-300"
                        aria-label="Application version"
                    >
                        "v0.1.0"
                    </span>
                </nav>
            </div>
        </header>
    }
}
