use leptos::prelude::*;

/// Shared coloured badge for item types (piece, exercise, or unknown).
#[component]
pub fn TypeBadge(item_type: String) -> impl IntoView {
    let classes = match item_type.as_str() {
        "piece" => "inline-flex items-center rounded-full bg-violet-500/20 px-3 py-1 text-sm font-medium text-violet-300",
        "exercise" => "inline-flex items-center rounded-full bg-emerald-500/20 px-3 py-1 text-sm font-medium text-emerald-300",
        _ => "inline-flex items-center rounded-full bg-white/10 px-3 py-1 text-sm font-medium text-gray-300",
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
