use leptos::prelude::*;

/// A small stat display card for analytics metrics (e.g., streak, weekly total).
///
/// Uses compact padding (12px) so the stat value dominates rather than
/// swimming in whitespace (audit #11). Uses the `glass-card` design
/// token utility (see `input.css`).
#[component]
pub fn StatCard(
    title: &'static str,
    #[prop(into)] value: String,
    #[prop(optional)] subtitle: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div class="glass-card p-card-compact text-center">
            <p class="field-label">{title}</p>
            <p class="text-2xl font-bold text-primary mt-1">{value}</p>
            {subtitle.map(|s| view! {
                <p class="text-xs text-muted mt-0.5">{s}</p>
            })}
        </div>
    }
}
