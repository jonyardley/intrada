use leptos::prelude::*;
use leptos_icons::Icon as LucideIcon;

/// Thin wrapper around `leptos_icons::Icon` that adds Tailwind class support.
/// Pass an `icondata::Lu*` value for `icon`; size and colour come from class.
#[component]
pub fn Icon(icon: icondata::Icon, #[prop(optional, into)] class: String) -> impl IntoView {
    view! { <LucideIcon icon=icon attr:class=class width="1em" height="1em" /> }
}
