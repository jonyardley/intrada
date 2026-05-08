use leptos::prelude::*;

/// A single card in the welcome carousel — typographic copy with optional
/// CTA slot for the final card.
#[component]
pub fn WelcomeCard(
    /// The card copy text.
    #[prop(into)]
    copy: String,
    /// Optional CTA slot (used on the final card for the "Add your first piece" button).
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    view! {
        // px-card-comfortable for horizontal padding token; mt-10 is intentional
        // breathing room on the full-screen canvas between copy and CTA, no
        // matching token in the spacing scale.
        <div class="flex flex-col items-center justify-center text-center px-card-comfortable max-w-md mx-auto">
            <p class="page-title">{copy}</p>
            {children.map(|c| view! {
                <div class="mt-10 w-full">
                    {c()}
                </div>
            })}
        </div>
    }
}
