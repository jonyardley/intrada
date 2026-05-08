use leptos::prelude::*;
use leptos_router::components::A;

use intrada_web::clerk_bindings;

/// Avatar button that links to the Settings route.
///
/// Shows the first letter of the signed-in user's email (initial fallback
/// when Clerk doesn't expose a name). Hidden when not signed in — the
/// app-level auth gate prevents render in that case anyway, but the
/// `Show` keeps the chrome stable during early init.
#[component]
pub fn ProfileButton() -> impl IntoView {
    let initial = move || {
        clerk_bindings::email()
            .as_deref()
            .and_then(|s| s.chars().next())
            .map(|c| c.to_ascii_uppercase().to_string())
            .unwrap_or_else(|| "?".to_string())
    };

    view! {
        <Show when=move || clerk_bindings::is_signed_in()>
            <A
                href="/settings"
                attr:class="ml-2 flex items-center justify-center h-8 w-8 rounded-full bg-surface-primary border border-border-default text-sm font-medium text-primary hover:bg-surface-hover focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent-focus motion-safe:transition-colors no-underline"
                attr:aria-label="Account and settings"
            >
                {initial}
            </A>
        </Show>
    }
}
