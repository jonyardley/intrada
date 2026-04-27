use leptos::prelude::*;
use leptos_router::components::A;

use intrada_core::ViewModel;

use crate::components::{LibraryItemCard, PageHeading, PullToRefresh, SkeletonItemCard};
use intrada_web::core_bridge::init_core;
use intrada_web::types::{IsLoading, IsSubmitting};

#[component]
pub fn LibraryListView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let is_refreshing = RwSignal::new(false);

    let on_refresh = Callback::new(move |_| {
        is_refreshing.set(true);
        init_core(&view_model, &is_loading, &is_submitting);
    });

    // Hide the refresh spinner once the load actually completes. Watches
    // is_loading rather than using a fixed delay so the spinner accurately
    // reflects the network round-trip.
    Effect::new(move |_| {
        if is_refreshing.get() && !is_loading.get() {
            is_refreshing.set(false);
        }
    });

    view! {
        <PullToRefresh on_refresh=on_refresh is_refreshing=is_refreshing>
        <div class="space-y-6">
            // Hero section with CTA
            <div class="library-hero flex flex-col sm:flex-row sm:items-start sm:justify-between gap-3">
                <PageHeading
                    text="Welcome to Intrada"
                    subtitle="Organize your music library, track your practice pieces and exercises, and build better practice habits."
                />
                <A href="/library/new" attr:class="cta-link shrink-0">
                    "Add Item"
                </A>
            </div>

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
                                            <A href="/library/new" attr:class="cta-link">
                                                "Add Item"
                                            </A>
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <ul class="library-list grid grid-cols-1 sm:grid-cols-2 gap-3" role="list" aria-label="Library items">
                                        {vm.items.into_iter().map(|item| {
                                            view! {
                                                <LibraryItemCard item=item />
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
    }
}
