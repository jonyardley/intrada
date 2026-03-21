use leptos::prelude::*;

use crate::components::{Button, ButtonVariant};

/// Sticky bottom bar for the mobile session builder.
///
/// Shows item count, total duration, and a "Start Session" button.
/// The summary area is tappable (opens the setlist sheet).
/// Visible only on mobile (hidden on `md:` breakpoint).
#[component]
pub fn StickyBottomBar(
    /// Number of items in the setlist.
    #[prop(into)]
    item_count: Signal<usize>,
    /// Total planned duration in minutes.
    #[prop(into)]
    total_minutes: Signal<u32>,
    /// Whether the Start Session button is disabled.
    #[prop(into)]
    disabled: Signal<bool>,
    /// Called when the summary area is tapped (to open setlist sheet).
    on_summary_click: Callback<()>,
    /// Called when "Start Session" is clicked.
    on_start: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="fixed bottom-0 left-0 right-0 md:hidden z-40 bg-surface-chrome border-t border-border-default backdrop-blur-lg">
            <div class="flex items-center justify-between px-4 py-3 safe-area-bottom">
                // Summary area (tappable to open sheet)
                <button
                    class="flex-1 text-left"
                    on:click=move |_| on_summary_click.run(())
                >
                    <span class="text-sm font-medium text-primary">
                        {move || {
                            let count = item_count.get();
                            if count == 0 {
                                "No items selected".to_string()
                            } else {
                                let mins = total_minutes.get();
                                if mins > 0 {
                                    format!("{count} item{} · {mins} min", if count == 1 { "" } else { "s" })
                                } else {
                                    format!("{count} item{}", if count == 1 { "" } else { "s" })
                                }
                            }
                        }}
                    </span>
                </button>

                // Start Session button
                <div class="shrink-0 ml-3">
                    <Button
                        variant=ButtonVariant::Primary
                        disabled=Signal::derive(move || disabled.get())
                        on_click=Callback::new(move |_| on_start.run(()))
                    >
                        "Start Session"
                    </Button>
                </div>
            </div>
        </div>
    }
}
