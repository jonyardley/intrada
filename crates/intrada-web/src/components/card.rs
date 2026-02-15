use leptos::prelude::*;

/// Shared card container with glassmorphism styling — semi-transparent with backdrop blur.
/// Falls back to solid semi-opaque background when backdrop-filter is not supported.
#[component]
pub fn Card(children: Children) -> impl IntoView {
    view! {
        <div class="bg-indigo-950/80 supports-backdrop:bg-white/10 supports-backdrop:backdrop-blur-md border border-white/15 rounded-xl shadow-lg p-4 sm:p-6">
            {children()}
        </div>
    }
}
