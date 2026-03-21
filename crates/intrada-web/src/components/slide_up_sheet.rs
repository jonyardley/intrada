use leptos::prelude::*;

/// CSS-based slide-up sheet/panel for mobile.
///
/// Uses `transform: translateY()` with transition for smooth animation.
/// Includes a backdrop overlay that dismisses on tap.
#[component]
pub fn SlideUpSheet(
    /// Whether the sheet is open.
    #[prop(into)]
    is_open: Signal<bool>,
    /// Called when the backdrop is tapped (to dismiss).
    on_dismiss: Callback<()>,
    /// Sheet content.
    children: Children,
) -> impl IntoView {
    view! {
        // Backdrop overlay
        <div
            class={move || {
                if is_open.get() {
                    "fixed inset-0 z-50 bg-black/50 transition-opacity duration-300 opacity-100 md:hidden"
                } else {
                    "fixed inset-0 z-50 bg-black/50 transition-opacity duration-300 opacity-0 pointer-events-none md:hidden"
                }
            }}
            on:click=move |_| on_dismiss.run(())
        />

        // Sheet panel
        <div
            class={move || {
                if is_open.get() {
                    "fixed bottom-0 left-0 right-0 z-50 bg-surface-primary border-t border-border-default rounded-t-2xl max-h-[80vh] overflow-y-auto transition-transform duration-300 ease-out translate-y-0 md:hidden"
                } else {
                    "fixed bottom-0 left-0 right-0 z-50 bg-surface-primary border-t border-border-default rounded-t-2xl max-h-[80vh] overflow-y-auto transition-transform duration-300 ease-out translate-y-full md:hidden"
                }
            }}
            // Prevent click-through to backdrop
            on:click=move |e| e.stop_propagation()
        >
            // Drag handle indicator
            <div class="flex justify-center pt-3 pb-2">
                <div class="w-10 h-1 rounded-full bg-border-default" />
            </div>

            // Content
            <div class="px-4 pb-6 safe-area-bottom">
                {children()}
            </div>
        </div>
    }
}
