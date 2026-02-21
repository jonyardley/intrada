use leptos::prelude::*;

/// Shared definition-term label for data display contexts.
///
/// Uses the `field-label` design token utility (see `input.css`).
#[component]
pub fn FieldLabel(text: &'static str) -> impl IntoView {
    view! {
        <dt class="field-label">{text}</dt>
    }
}
