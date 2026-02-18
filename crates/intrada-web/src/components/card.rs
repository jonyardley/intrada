use leptos::prelude::*;

/// Shared card container with glassmorphism styling — semi-transparent with backdrop blur.
/// Falls back to solid semi-opaque background when backdrop-filter is not supported.
///
/// Uses the `glass-card` design token utility (see `input.css`).
#[component]
pub fn Card(children: Children) -> impl IntoView {
    view! {
        <div class="glass-card p-4 sm:p-6">
            {children()}
        </div>
    }
}
