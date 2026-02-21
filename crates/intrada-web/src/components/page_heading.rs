use leptos::prelude::*;

/// Shared page-level heading with consistent styling.
///
/// Uses the serif heading font (Source Serif 4) to signal
/// "music space" (audit #9).
#[component]
pub fn PageHeading(text: &'static str) -> impl IntoView {
    view! {
        <h2 class="text-2xl font-bold text-primary mb-6 font-heading">{text}</h2>
    }
}
