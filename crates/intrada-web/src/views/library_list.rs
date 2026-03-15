use leptos::prelude::*;
use leptos_router::components::A;

use intrada_core::ViewModel;

use crate::components::{LibraryItemCard, PageHeading, SkeletonItemCard};
use intrada_web::types::IsLoading;

#[component]
pub fn LibraryListView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();
    view! {
        <div class="space-y-6">
            // Hero section with CTA
            <div class="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-3">
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
                                <ul class="grid grid-cols-1 sm:grid-cols-2 gap-3">
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
                                    <div class="text-center py-12">
                                        <p class="text-muted">"No items in your library yet."</p>
                                        <p class="text-sm text-faint mt-2">"Add a piece or exercise to get started."</p>
                                        <div class="mt-6">
                                            <A href="/library/new" attr:class="cta-link">
                                                "Add Item"
                                            </A>
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <ul class="grid grid-cols-1 sm:grid-cols-2 gap-3" role="list" aria-label="Library items">
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
    }
}
