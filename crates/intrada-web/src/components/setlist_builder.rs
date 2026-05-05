use std::collections::HashSet;

use leptos::ev;
use leptos::prelude::*;
use leptos::web_sys;
use wasm_bindgen::JsCast;

use intrada_core::{Event, ItemKind, LibraryItemView, SessionEvent, ViewModel};

use crate::components::{
    BuilderItemRow, Button, ButtonVariant, Icon, IconName, LibraryTypeTabs, SessionReviewSheet,
};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Case-insensitive substring match over title, subtitle, and tags.
fn matches_query(item: &LibraryItemView, q: &str) -> bool {
    item.title.to_lowercase().contains(q)
        || item.subtitle.to_lowercase().contains(q)
        || item.tags.iter().any(|t| t.to_lowercase().contains(q))
}

/// Setlist builder: library list with type-tab filter + search at the top,
/// a sticky bottom toolbar showing item count + total minutes, and a
/// "Review session" CTA that opens the [`SessionReviewSheet`].
///
/// The sheet is where reordering, intention, per-entry options and the
/// Start Session CTA live — picking items happens here on the library page.
#[component]
pub fn SetlistBuilder() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    let core_toggle = core.clone();
    let core_cancel = core.clone();

    // Library list filter state.
    let active_filter: RwSignal<Option<ItemKind>> = RwSignal::new(None);
    let query = RwSignal::new(String::new());

    // Sheet open state — owned here so the toolbar can open it.
    let review_open = RwSignal::new(false);
    let close_review = Callback::new(move |_| review_open.set(false));
    let open_review = move |_| review_open.set(true);

    // Set of item ids currently in the setlist — used to render the toggle
    // state on each library row.
    let selected_ids = Memo::new(move |_| {
        view_model
            .get()
            .building_setlist
            .as_ref()
            .map(|s| {
                s.entries
                    .iter()
                    .map(|e| e.item_id.clone())
                    .collect::<HashSet<String>>()
            })
            .unwrap_or_default()
    });

    // Filtered library items: tab → query.
    let filtered_items = Memo::new(move |_| {
        let vm = view_model.get();
        let kind = active_filter.get();
        let q = query.get().trim().to_lowercase();
        vm.items
            .into_iter()
            .filter(|i| match &kind {
                None => true,
                Some(k) => &i.item_type == k,
            })
            .filter(|i| q.is_empty() || matches_query(i, &q))
            .collect::<Vec<_>>()
    });

    // Toggle handler — adds the item if not present, removes it if it is.
    let on_toggle = Callback::new(move |item_id: String| {
        let vm = view_model.get();
        let entry_for_item = vm
            .building_setlist
            .as_ref()
            .and_then(|s| s.entries.iter().find(|e| e.item_id == item_id))
            .map(|e| e.id.clone());

        let event = match entry_for_item {
            Some(entry_id) => Event::Session(SessionEvent::RemoveFromSetlist { entry_id }),
            None => Event::Session(SessionEvent::AddToSetlist { item_id }),
        };
        let core_ref = core_toggle.borrow();
        let effects = core_ref.process_event(event);
        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
    });

    // Bottom-toolbar summary: item count + planned minutes.
    let summary = Signal::derive(move || {
        let vm = view_model.get();
        match &vm.building_setlist {
            Some(s) => {
                let count = s.entries.len();
                let total: u32 = s
                    .entries
                    .iter()
                    .map(|e| {
                        e.planned_duration_secs
                            .unwrap_or(intrada_core::validation::DEFAULT_PLANNED_DURATION_SECS)
                    })
                    .sum::<u32>()
                    / 60;
                (count, total)
            }
            None => (0, 0),
        }
    });

    let setlist_empty = Signal::derive(move || summary.get().0 == 0);

    view! {
        <div class="space-y-4 pb-32">
            // Search bar with built-in clear button (mirrors the library
            // list's affordance — clear the query without clearing tabs).
            <div class="search-bar">
                <Icon name=IconName::Search class="search-bar-icon" />
                <input
                    type="search"
                    class="search-bar-input"
                    placeholder="Search library..."
                    aria-label="Search library"
                    prop:value=move || query.get()
                    on:input=move |ev: ev::Event| {
                        if let Some(target) = ev.target() {
                            if let Some(input) = target.dyn_ref::<web_sys::HtmlInputElement>() {
                                query.set(input.value());
                            }
                        }
                    }
                />
                <Show when=move || !query.get().is_empty()>
                    <button
                        type="button"
                        class="search-bar-clear"
                        aria-label="Clear search"
                        on:mousedown=move |ev| {
                            ev.prevent_default();
                            query.set(String::new());
                        }
                        on:touchstart=move |_| query.set(String::new())
                    >"×"</button>
                </Show>
            </div>

            // Type tabs — All / Pieces / Exercises.
            <LibraryTypeTabs
                active=Signal::derive(move || active_filter.get())
                on_change=Callback::new(move |kind| active_filter.set(kind))
            />

            // Library list — tap a row to add/remove from the setlist.
            <div class="space-y-2">
                {move || {
                    let items = filtered_items.get();
                    if items.is_empty() {
                        let vm = view_model.get();
                        let msg = if vm.items.is_empty() {
                            "No library items yet. Add a piece or exercise first."
                        } else {
                            "No matches."
                        };
                        view! {
                            <p class="text-sm text-muted text-center py-6">{msg}</p>
                        }.into_any()
                    } else {
                        view! {
                            <div class="space-y-2">
                                {items.into_iter().map(|item| {
                                    let item_id = item.id.clone();
                                    let is_selected = Signal::derive(move || selected_ids.get().contains(&item_id));
                                    view! {
                                        <BuilderItemRow item=item is_selected=is_selected on_toggle=on_toggle />
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </div>

            // Cancel — kept here so the user can back out without leaving
            // the page; primary navigation away is "Start Session" inside
            // the sheet.
            <div class="flex justify-center pt-2">
                <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| {
                    let event = Event::Session(SessionEvent::CancelBuilding);
                    let core_ref = core_cancel.borrow();
                    let effects = core_ref.process_event(event);
                    process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                })>
                    "Cancel"
                </Button>
            </div>
        </div>

        // Sticky bottom toolbar — count + duration + Review CTA.
        // Tappable area extends across the count column so the user can
        // also expand the sheet by tapping the summary text.
        <div class="action-bar" role="toolbar" aria-label="Setlist summary">
            <button
                type="button"
                class="builder-toolbar-summary"
                on:click=open_review
                disabled=move || setlist_empty.get()
            >
                {move || {
                    let (count, total) = summary.get();
                    if count == 0 {
                        view! {
                            <span class="text-sm text-muted">"Tap items to build your setlist"</span>
                        }.into_any()
                    } else {
                        let plural = if count == 1 { "item" } else { "items" };
                        view! {
                            <span class="builder-toolbar-summary-text">
                                <span class="text-sm font-medium text-primary">{format!("{count} {plural} · {total} min")}</span>
                                <span class="text-xs text-muted">"Tap to review setlist"</span>
                            </span>
                        }.into_any()
                    }
                }}
            </button>
            <Button
                variant=ButtonVariant::Primary
                disabled=setlist_empty
                on_click=Callback::new(move |_| review_open.set(true))
            >
                "Review session"
            </Button>
        </div>

        <SessionReviewSheet open=review_open.into() on_close=close_review />
    }
}
