use leptos::prelude::*;

/// Shared section heading — used for top-level sections within a card
/// (e.g., "Add from Library", "Entries").
///
/// Uses the `section-title` design token utility (see `input.css`).
#[component]
pub fn SectionTitle(text: &'static str) -> impl IntoView {
    view! {
        <h3 class="section-title">{text}</h3>
    }
}
