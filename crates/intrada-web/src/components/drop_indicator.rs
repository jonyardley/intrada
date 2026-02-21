use leptos::prelude::*;

/// A horizontal line that indicates where a dragged item will be dropped.
///
/// Renders a 2px accent-coloured horizontal line. Uses `motion-safe:` transitions
/// for `prefers-reduced-motion` compliance (FR-011). Hidden when `visible` is false.
///
/// IMPORTANT: The indicator always occupies the same layout space (h-0.5 my-0.5)
/// regardless of visibility. Only opacity changes. This prevents the indicator from
/// shifting entry bounding rects when it appears, which would cause the drag hook's
/// hover-index computation to oscillate (feedback loop).
#[component]
pub fn DropIndicator(
    /// Whether the drop indicator line should be visible.
    visible: Signal<bool>,
) -> impl IntoView {
    view! {
        <div
            class=move || {
                if visible.get() {
                    "drop-indicator h-0.5 my-0.5 opacity-100"
                } else {
                    "drop-indicator h-0.5 my-0.5 opacity-0"
                }
            }
            aria-hidden="true"
        />
    }
}
