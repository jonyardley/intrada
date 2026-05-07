use leptos::ev;
use leptos::prelude::*;
use leptos::web_sys;
use leptos_router::hooks::use_query_map;
use wasm_bindgen::JsCast;

use intrada_core::{Event, ItemEvent, ItemKind, LibraryItemView, SetEvent, SetView, ViewModel};

use crate::components::{
    BottomSheet, EmptyState, Icon, IconName, LibraryFilter, LibraryFilterTabs, LibraryItemCard,
    LibrarySetCard, PageAddButton, PageHeading, PullToRefresh, SkeletonItemCard,
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

/// Map a `?type=` query value to a `LibraryFilter`. Accepts the
/// canonical singulars (`piece`, `exercise`, `set`) plus the natural
/// plural / legacy forms so a deep link from any external surface
/// resolves the same way.
fn filter_from_query(value: &str) -> Option<LibraryFilter> {
    match value.trim().to_ascii_lowercase().as_str() {
        "all" => Some(LibraryFilter::All),
        "piece" | "pieces" => Some(LibraryFilter::Pieces),
        "exercise" | "exercises" => Some(LibraryFilter::Exercises),
        "set" | "sets" => Some(LibraryFilter::Sets),
        _ => None,
    }
}

/// Set-name search. Matches against the set's name + the title of every
/// entry inside it — so searching "Hanon" finds a set that *contains*
/// Hanon, not just sets named "Hanon".
fn matches_set_query(set: &SetView, q: &str) -> bool {
    set.name.to_lowercase().contains(q)
        || set
            .entries
            .iter()
            .any(|e| e.item_title.to_lowercase().contains(q))
}

#[component]
pub fn LibraryListView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let core = expect_context::<SharedCore>();
    let is_refreshing = RwSignal::new(false);
    let add_sheet_open = RwSignal::new(false);

    // Initial filter respects `?type=` so the /routines redirect (and any
    // other deep link) can land on the right tab. Unknown / missing →
    // default (All) so the user sees their whole library.
    //
    // Intentionally `with_untracked` — the URL is a deep-link entry
    // point, not a live source of truth. Once the user is on the page,
    // tab clicks own the filter; we don't want the URL changing back to
    // override their selection.
    let initial_filter = use_query_map()
        .with_untracked(|q| q.get("type").and_then(|v| filter_from_query(&v)))
        .unwrap_or_default();
    let active_filter: RwSignal<LibraryFilter> = RwSignal::new(initial_filter);
    let query = RwSignal::new(String::new());

    // Filtered items (pieces + exercises). Driven by the All / Pieces /
    // Exercises tabs — for the Sets tab, this returns empty and the
    // Set-list memo below takes over.
    // Memo (not Signal::derive) so the filter runs once per dependency
    // change and caches for re-reads in the same render cycle (count span,
    // list, and empty-state branches all read it).
    let filtered_items = Memo::new(move |_| {
        let vm = view_model.get();
        let filter = active_filter.get();
        let q = query.get().trim().to_lowercase();
        match filter {
            // Sets tab — items list is empty (the set list handles render)
            LibraryFilter::Sets => Vec::new(),
            _ => vm
                .items
                .into_iter()
                .filter(|item| {
                    matches!(
                        (filter, &item.item_type),
                        (LibraryFilter::All, _)
                            | (LibraryFilter::Pieces, ItemKind::Piece)
                            | (LibraryFilter::Exercises, ItemKind::Exercise)
                    )
                })
                .filter(|item| q.is_empty() || matches_query(item, &q))
                .collect::<Vec<_>>(),
        }
    });

    // Filtered Sets — only populated when the Sets tab is active.
    // Search matches set name + entry titles (so "Hanon" finds a set
    // that contains Hanon, not just sets named "Hanon").
    let filtered_sets = Memo::new(move |_| {
        let vm = view_model.get();
        if active_filter.get() != LibraryFilter::Sets {
            return Vec::new();
        }
        let q = query.get().trim().to_lowercase();
        vm.sets
            .into_iter()
            .filter(|s| q.is_empty() || matches_set_query(s, &q))
            .collect::<Vec<_>>()
    });

    let total_count = Memo::new(move |_| match active_filter.get() {
        LibraryFilter::Sets => filtered_sets.get().len(),
        _ => filtered_items.get().len(),
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

    let core_for_set_delete = core.clone();
    let on_delete_set = Callback::new(move |id: String| {
        let event = Event::Set(SetEvent::DeleteSet { id });
        let effects = {
            let core_ref = core_for_set_delete.borrow();
            core_ref.process_event(event)
        };
        process_effects_with_core(
            &core_for_set_delete,
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
            // Sets, Analytics). The "Add Item" trailing action lives
            // in PageHeading's trailing slot so it sits at the title's
            // level, not floating below the subtitle. Uses PageAddButton
            // (circular "+") for consistency across all top-level pages.
            <PageHeading
                text="Library"
                subtitle="Your pieces and exercises."
                trailing=Box::new(move || view! {
                    <PageAddButton
                        aria_label="Add Item"
                        on_click=Callback::new(move |_| open_add_sheet.run(()))
                    />
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

            // Type tabs — All / Pieces / Exercises / Sets. Underline-style
            // with a sliding accent indicator. Default active = All. The
            // 4-tab variant; the setlist builder uses the 3-tab
            // `<LibraryTypeTabs>` since Sets aren't items you can pick.
            <LibraryFilterTabs
                active=Signal::derive(move || active_filter.get())
                on_change=Callback::new(move |f| active_filter.set(f))
            />

            // Library items section. The page-level <PageHeading> above
            // already supplies the visible "Library" title, so the
            // section just carries an aria-label for screen readers and
            // an inline count (reflects the *filtered* total).
            <section id="library-list" aria-label="Library items">
                <div class="flex justify-end mb-4">
                    <span class="text-sm text-muted">
                        {move || {
                            let count = total_count.get();
                            let is_sets = active_filter.get() == LibraryFilter::Sets;
                            match (count, is_sets) {
                                (1, true) => "1 set".to_string(),
                                (n, true) => format!("{n} sets"),
                                (1, false) => "1 item".to_string(),
                                (n, false) => format!("{n} items"),
                            }
                        }}
                    </span>
                </div>

                // List body — branches on filter to render the right kind
                // of row (atomic items vs sets). The skeleton + empty
                // states are shared across both modes; the row component
                // is the only piece that changes.
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
                        } else if active_filter.get() == LibraryFilter::Sets {
                            // Sets tab — render Set rows.
                            let vm = view_model.get();
                            let filtered = filtered_sets.get();
                            if vm.sets.is_empty() {
                                view! {
                                    <EmptyState
                                        icon=IconName::ListChecks
                                        title="No saved sets yet"
                                        body="Save a setlist as a set to reuse it later."
                                    />
                                }.into_any()
                            } else if filtered.is_empty() {
                                let q = query.get();
                                let trimmed = q.trim();
                                let (title, body) = if trimmed.is_empty() {
                                    // Empty filter + no query is unreachable
                                    // here — the truly-empty branch above
                                    // catches `vm.sets.is_empty()`, and an
                                    // empty query short-circuits the filter
                                    // (matches_set_query) so a non-empty
                                    // sets vec always passes through.
                                    unreachable!(
                                        "Sets-tab + empty query is handled by the truly-empty branch"
                                    )
                                } else {
                                    ("No matching sets".to_string(),
                                     format!("No sets match \u{201C}{trimmed}\u{201D}."))
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
                                    <ul class="space-y-2 list-none p-0" role="list" aria-label="Library sets">
                                        {filtered.into_iter().map(|set| {
                                            view! {
                                                <LibrarySetCard set=set on_delete=on_delete_set />
                                            }
                                        }).collect::<Vec<_>>()}
                                    </ul>
                                }.into_any()
                            }
                        } else {
                            // Items tabs (All / Pieces / Exercises).
                            let vm = view_model.get();
                            let filtered = filtered_items.get();
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
                                    // Empty filter + no query — user
                                    // filtered to a kind they have none of.
                                    // All-tab is unreachable (truly-empty
                                    // covers it).
                                    match active_filter.get() {
                                        LibraryFilter::Pieces => (
                                            "No pieces yet".to_string(),
                                            "Switch tabs to see your other items, or add a new one.".to_string(),
                                        ),
                                        LibraryFilter::Exercises => (
                                            "No exercises yet".to_string(),
                                            "Switch tabs to see your other items, or add a new one.".to_string(),
                                        ),
                                        LibraryFilter::All => unreachable!(
                                            "All-tab + empty query is handled by the truly-empty branch"
                                        ),
                                        LibraryFilter::Sets => unreachable!(
                                            "Sets handled by the branch above"
                                        ),
                                    }
                                } else {
                                    let kind_label = match active_filter.get() {
                                        LibraryFilter::All => "items",
                                        LibraryFilter::Pieces => "pieces",
                                        LibraryFilter::Exercises => "exercises",
                                        LibraryFilter::Sets => unreachable!(
                                            "Sets handled by the branch above"
                                        ),
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

        <AddItemSheet open=add_sheet_open on_close=close_add_sheet is_submitting=is_submitting />
    }
}

/// Wraps `<AddLibraryItemForm>` inside a `<BottomSheet>` configured for the
/// iOS Mail-compose pattern: Cancel on the left of the nav bar, Save on
/// the right (triggers form submission via the form's NodeRef).
#[component]
fn AddItemSheet(
    open: RwSignal<bool>,
    on_close: Callback<()>,
    is_submitting: IsSubmitting,
) -> impl IntoView {
    let form_ref = NodeRef::<leptos::html::Form>::new();
    let on_save = Callback::new(move |_| {
        if let Some(form) = form_ref.get() {
            let _ = form.request_submit();
        }
    });
    let submitting_signal = Signal::derive(move || is_submitting.get());
    view! {
        <BottomSheet
            open=open
            on_close=on_close
            nav_title="Add Item".to_string()
            nav_action_label="Save".to_string()
            on_nav_action=on_save
            nav_action_disabled=submitting_signal
        >
            <AddLibraryItemForm in_sheet=true on_dismiss=on_close form_ref=form_ref />
        </BottomSheet>
    }
}
