use leptos::prelude::*;

/// A small stat display card for analytics metrics (e.g., streak, weekly total).
/// Renders a glassmorphism card matching the existing `Card` style.
#[component]
pub fn StatCard(
    title: &'static str,
    #[prop(into)] value: String,
    #[prop(optional)] subtitle: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div class="bg-indigo-950/80 supports-backdrop:bg-white/10 supports-backdrop:backdrop-blur-md border border-white/15 rounded-xl shadow-lg p-4 text-center">
            <p class="text-xs font-medium text-gray-400 uppercase tracking-wider">{title}</p>
            <p class="text-2xl font-bold text-white mt-1">{value}</p>
            {subtitle.map(|s| view! {
                <p class="text-xs text-gray-400 mt-0.5">{s}</p>
            })}
        </div>
    }
}
