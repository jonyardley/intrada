use leptos::ev;
use leptos::prelude::*;
use leptos::web_sys;
use wasm_bindgen::JsCast;

use intrada_web::haptics::haptic_selection;
use intrada_web::types::ItemType;

/// Horizontal toggle for selecting between Piece and Exercise types.
///
/// - When `on_change` is `Some`, the control is interactive (add form).
/// - When `on_change` is `None`, the control is display-only (edit form).
///
/// Web: pill-style toggle with accent fill on active segment.
/// iOS (`[data-platform="ios"]`): UISegmentedControl-style with a sliding
/// neutral thumb — no accent fill, monochrome text with weight bump.
#[component]
pub fn TypeTabs(
    active: Signal<ItemType>,
    #[prop(optional)] on_change: Option<Callback<ItemType>>,
) -> impl IntoView {
    let is_interactive = on_change.is_some();

    // Drives the sliding thumb position via CSS custom property.
    // On web the thumb is hidden (display:none); on iOS it slides.
    let thumb_style = move || {
        let offset = if active.get() == ItemType::Piece {
            "0%"
        } else {
            "100%"
        };
        format!("--thumb-x: {offset}")
    };

    // Build button classes. Web: individual pill fill. iOS: transparent bg
    // (overridden by .type-tabs-btn CSS); aria-selected drives font weight.
    let tab_class = move |tab: ItemType| {
        let is_active = active.get() == tab;
        let base = "type-tabs-btn relative z-10 flex-1 inline-flex items-center justify-center px-4 py-2 text-sm font-medium rounded-full motion-safe:transition-colors focus:outline-none focus:ring-2 focus:ring-accent-focus focus:ring-offset-0";
        if is_active {
            format!("{base} bg-accent text-primary shadow-sm")
        } else if is_interactive {
            format!("{base} text-muted hover:text-primary cursor-pointer")
        } else {
            format!("{base} text-faint cursor-default")
        }
    };

    let handle_click = move |tab: ItemType| {
        if let Some(cb) = on_change {
            haptic_selection();
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
                    haptic_selection();
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
            class="type-tabs relative inline-flex items-center rounded-full bg-surface-input p-1 gap-1"
            style=thumb_style
            on:keydown=handle_keydown
        >
            // Decorative sliding thumb — hidden on web, visible + animated on iOS.
            // Sits behind the buttons (z-0 vs buttons' z-10).
            <div class="type-tabs-thumb" aria-hidden="true" />

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
