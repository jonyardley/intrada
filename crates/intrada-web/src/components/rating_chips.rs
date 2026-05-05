use leptos::prelude::*;

/// 1–5 rating chips with toggle-to-clear behaviour. The signature
/// 36px-circle (`w-9 h-9`) row used wherever the user picks a self-rating
/// — post-session summary and the mid-session reflection sheet are the
/// current consumers.
///
/// Re-tapping the currently-selected value fires `on_change(None)`. The
/// component is purely presentational: it doesn't own state, doesn't talk
/// to the core, doesn't manage focus. Consumers wire `selected` from
/// wherever the source of truth lives (a local signal, a view-model
/// snapshot, etc.) and `on_change` does the persistence.
#[component]
pub fn RatingChips(
    /// Currently-selected rating, 1–5, or `None`.
    #[prop(into)]
    selected: Signal<Option<u8>>,
    /// Fired when the user picks a rating, or clears it by re-tapping
    /// the selected one. Receives the new value (`Some(n)` or `None`).
    on_change: Callback<Option<u8>>,
    /// Prefix for the `aria-label` of each chip — defaults to `"Rate"`,
    /// rendering e.g. `"Rate 4 out of 5"`. Pass `"Rate confidence"` on
    /// the summary screen so screen readers announce the chip's purpose.
    ///
    /// Constrained to `&'static str` while we only have static-string
    /// callers — switch to `Signal<String>` (or an `AttributeValue`) the
    /// moment a runtime label (e.g. i18n, item-name interpolation) is
    /// needed.
    #[prop(optional, default = "Rate")]
    aria_label_prefix: &'static str,
) -> impl IntoView {
    view! {
        <div class="flex items-center gap-2">
            {(1u8..=5).map(|n| {
                let is_selected = move || selected.get() == Some(n);
                let class_fn = move || {
                    if is_selected() {
                        "w-9 h-9 rounded-full text-sm font-semibold bg-accent text-primary shadow-md motion-safe:transition-all motion-safe:duration-150"
                    } else {
                        "w-9 h-9 rounded-full text-sm font-semibold bg-surface-primary text-primary/60 hover:bg-surface-hover hover:text-primary motion-safe:transition-all motion-safe:duration-150"
                    }
                };
                let aria_label = format!("{aria_label_prefix} {n} out of 5");
                let aria_pressed = move || if is_selected() { "true" } else { "false" };
                view! {
                    <button
                        type="button"
                        class=class_fn
                        aria-label=aria_label
                        aria-pressed=aria_pressed
                        on:click=move |_| {
                            // Toggle: re-tapping the selected value clears it.
                            let next = if selected.get_untracked() == Some(n) { None } else { Some(n) };
                            on_change.run(next);
                        }
                    >
                        {n.to_string()}
                    </button>
                }
            }).collect::<Vec<_>>()}
        </div>
    }
}
