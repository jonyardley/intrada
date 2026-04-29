use leptos::prelude::*;

use intrada_web::types::ItemType;

/// Inline type indicator — 6px coloured dot + label. Replaces the boxed
/// `TypeBadge` for in-row use in the 2026 refresh, where the boxed
/// badge would visually compete with the row's accent bar.
///
/// The boxed `TypeBadge` is still the right choice for surfaces where
/// the type is the primary content (e.g. the form-mode toggle in
/// `Add Piece`).
#[component]
pub fn InlineTypeIndicator(item_type: ItemType) -> impl IntoView {
    let (modifier, label) = match item_type {
        ItemType::Piece => ("inline-type-indicator--piece", "Piece"),
        ItemType::Exercise => ("inline-type-indicator--exercise", "Exercise"),
    };
    let class = format!("inline-type-indicator {modifier}");
    view! {
        <span class=class>
            <span class="inline-type-indicator-dot" aria-hidden="true"></span>
            {label}
        </span>
    }
}
