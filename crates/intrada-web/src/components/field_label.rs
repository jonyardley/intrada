use leptos::prelude::*;

/// Shared definition-term label for data display contexts.
#[component]
pub fn FieldLabel(text: &'static str) -> impl IntoView {
    view! {
        <dt class="text-xs font-medium text-gray-400 uppercase tracking-wider">{text}</dt>
    }
}
