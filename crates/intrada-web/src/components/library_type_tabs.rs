use leptos::prelude::*;

use intrada_core::ItemKind;
use intrada_web::haptics::haptic_selection;

/// Underline-style 2-tab toggle for filtering the Library by item kind.
///
/// Visually distinct from `TypeTabs` (which is a pill / segmented-control
/// for the Add/Edit forms): this one is a plain text row with an accent
/// underline on the active tab. The Pencil refresh frames use this pattern
/// for the Library — see `NEW DESIGN - Library` (k9mpoW).
#[component]
pub fn LibraryTypeTabs(active: Signal<ItemKind>, on_change: Callback<ItemKind>) -> impl IntoView {
    let tab_class = move |kind: ItemKind| {
        let base = "tabs-underline-btn";
        if active.get() == kind {
            format!("{base} tabs-underline-btn--active")
        } else {
            base.to_string()
        }
    };

    let handle_click = move |kind: ItemKind| {
        if active.get_untracked() == kind {
            return;
        }
        haptic_selection();
        on_change.run(kind);
    };

    let piece_selected = move || {
        if active.get() == ItemKind::Piece {
            "true"
        } else {
            "false"
        }
    };
    let exercise_selected = move || {
        if active.get() == ItemKind::Exercise {
            "true"
        } else {
            "false"
        }
    };

    view! {
        <div class="tabs-underline" role="tablist" aria-label="Filter by type">
            <button
                type="button"
                role="tab"
                aria-selected=piece_selected
                aria-controls="library-list"
                class=move || tab_class(ItemKind::Piece)
                on:click=move |_| handle_click(ItemKind::Piece)
            >
                "Pieces"
            </button>
            <button
                type="button"
                role="tab"
                aria-selected=exercise_selected
                aria-controls="library-list"
                class=move || tab_class(ItemKind::Exercise)
                on:click=move |_| handle_click(ItemKind::Exercise)
            >
                "Exercises"
            </button>
        </div>
    }
}
