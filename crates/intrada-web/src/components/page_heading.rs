use leptos::prelude::*;

/// Shared page-level heading with consistent styling.
///
/// Uses the serif heading font (Source Serif 4) to signal
/// "music space" (audit #9). When a `subtitle` is provided,
/// a description paragraph is rendered beneath the heading.
#[component]
pub fn PageHeading(
    text: &'static str,
    #[prop(optional)] subtitle: Option<&'static str>,
) -> impl IntoView {
    let heading_mb = if subtitle.is_some() { "mb-3" } else { "mb-6" };

    view! {
        <div>
            <h2 class=format!("text-2xl font-bold text-primary font-heading {heading_mb}")>{text}</h2>
            {subtitle.map(|sub| view! {
                <p class="text-sm text-secondary leading-relaxed max-w-2xl mb-6">{sub}</p>
            })}
        </div>
    }
}
