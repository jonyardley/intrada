use leptos::ev;
use leptos::prelude::*;
use leptos::web_sys;
use wasm_bindgen::JsCast;

use intrada_core::{Event, ItemEvent, ItemKind, LibraryItemView, ViewModel};

use crate::components::{
    BottomSheet, EmptyState, Icon, IconName, LibraryItemCard, LibraryTypeTabs, PageHeading,
    PullToRefresh, SkeletonItemCard,
};
use crate::views::AddLibraryItemForm;
use intrada_web::core_bridge::process_effects_with_core;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Case-insensitive substring match over title, subtitle (composer), and tags.
/// Caller must lowercase `q` once before calling — keeps the per-item cost to
/// just the per-field lowercase + contains.
fn matches_query(item: &LibraryItemView, q: &str) -> bool {
    item.title.to_lowercase().contains(q)
        || item.subtitle.to_lowercase().contains(q)
        || item.tags.iter().any(|t| t.to_lowercase().contains(q))
}

#[component]
pub fn LibraryListView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let core = expect_context::<SharedCore>();
    let is_refreshing = RwSignal::new(false);
    let add_sheet_open = RwSignal::new(false);

    // Default to All so the user sees their whole library on first load.
    // `None` = no kind filter; `Some(kind)` filters to that kind.
    let active_filter: RwSignal<Option<ItemKind>> = RwSignal::new(None);
    let query = RwSignal::new(String::new());

    // Filtered view: by tab first, then by query (title / composer / tag,
    // case-insensitive, substring match). Empty query passes through.
    // Memo (not Signal::derive) so the filter runs once per dependency
    // change and caches for re-reads in the same render cycle (count span,
    // list, and empty-state branches all read it).
    let filtered_items = Memo::new(move |_| {
        let vm = view_model.get();
        let kind = active_filter.get();
        let q = query.get().trim().to_lowercase();
        vm.items
            .into_iter()
            .filter(|item| match &kind {
                None => true,
                Some(k) => &item.item_type == k,
            })
            .filter(|item| q.is_empty() || matches_query(item, &q))
            .collect::<Vec<_>>()
    });

    let open_add_sheet = Callback::new(move |_| add_sheet_open.set(true));
    let close_add_sheet = Callback::new(move |_| add_sheet_open.set(false));

    // Swipe-to-delete handler — invoked from each row's SwipeActions when
    // the user full-swipes or taps the revealed Delete button.
    let core_for_delete = core.clone();
    let on_delete_item = Callback::new(move |id: String| {
        let event = Event::Item(ItemEvent::Delete { id });
        let effects = {
            let core_ref = core_for_delete.borrow();
            core_ref.process_event(event)
        };
        process_effects_with_core(
            &core_for_delete,
            effects,
            &view_model,
            &is_loading,
            &is_submitting,
        );
    });

    let on_refresh = Callback::new(move |_| {
        // Skip if the initial app load is still in flight — the global
        // skeleton already covers that case.
        if is_loading.get_untracked() {
            return;
        }
        let effects = {
            let core_ref = core.borrow();
            core_ref.process_event(Event::RefetchItems)
        };
        // Use the with-core variant: this callback is invoked from a raw JS
        // touch event listener (no Leptos owner), so the expect_context inside
        // plain process_effects would panic.
        process_effects_with_core(&core, effects, &view_model, &is_loading, &is_submitting);
        is_refreshing.set(true);
    });

    // Clear the refresh spinner when the in-flight refetch completes.
    // Tied to is_submitting (per-mutation) rather than is_loading
    // (whole-app initial load) so a stuck initial load can't leave the
    // refresh spinner orphaned.
    Effect::new(move |_| {
        if is_refreshing.get() && !is_submitting.get() {
            is_refreshing.set(false);
        }
    });

    view! {
        <PullToRefresh on_refresh=on_refresh is_refreshing=is_refreshing>
        <div class="space-y-6">
            // Page heading matches the other top-level tabs (Practice,
            // Routines, Analytics). The "Add Item" trailing action lives
            // in PageHeading's trailing slot so it sits at the title's
            // level, not floating below the subtitle.
            //
            // The cta-link's icon/label children are CSS-swapped per
            // platform: web shows the "Add Item" pill, iOS shows the
            // "+" icon-only nav action.
            <PageHeading
                text="Library"
                subtitle="Your pieces and exercises."
                trailing=Box::new(move || view! {
                    <button
                        type="button"
                        class="cta-link cta-link--page-add shrink-0"
                        aria-label="Add Item"
                        on:click=move |_| open_add_sheet.run(())
                    >
                        <Icon name=IconName::Plus class="cta-link-icon" />
                        <span class="cta-link-label">"Add Item"</span>
                    </button>
                }.into_any())
            />

            // Search bar — title / composer / tag, case-insensitive,
            // substring match. Empty query falls through (tab still filters).
            <div class="search-bar">
                <Icon name=IconName::Search class="search-bar-icon" />
                <input
                    type="search"
                    class="search-bar-input"
                    placeholder="Search pieces..."
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
                        // mousedown fires before the input loses focus, so we
                        // can clear without blurring (which would hide the
                        // clear button before our click fires on iOS).
                        on:mousedown=move |ev| {
                            ev.prevent_default();
                            query.set(String::new());
                        }
                        on:touchstart=move |_| query.set(String::new())
                    >
                        "×"
                    </button>
                </Show>
            </div>

            // Type tabs — All / Pieces / Exercises. Underline-style with a
            // sliding accent indicator. Default active = All.
            <LibraryTypeTabs
                active=Signal::derive(move || active_filter.get())
                on_change=Callback::new(move |kind| active_filter.set(kind))
            />

            // Library items section. The page-level <PageHeading> above
            // already supplies the visible "Library" title, so the
            // section just carries an aria-label for screen readers and
            // an inline item count (reflects the *filtered* total).
            <section id="library-list" aria-label="Library items">
                <div class="flex justify-end mb-4">
                    <span class="text-sm text-muted">
                        {move || {
                            let count = filtered_items.get().len();
                            if count == 1 {
                                "1 item".to_string()
                            } else {
                                format!("{count} items")
                            }
                        }}
                    </span>
                </div>

                // Items list
                <div>
                    {move || {
                        if is_loading.get() {
                            view! {
                                <ul class="space-y-2 list-none p-0">
                                    <SkeletonItemCard />
                                    <SkeletonItemCard />
                                    <SkeletonItemCard />
                                    <SkeletonItemCard />
                                </ul>
                            }.into_any()
                        } else {
                            let vm = view_model.get();
                            let filtered = filtered_items.get();
                            // Three empty states:
                            //  1. truly empty library  — onboarding CTA
                            //  2. tab + non-empty query  — "no matches"
                            //  3. tab with no items but a non-empty other tab — neutral message
                            if vm.items.is_empty() {
                                view! {
                                    <EmptyState
                                        icon=IconName::Music
                                        title="No items in your library yet"
                                        body="Add a piece or exercise to get started."
                                    >
                                        <button
                                            type="button"
                                            class="cta-link"
                                            on:click=move |_| open_add_sheet.run(())
                                        >
                                            "Add Item"
                                        </button>
                                    </EmptyState>
                                }.into_any()
                            } else if filtered.is_empty() {
                                let q = query.get();
                                let trimmed = q.trim();
                                let (title, body) = if trimmed.is_empty() {
                                    // Empty filter and no query — the user
                                    // filtered to a kind they have none of.
                                    // All-tab + non-empty items is always
                                    // non-empty (truly-empty branch covers
                                    // the rest), so None is unreachable here.
                                    match active_filter.get() {
                                        Some(ItemKind::Piece) => (
                                            "No pieces yet".to_string(),
                                            "Switch tabs to see your other items, or add a new one.".to_string(),
                                        ),
                                        Some(ItemKind::Exercise) => (
                                            "No exercises yet".to_string(),
                                            "Switch tabs to see your other items, or add a new one.".to_string(),
                                        ),
                                        None => unreachable!(
                                            "All-tab + empty query is handled by the truly-empty branch"
                                        ),
                                    }
                                } else {
                                    let kind_label = match active_filter.get() {
                                        None => "items",
                                        Some(ItemKind::Piece) => "pieces",
                                        Some(ItemKind::Exercise) => "exercises",
                                    };
                                    (
                                        "No matching items".to_string(),
                                        format!("No {kind_label} match \u{201C}{trimmed}\u{201D}."),
                                    )
                                };
                                view! {
                                    <EmptyState
                                        icon=IconName::Search
                                        title=title
                                        body=body
                                    />
                                }.into_any()
                            } else {
                                view! {
                                    <ul class="space-y-2 list-none p-0" role="list" aria-label="Library items">
                                        {filtered.into_iter().map(|item| {
                                            view! {
                                                <LibraryItemCard item=item on_delete=on_delete_item />
                                            }
                                        }).collect::<Vec<_>>()}
                                    </ul>
                                }.into_any()
                            }
                        }
                    }}
                </div>
            </section>
        </div>
        </PullToRefresh>

        <BottomSheet
            open=add_sheet_open
            on_close=close_add_sheet
            nav_title="Add Item".to_string()
        >
            <AddLibraryItemForm in_sheet=true on_dismiss=close_add_sheet />
        </BottomSheet>
    }
}
