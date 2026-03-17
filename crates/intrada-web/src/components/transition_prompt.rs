use leptos::prelude::*;

/// Non-blocking visual prompt shown when an item's planned duration has elapsed.
///
/// Displays either "Up next: [Item Name]" when there's a next item, or
/// "Practice complete — ready to finish?" for the last item. The prompt is
/// purely informational — it doesn't block controls or interrupt practice.
#[component]
pub fn TransitionPrompt(
    /// The title of the next item, or `None` if this is the last item.
    next_item_title: Option<String>,
) -> impl IntoView {
    view! {
        <div
            class="rounded-xl border border-border-default bg-surface-secondary px-4 py-3 text-center motion-safe:animate-in motion-safe:fade-in"
            role="status"
            aria-live="polite"
        >
            {match next_item_title {
                Some(title) => view! {
                    <p class="text-sm text-secondary">
                        <span class="text-muted">"Up next: "</span>
                        <span class="font-semibold text-primary">{title}</span>
                    </p>
                }.into_any(),
                None => view! {
                    <p class="text-sm text-secondary">
                        "Session complete \u{2014} ready to finish?"
                    </p>
                }.into_any(),
            }}
        </div>
    }
}
