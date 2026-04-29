use leptos::prelude::*;

/// Uppercase letter-spaced label sitting above grouped content
/// (DETAILS / RECENT ACTIVITY / NOTES / THIS WEEK). Introduced in the
/// 2026 refresh as a visible IA cue without visual weight.
///
/// Pairs with `<DetailGroup>` but also works standalone above any list.
#[component]
pub fn SectionLabel(#[prop(into)] text: String) -> impl IntoView {
    view! {
        <p class="section-label">{text}</p>
    }
}
