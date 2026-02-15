use leptos::ev;
use leptos::prelude::*;
use leptos::web_sys;
use wasm_bindgen::JsCast;

use intrada_web::types::ItemType;

/// Horizontal toggle switch for selecting between Piece and Exercise types.
///
/// - When `on_change` is `Some`, the switch is interactive (add form).
/// - When `on_change` is `None`, the switch is display-only (edit form).
#[component]
pub fn TypeTabs(
    active: Signal<ItemType>,
    #[prop(optional)] on_change: Option<Callback<ItemType>>,
) -> impl IntoView {
    let is_interactive = on_change.is_some();

    // Helper to build class strings — pill-style segmented control
    let tab_class = move |tab: ItemType| {
        let is_active = active.get() == tab;
        let base = "relative z-10 flex-1 inline-flex items-center justify-center px-4 py-2 text-sm font-medium rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-indigo-400 focus:ring-offset-0";
        if is_active {
            format!("{base} bg-indigo-600 text-white shadow-sm")
        } else if is_interactive {
            format!("{base} text-gray-400 hover:text-white cursor-pointer")
        } else {
            // Display-only inactive
            format!("{base} text-gray-500 cursor-default")
        }
    };

    let handle_click = move |tab: ItemType| {
        if let Some(cb) = on_change {
            cb.run(tab);
        }
    };

    let handle_keydown = move |ev: ev::KeyboardEvent| {
        if !is_interactive {
            return;
        }
        let key = ev.key();
        match key.as_str() {
            "ArrowLeft" | "ArrowRight" => {
                ev.prevent_default();
                let new_tab = match active.get() {
                    ItemType::Piece => ItemType::Exercise,
                    ItemType::Exercise => ItemType::Piece,
                };
                // Move focus to the other tab button
                if let Some(target) = ev.target() {
                    if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                        if let Some(parent) = el.parent_element() {
                            let selector = match new_tab {
                                ItemType::Piece => "[data-tab='piece']",
                                ItemType::Exercise => "[data-tab='exercise']",
                            };
                            if let Some(sibling) = parent.query_selector(selector).ok().flatten() {
                                if let Some(sibling_el) = sibling.dyn_ref::<web_sys::HtmlElement>()
                                {
                                    let _ = sibling_el.focus();
                                }
                            }
                        }
                    }
                }
                if let Some(cb) = on_change {
                    cb.run(new_tab);
                }
            }
            _ => {}
        }
    };

    let piece_tabindex = move || {
        if active.get() == ItemType::Piece {
            "0"
        } else {
            "-1"
        }
    };
    let exercise_tabindex = move || {
        if active.get() == ItemType::Exercise {
            "0"
        } else {
            "-1"
        }
    };

    let piece_selected = move || {
        if active.get() == ItemType::Piece {
            "true"
        } else {
            "false"
        }
    };
    let exercise_selected = move || {
        if active.get() == ItemType::Exercise {
            "true"
        } else {
            "false"
        }
    };

    let piece_disabled = move || {
        if !is_interactive && active.get() != ItemType::Piece {
            "true"
        } else {
            "false"
        }
    };
    let exercise_disabled = move || {
        if !is_interactive && active.get() != ItemType::Exercise {
            "true"
        } else {
            "false"
        }
    };

    view! {
        <div
            role="tablist"
            aria-label="Item type"
            class="inline-flex items-center rounded-full bg-white/10 p-1 gap-1"
            on:keydown=handle_keydown
        >
            <button
                type="button"
                role="tab"
                id="tab-piece"
                data-tab="piece"
                aria-selected=piece_selected
                aria-controls="tabpanel-content"
                aria-disabled=piece_disabled
                tabindex=piece_tabindex
                class=move || tab_class(ItemType::Piece)
                on:click=move |_| handle_click(ItemType::Piece)
            >
                "Piece"
            </button>
            <button
                type="button"
                role="tab"
                id="tab-exercise"
                data-tab="exercise"
                aria-selected=exercise_selected
                aria-controls="tabpanel-content"
                aria-disabled=exercise_disabled
                tabindex=exercise_tabindex
                class=move || tab_class(ItemType::Exercise)
                on:click=move |_| handle_click(ItemType::Exercise)
            >
                "Exercise"
            </button>
        </div>
    }
}
