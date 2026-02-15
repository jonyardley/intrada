use leptos::prelude::*;

/// Application footer with informational text.
#[component]
pub fn AppFooter() -> impl IntoView {
    view! {
        <footer class="max-w-4xl mx-auto px-4 sm:px-6 py-6 border-t border-white/10" role="contentinfo">
            <p class="text-xs text-gray-500 text-center">
                "Built with Rust, Leptos & Crux — Page reload resets to stub data"
            </p>
        </footer>
    }
}
