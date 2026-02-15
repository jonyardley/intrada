use leptos::prelude::*;
use leptos_router::components::A;

/// Shared back-navigation link with left arrow icon.
#[component]
pub fn BackLink(label: &'static str, href: String) -> impl IntoView {
    view! {
        <A href=href attr:class="mb-6 inline-flex items-center gap-1 text-sm text-gray-400 hover:text-white motion-safe:transition-colors">
            "← "{label}
        </A>
    }
}
