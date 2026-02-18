use leptos::prelude::*;

/// A small stat display card for analytics metrics (e.g., streak, weekly total).
/// Uses the `glass-card` design token utility (see `input.css`).
#[component]
pub fn StatCard(
    title: &'static str,
    #[prop(into)] value: String,
    #[prop(optional)] subtitle: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div class="glass-card p-4 text-center">
            <p class="text-xs font-medium text-gray-400 uppercase tracking-wider">{title}</p>
            <p class="text-2xl font-bold text-white mt-1">{value}</p>
            {subtitle.map(|s| view! {
                <p class="text-xs text-gray-400 mt-0.5">{s}</p>
            })}
        </div>
    }
}
