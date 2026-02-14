use leptos::prelude::*;

/// Application header with name, tagline, and version badge.
#[component]
pub fn AppHeader() -> impl IntoView {
    view! {
        <header class="bg-white shadow-sm border-b border-slate-200" role="banner">
            <div class="max-w-4xl mx-auto px-6 py-5 flex items-center justify-between">
                <div>
                    <h1 class="text-3xl font-bold tracking-tight text-slate-900">"Intrada"</h1>
                    <p class="text-sm text-slate-500 mt-0.5">"Your music practice companion"</p>
                </div>
                <span
                    class="inline-flex items-center rounded-full bg-amber-100 px-3 py-1 text-xs font-medium text-amber-800"
                    aria-label="Application version"
                >
                    "v0.1.0"
                </span>
            </div>
        </header>
    }
}
