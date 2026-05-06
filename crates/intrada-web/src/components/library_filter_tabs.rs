use leptos::ev;
use leptos::prelude::*;
use leptos::web_sys;
use wasm_bindgen::JsCast;

use intrada_web::haptics::haptic_selection;

/// The four states the Library page's type filter can be in.
///
/// Distinct from `LibraryTypeTabs`'s `Option<ItemKind>` (which has only
/// three states — All / Pieces / Exercises) because Sets aren't an
/// `ItemKind` — they're a separate domain model. The Library list view
/// uses this enum to decide whether to render atomic-item rows
/// (LibraryItemCard, gold/blue bar) or Set rows (LibrarySetCard, teal bar).
///
/// The `setlist_builder` uses the older `LibraryTypeTabs` (3 tabs)
/// because adding a Set entry to a setlist would be ill-defined — Sets
/// are loaded via a separate "Load Set" affordance that appends the
/// set's entries, not picked as if they were items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LibraryFilter {
    /// Show pieces + exercises only. Sets live behind the Sets tab.
    #[default]
    All,
    /// Pieces only.
    Pieces,
    /// Exercises only.
    Exercises,
    /// Sets only.
    Sets,
}

/// Underline-style tabs with a sliding accent indicator. Four states:
/// All / Pieces / Exercises / Sets.
///
/// 4-tab variant of `LibraryTypeTabs` — same visual treatment (sliding
/// 3px accent indicator, equal-width buttons, WAI-ARIA tabs pattern,
/// haptics on change) but with a fourth tab for Sets. Used on the
/// Library page where Sets sit alongside atomic items.
///
/// `--tab-count: 4` is set inline so the indicator slides to
/// 0% / 100% / 200% / 300% via the existing `--thumb-x` mechanism.
#[component]
pub fn LibraryFilterTabs(
    /// Active filter.
    active: Signal<LibraryFilter>,
    on_change: Callback<LibraryFilter>,
) -> impl IntoView {
    fn filter_at(index: usize) -> LibraryFilter {
        match index {
            0 => LibraryFilter::All,
            1 => LibraryFilter::Pieces,
            2 => LibraryFilter::Exercises,
            _ => LibraryFilter::Sets,
        }
    }
    fn index_of(f: LibraryFilter) -> usize {
        match f {
            LibraryFilter::All => 0,
            LibraryFilter::Pieces => 1,
            LibraryFilter::Exercises => 2,
            LibraryFilter::Sets => 3,
        }
    }
    const N_TABS: usize = 4;

    let active_index = move || index_of(active.get());

    let indicator_style = move || {
        let pct = active_index() * 100;
        // --tab-count overrides the .tabs-underline default of 3 so the
        // indicator width + slide steps recompute for the 4-tab variant.
        format!("--tab-count: {N_TABS}; --thumb-x: {pct}%")
    };

    let tab_class = move |f: LibraryFilter| {
        let base = "tabs-underline-btn";
        if active.get() == f {
            format!("{base} tabs-underline-btn--active")
        } else {
            base.to_string()
        }
    };

    let activate = move |f: LibraryFilter| {
        if active.get_untracked() == f {
            return;
        }
        haptic_selection();
        on_change.run(f);
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
        let target_filter = filter_at(target_index);
        if let Some(target) = ev.target() {
            if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                if let Some(parent) = el.parent_element() {
                    let selector = data_tab_selector(target_filter);
                    if let Some(sibling) = parent.query_selector(selector).ok().flatten() {
                        if let Some(sibling_el) = sibling.dyn_ref::<web_sys::HtmlElement>() {
                            let _ = sibling_el.focus();
                        }
                    }
                }
            }
        }
        activate(target_filter);
    };

    let render_tab = move |f: LibraryFilter| {
        let label = label_for(f);
        let data_attr = data_tab_value(f);
        let selected = move || {
            if active.get() == f {
                "true"
            } else {
                "false"
            }
        };
        let tabindex = move || {
            if active.get() == f {
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
                class=move || tab_class(f)
                on:click=move |_| activate(f)
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
            <div class="tabs-underline-indicator" aria-hidden="true" />
            {render_tab(LibraryFilter::All)}
            {render_tab(LibraryFilter::Pieces)}
            {render_tab(LibraryFilter::Exercises)}
            {render_tab(LibraryFilter::Sets)}
        </div>
    }
}

fn label_for(f: LibraryFilter) -> &'static str {
    match f {
        LibraryFilter::All => "All",
        LibraryFilter::Pieces => "Pieces",
        LibraryFilter::Exercises => "Exercises",
        LibraryFilter::Sets => "Sets",
    }
}

fn data_tab_value(f: LibraryFilter) -> &'static str {
    match f {
        LibraryFilter::All => "all",
        LibraryFilter::Pieces => "piece",
        LibraryFilter::Exercises => "exercise",
        LibraryFilter::Sets => "set",
    }
}

fn data_tab_selector(f: LibraryFilter) -> &'static str {
    match f {
        LibraryFilter::All => "button[data-tab='all']",
        LibraryFilter::Pieces => "button[data-tab='piece']",
        LibraryFilter::Exercises => "button[data-tab='exercise']",
        LibraryFilter::Sets => "button[data-tab='set']",
    }
}
