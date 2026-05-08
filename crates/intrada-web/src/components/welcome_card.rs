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
        <div class="flex flex-col items-center justify-center text-center px-6 max-w-md mx-auto">
            <p class="font-heading text-[1.75rem] leading-9 font-semibold text-primary tracking-tight">
                {copy}
            </p>
            {children.map(|c| view! {
                <div class="mt-10 w-full">
                    {c()}
                </div>
            })}
        </div>
    }
}
