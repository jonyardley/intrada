use leptos::prelude::*;

use intrada_core::ItemKind;

/// Shared coloured badge for item types (piece, exercise).
///
/// Uses the badge design tokens from `input.css`. Both type badges have
/// equal visual weight but different hues (audit #16):
/// - Piece: accent-derived (indigo)
/// - Exercise: warm-accent-derived (gold)
#[component]
pub fn TypeBadge(item_type: ItemKind) -> impl IntoView {
    let (classes, display) = match item_type {
        ItemKind::Piece => ("badge badge--lg badge--piece", "Piece"),
        ItemKind::Exercise => ("badge badge--lg badge--exercise", "Exercise"),
    };

    view! {
        <span class=classes>{display}</span>
    }
}
