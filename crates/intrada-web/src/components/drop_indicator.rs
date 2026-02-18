use leptos::prelude::*;

/// A horizontal line that indicates where a dragged item will be dropped.
///
/// Renders a 2px `bg-indigo-400` horizontal line. Uses `motion-safe:` transitions
/// for `prefers-reduced-motion` compliance (FR-011). Hidden when `visible` is false.
#[component]
pub fn DropIndicator(
    /// Whether the drop indicator line should be visible.
    visible: Signal<bool>,
) -> impl IntoView {
    view! {
        <div
            class=move || {
                if visible.get() {
                    "drop-indicator opacity-100 h-0.5 my-0.5"
                } else {
                    "drop-indicator opacity-0 h-0 my-0"
                }
            }
            aria-hidden="true"
        />
    }
}
