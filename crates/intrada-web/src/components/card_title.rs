use leptos::prelude::*;

/// Shared card/subsection heading — used for titled sections inside cards
/// (e.g., "Practice History (28 days)", "Most Practised", "Score Trends").
///
/// Uses the `card-title` design token utility (see `input.css`).
#[component]
pub fn CardTitle(#[prop(into)] text: String) -> impl IntoView {
    view! {
        <h3 class="card-title">{text}</h3>
    }
}
