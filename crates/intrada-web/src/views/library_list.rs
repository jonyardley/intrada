use leptos::prelude::*;

use intrada_core::{Event, ItemEvent, ViewModel};

use crate::components::{
    BottomSheet, Icon, IconName, LibraryItemCard, PageHeading, PullToRefresh, SkeletonItemCard,
};
use crate::views::AddLibraryItemForm;
use intrada_web::core_bridge::process_effects_with_core;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

#[component]
pub fn LibraryListView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let core = expect_context::<SharedCore>();
    let is_refreshing = RwSignal::new(false);
    let add_sheet_open = RwSignal::new(false);

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

            // Library section header
            <section aria-labelledby="library-heading">
                <div class="flex items-center justify-between mb-4">
                    <h2 id="library-heading" class="text-lg font-semibold text-primary">"Library"</h2>
                    <span class="text-sm text-muted">
                        {move || {
                            let count = view_model.get().items.len();
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
                                <ul class="library-list grid grid-cols-1 sm:grid-cols-2 gap-3">
                                    <SkeletonItemCard />
                                    <SkeletonItemCard />
                                    <SkeletonItemCard />
                                    <SkeletonItemCard />
                                </ul>
                            }.into_any()
                        } else {
                            let vm = view_model.get();
                            if vm.items.is_empty() {
                                view! {
                                    <div class="empty-state text-center py-12">
                                        <svg class="empty-state-icon mx-auto mb-4 w-16 h-16 text-faint" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                                            <path d="M18 3a1 1 0 00-1.196-.98l-10 2A1 1 0 006 5v9.114A4.369 4.369 0 005 14c-1.657 0-3 .895-3 2s1.343 2 3 2 3-.895 3-2V7.82l8-1.6v5.894A4.37 4.37 0 0015 12c-1.657 0-3 .895-3 2s1.343 2 3 2 3-.895 3-2V3z" />
                                        </svg>
                                        <p class="empty-state-title text-base font-semibold text-secondary">"No items in your library yet"</p>
                                        <p class="text-sm text-faint mt-2 max-w-xs mx-auto">"Add a piece or exercise to get started."</p>
                                        <div class="mt-6">
                                            <button
                                                type="button"
                                                class="cta-link"
                                                on:click=move |_| open_add_sheet.run(())
                                            >
                                                "Add Item"
                                            </button>
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <ul class="library-list grid grid-cols-1 sm:grid-cols-2 gap-3" role="list" aria-label="Library items">
                                        {vm.items.into_iter().map(|item| {
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
