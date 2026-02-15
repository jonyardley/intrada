use leptos::prelude::*;

/// Shared page-level heading with consistent styling.
#[component]
pub fn PageHeading(text: &'static str) -> impl IntoView {
    view! {
        <h2 class="text-2xl font-bold text-white mb-6">{text}</h2>
    }
}
