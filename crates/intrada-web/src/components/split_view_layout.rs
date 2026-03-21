use leptos::prelude::*;

/// Responsive split-view layout container.
///
/// On desktop (≥768px / `md:` breakpoint): renders sidebar and detail side-by-side.
/// On mobile (<768px): renders only the active slot (controlled by CSS visibility).
///
/// The sidebar has a fixed width (~320px) and the detail pane fills remaining space.
/// Both slots are always in the DOM; CSS controls which is visible on mobile.
#[component]
pub fn SplitViewLayout(
    /// Content for the sidebar (left column on desktop).
    sidebar: Children,
    /// Content for the detail pane (right column on desktop).
    detail: Children,
    /// Whether to show the detail pane on mobile (hides sidebar).
    /// When false on mobile, sidebar is shown and detail is hidden.
    #[prop(into)]
    show_detail_mobile: Signal<bool>,
) -> impl IntoView {
    view! {
        <div class="flex h-full">
            // Sidebar — always visible on desktop, conditionally on mobile
            <div class={move || {
                if show_detail_mobile.get() {
                    // Mobile: detail is active, hide sidebar
                    "hidden md:flex md:flex-col md:w-80 md:shrink-0 md:border-r md:border-border-default md:overflow-y-auto"
                } else {
                    // Mobile: sidebar is active, show it full-width
                    "flex flex-col w-full md:w-80 md:shrink-0 md:border-r md:border-border-default md:overflow-y-auto"
                }
            }}>
                {sidebar()}
            </div>

            // Detail pane — always visible on desktop, conditionally on mobile
            <div class={move || {
                if show_detail_mobile.get() {
                    // Mobile: detail is active, show it full-width
                    "flex flex-col flex-1 overflow-y-auto w-full"
                } else {
                    // Mobile: sidebar is active, hide detail
                    "hidden md:flex md:flex-col md:flex-1 md:overflow-y-auto"
                }
            }}>
                {detail()}
            </div>
        </div>
    }
}
