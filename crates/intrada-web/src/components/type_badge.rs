use leptos::prelude::*;

/// Shared coloured badge for item types (piece, exercise, or unknown).
///
/// Uses the badge design tokens from `input.css`. Both type badges have
/// equal visual weight but different hues (audit #16):
/// - Piece: accent-derived (indigo)
/// - Exercise: warm-accent-derived (gold)
#[component]
pub fn TypeBadge(item_type: String) -> impl IntoView {
    let classes = match item_type.as_str() {
        "piece" => "inline-flex items-center rounded-full bg-badge-piece-bg px-3 py-1 text-sm font-medium text-badge-piece-text",
        "exercise" => "inline-flex items-center rounded-full bg-badge-exercise-bg px-3 py-1 text-sm font-medium text-badge-exercise-text",
        _ => "inline-flex items-center rounded-full bg-surface-primary px-3 py-1 text-sm font-medium text-secondary",
    };

    let display = match item_type.as_str() {
        "piece" => "Piece".to_string(),
        "exercise" => "Exercise".to_string(),
        other => other.to_string(),
    };

    view! {
        <span class=classes>{display}</span>
    }
}
