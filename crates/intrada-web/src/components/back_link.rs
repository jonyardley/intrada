use leptos::prelude::*;
use leptos_router::components::A;

use super::{Icon, IconName};

/// Shared back-navigation link with left arrow icon.
#[component]
pub fn BackLink(label: &'static str, href: String) -> impl IntoView {
    view! {
        <A href=href attr:class="mb-6 inline-flex items-center gap-1 text-sm text-muted hover:text-primary motion-safe:transition-colors">
            <Icon name=IconName::ArrowLeft class="w-4 h-4" />
            {label}
        </A>
    }
}
