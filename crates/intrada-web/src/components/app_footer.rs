use leptos::prelude::*;

/// Application footer with informational text.
#[component]
pub fn AppFooter() -> impl IntoView {
    view! {
        <footer class="max-w-4xl mx-auto px-4 sm:px-6 py-6 border-t border-border-default" role="contentinfo">
            <p class="text-xs text-faint text-center">
                "Built with Rust, Leptos & Crux by Jon Yardley - with help from Claude"
            </p>
        </footer>
    }
}
