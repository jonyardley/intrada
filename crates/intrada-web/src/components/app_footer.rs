use leptos::prelude::*;

/// Application footer with informational text.
#[component]
pub fn AppFooter() -> impl IntoView {
    view! {
        <footer class="max-w-4xl mx-auto px-4 sm:px-6 py-6 sm:border-t sm:border-border-default" role="contentinfo">
            <p class="text-xs text-faint text-center">
                "© 2026 Jon Yardley · Built for musicians who practice with intent."
            </p>
        </footer>
    }
}
