use leptos::prelude::*;

/// Shared page-level heading with consistent styling.
///
/// Uses the serif heading font (Source Serif 4) to signal "music space"
/// (audit #9). When a `subtitle` is provided, a description paragraph is
/// rendered beneath the heading.
///
/// An optional `trailing` slot sits on the title's row at the trailing
/// edge — used for nav-bar-style page actions (e.g. an "Add" button).
/// The trailing slot vertically centres against the title only, not the
/// title + subtitle group, which avoids the "button looks like it's
/// floating below the heading" problem of putting the action in a
/// sibling flex container outside `<PageHeading>`.
#[component]
pub fn PageHeading(
    text: &'static str,
    #[prop(optional)] subtitle: Option<&'static str>,
    #[prop(optional)] trailing: Option<Children>,
) -> impl IntoView {
    let row_mb = if subtitle.is_some() { "mb-3" } else { "mb-6" };

    view! {
        <div>
            <div class=format!("flex items-center justify-between gap-3 {row_mb}")>
                <h2 class="text-2xl font-bold text-primary font-heading">{text}</h2>
                {trailing.map(|t| t())}
            </div>
            {subtitle.map(|sub| view! {
                <p class="text-sm text-secondary leading-relaxed max-w-2xl mb-6">{sub}</p>
            })}
        </div>
    }
}
