use leptos::prelude::*;

/// Shared coloured badge for item types (piece, exercise, or unknown).
#[component]
pub fn TypeBadge(item_type: String) -> impl IntoView {
    let classes = match item_type.as_str() {
        "piece" => "inline-flex items-center rounded-full bg-violet-100 px-3 py-1 text-sm font-medium text-violet-800",
        "exercise" => "inline-flex items-center rounded-full bg-emerald-100 px-3 py-1 text-sm font-medium text-emerald-800",
        _ => "inline-flex items-center rounded-full bg-slate-100 px-3 py-1 text-sm font-medium text-slate-800",
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
