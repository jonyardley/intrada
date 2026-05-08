use leptos::prelude::*;

/// A single card in the welcome carousel — layered typography with
/// an optional pillar label, anchor phrase, and continuation. The
/// animated mark and CTA button are passed in by the carousel as
/// children rendered above and below the text block respectively.
///
/// The opener (card 1) skips the label by passing `label = None`;
/// the final card uses `children` for the CTA.
#[component]
pub fn WelcomeCard(
    /// Optional pillar label rendered above the anchor (e.g. "CAPTURE").
    /// Pass `None` for cards that should skip the label.
    label: Option<String>,
    /// Main anchor phrase rendered as a large serif heading.
    #[prop(into)]
    anchor: String,
    /// Optional softer continuation rendered below the anchor.
    continuation: Option<String>,
    /// Optional CTA slot rendered below the continuation. Used on the
    /// final card for the "Get started →" button. The animated mark
    /// is rendered separately by the carousel above this card, not
    /// inside it.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center text-center px-card-comfortable max-w-md mx-auto">
            // Pillar label — small uppercase muted, skipped on opener.
            {label.map(|l| view! {
                <p class="field-label mb-3">{l}</p>
            })}

            // Anchor — large serif heading.
            <p class="page-title">{anchor}</p>

            // Continuation — softer body line, smaller and muted.
            {continuation.map(|c| view! {
                <p class="text-base text-muted mt-3 max-w-sm">{c}</p>
            })}

            // CTA slot — used on the final card.
            // mt-10 is intentional breathing room on the full-screen canvas
            // between continuation and CTA, no matching token in the spacing scale.
            {children.map(|c| view! {
                <div class="mt-10 w-full">{c()}</div>
            })}
        </div>
    }
}
