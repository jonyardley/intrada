use leptos::ev;
use leptos::prelude::*;
use leptos::web_sys;
use wasm_bindgen::JsCast;

use intrada_core::ItemKind;
use intrada_web::haptics::haptic_selection;

/// Underline-style tabs with a sliding accent indicator. Three states:
/// All (no filter) / Pieces / Exercises.
///
/// Visually distinct from `TypeTabs` (which is a pill / segmented-control
/// for the Add/Edit forms): a plain text row with a 3px accent underline
/// that slides between tabs on selection. The Pencil refresh frames use
/// this pattern for the Library — see `NEW DESIGN - Library` (k9mpoW).
///
/// Tabs are equal-width (`flex: 1`) so the slide is a pure CSS transform
/// (no DOM measurement) — `--thumb-x` drives `translateX(0% | 100% | 200%)`
/// on the absolute-positioned indicator.
///
/// Keyboard navigation: ArrowLeft / ArrowRight / Home / End move focus
/// between tabs and emit `on_change`, matching the WAI-ARIA tabs pattern
/// and the existing `TypeTabs` component for parity. No wrapping at edges.
#[component]
pub fn LibraryTypeTabs(
    /// Active filter. `None` = All (no kind filter); `Some(kind)` = filter to that kind.
    active: Signal<Option<ItemKind>>,
    on_change: Callback<Option<ItemKind>>,
) -> impl IntoView {
    // Tab order — used by both keyboard navigation and the indicator slide.
    // Indices: 0 = All, 1 = Pieces, 2 = Exercises.
    fn kind_at(index: usize) -> Option<ItemKind> {
        match index {
            0 => None,
            1 => Some(ItemKind::Piece),
            _ => Some(ItemKind::Exercise),
        }
    }
    fn index_of(kind: &Option<ItemKind>) -> usize {
        match kind {
            None => 0,
            Some(ItemKind::Piece) => 1,
            Some(ItemKind::Exercise) => 2,
        }
    }
    const N_TABS: usize = 3;

    let active_index = move || index_of(&active.get());

    let indicator_style = move || {
        let pct = active_index() * 100;
        format!("--thumb-x: {pct}%")
    };

    let tab_class = move |kind: Option<ItemKind>| {
        let base = "tabs-underline-btn";
        if active.get() == kind {
            format!("{base} tabs-underline-btn--active")
        } else {
            base.to_string()
        }
    };

    let activate = move |kind: Option<ItemKind>| {
        if active.get_untracked() == kind {
            return;
        }
        haptic_selection();
        on_change.run(kind);
    };

    let handle_keydown = move |ev: ev::KeyboardEvent| {
        let current = active_index();
        let target_index = match ev.key().as_str() {
            "ArrowLeft" if current > 0 => current - 1,
            "ArrowRight" if current + 1 < N_TABS => current + 1,
            "Home" => 0,
            "End" => N_TABS - 1,
            _ => return,
        };
        if target_index == current {
            return;
        }
        ev.prevent_default();
        let target_kind = kind_at(target_index);
        // Move DOM focus to the sibling tab so the visual focus ring tracks
        // the active tab — required by the WAI-ARIA tabs pattern.
        if let Some(target) = ev.target() {
            if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                if let Some(parent) = el.parent_element() {
                    let selector = data_tab_selector(&target_kind);
                    if let Some(sibling) = parent.query_selector(selector).ok().flatten() {
                        if let Some(sibling_el) = sibling.dyn_ref::<web_sys::HtmlElement>() {
                            let _ = sibling_el.focus();
                        }
                    }
                }
            }
        }
        activate(target_kind);
    };

    let render_tab = move |kind: Option<ItemKind>| {
        let label = label_for(&kind);
        let data_attr = data_tab_value(&kind);
        let kind_for_class = kind.clone();
        let kind_for_selected = kind.clone();
        let kind_for_tabindex = kind.clone();
        let kind_for_click = kind.clone();
        let selected = move || {
            if active.get() == kind_for_selected {
                "true"
            } else {
                "false"
            }
        };
        let tabindex = move || {
            if active.get() == kind_for_tabindex {
                "0"
            } else {
                "-1"
            }
        };
        view! {
            <button
                type="button"
                role="tab"
                data-tab=data_attr
                aria-selected=selected
                aria-controls="library-list"
                tabindex=tabindex
                class=move || tab_class(kind_for_class.clone())
                on:click=move |_| activate(kind_for_click.clone())
            >
                {label}
            </button>
        }
    };

    view! {
        <div
            class="tabs-underline"
            role="tablist"
            aria-label="Filter by type"
            style=indicator_style
            on:keydown=handle_keydown
        >
            // Sliding indicator — absolute-positioned, width = 1/n of the
            // tablist, x driven by --thumb-x set above. Behind the buttons.
            <div class="tabs-underline-indicator" aria-hidden="true" />
            {render_tab(None)}
            {render_tab(Some(ItemKind::Piece))}
            {render_tab(Some(ItemKind::Exercise))}
        </div>
    }
}

fn label_for(kind: &Option<ItemKind>) -> &'static str {
    match kind {
        None => "All",
        Some(ItemKind::Piece) => "Pieces",
        Some(ItemKind::Exercise) => "Exercises",
    }
}

fn data_tab_value(kind: &Option<ItemKind>) -> &'static str {
    match kind {
        None => "all",
        Some(ItemKind::Piece) => "piece",
        Some(ItemKind::Exercise) => "exercise",
    }
}

fn data_tab_selector(kind: &Option<ItemKind>) -> &'static str {
    match kind {
        None => "[data-tab='all']",
        Some(ItemKind::Piece) => "[data-tab='piece']",
        Some(ItemKind::Exercise) => "[data-tab='exercise']",
    }
}
