use leptos::prelude::*;
use leptos_router::components::A;

use intrada_core::ViewModel;

use crate::components::LibraryItemCard;

#[component]
pub fn LibraryListView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    view! {
        // Hero section
        <section class="mb-10 px-4 sm:px-6 lg:px-0" aria-labelledby="welcome-heading">
            <h2 id="welcome-heading" class="text-2xl font-semibold text-white mb-3">
                "Welcome to Intrada"
            </h2>
            <p class="text-gray-300 leading-relaxed max-w-2xl">
                "Organize your music library, track your practice pieces and exercises, "
                "and build better practice habits. Intrada helps musicians stay focused "
                "on what matters — making music."
            </p>
        </section>

        // Error banner
        {move || {
            view_model.get().error.map(|err| {
                view! {
                    <div class="mb-6 rounded-lg bg-red-500/10 border border-red-400/20 p-4" role="alert">
                        <p class="text-sm text-red-300">
                            <span class="font-medium">"Error: "</span>{err}
                        </p>
                    </div>
                }
            })
        }}

        // Library section header
        <section class="mb-10 px-4 sm:px-6 lg:px-0" aria-labelledby="library-heading">
            <div class="flex items-center justify-between mb-4">
                <h2 id="library-heading" class="text-lg font-semibold text-gray-200">"Library"</h2>
                <div class="flex items-center gap-3">
                    <span class="text-sm text-gray-400">
                        {move || {
                            let count = view_model.get().items.len();
                            if count == 1 {
                                "1 item".to_string()
                            } else {
                                format!("{count} items")
                            }
                        }}
                    </span>

                    // Single "Add Item" button — navigates to unified form (FR-011)
                    <A href="/library/new" attr:class="inline-flex items-center justify-center rounded-lg bg-indigo-600 px-4 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 motion-safe:transition-colors">
                        <span aria-hidden="true" class="mr-1">"+"</span>
                        " Add Item"
                    </A>
                </div>
            </div>

            // Items list (FR-006: clickable items)
            <div>
                {move || {
                    let vm = view_model.get();
                    if vm.items.is_empty() {
                        view! {
                            <div class="bg-white/5 rounded-xl border border-white/10 p-8 text-center">
                                <p class="text-gray-400">"No items in your library yet."</p>
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
                }}
            </div>
        </section>
    }
}
