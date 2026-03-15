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
        ItemKind::Piece => (
            "inline-flex items-center rounded-full bg-badge-piece-bg px-3 py-1 text-sm font-medium text-badge-piece-text",
            "Piece",
        ),
        ItemKind::Exercise => (
            "inline-flex items-center rounded-full bg-badge-exercise-bg px-3 py-1 text-sm font-medium text-badge-exercise-text",
            "Exercise",
        ),
    };

    view! {
        <span class=classes>{display}</span>
    }
}
