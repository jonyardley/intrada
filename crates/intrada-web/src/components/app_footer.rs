use leptos::prelude::*;

/// Application footer with informational text.
#[component]
pub fn AppFooter() -> impl IntoView {
    view! {
        <footer class="max-w-4xl mx-auto px-6 py-6 border-t border-slate-200" role="contentinfo">
            <p class="text-xs text-slate-400 text-center">
                "Built with Rust, Leptos & Crux \u{2014} Page reload resets to stub data"
            </p>
        </footer>
    }
}
