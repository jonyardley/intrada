use leptos::prelude::*;
use leptos_router::components::A;

use intrada_core::ViewModel;

use crate::components::LibraryItemCard;

#[component]
pub fn LibraryListView(view_model: RwSignal<ViewModel>) -> impl IntoView {
    view! {
        // Hero section
        <section class="mb-10" aria-labelledby="welcome-heading">
            <h2 id="welcome-heading" class="text-2xl font-semibold text-slate-800 mb-3">
                "Welcome to Intrada"
            </h2>
            <p class="text-slate-600 leading-relaxed max-w-2xl">
                "Organize your music library, track your practice pieces and exercises, "
                "and build better practice habits. Intrada helps musicians stay focused "
                "on what matters \u{2014} making music."
            </p>
        </section>

        // Error banner
        {move || {
            view_model.get().error.map(|err| {
                view! {
                    <div class="mb-6 rounded-lg bg-red-50 border border-red-200 p-4" role="alert">
                        <p class="text-sm text-red-800">
                            <span class="font-medium">"Error: "</span>{err}
                        </p>
                    </div>
                }
            })
        }}

        // Status message
        {move || {
            view_model.get().status.map(|status| {
                view! {
                    <div class="mb-6 rounded-lg bg-blue-50 border border-blue-200 p-4" role="status">
                        <p class="text-sm text-blue-800">{status}</p>
                    </div>
                }
            })
        }}

        // Library section header
        <section class="mb-10" aria-labelledby="library-heading">
            <div class="flex items-center justify-between mb-4">
                <h2 id="library-heading" class="text-lg font-semibold text-slate-700">"Library"</h2>
                <div class="flex items-center gap-3">
                    <span class="text-sm text-slate-500">
                        {move || {
                            let count = view_model.get().item_count;
                            format!("{count} item(s)")
                        }}
                    </span>

                    // Single "Add Item" button — navigates to unified form (FR-011)
                    <A href="/library/new" attr:class="inline-flex items-center justify-center rounded-lg bg-indigo-600 px-4 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 transition-colors">
                        <span aria-hidden="true" class="mr-1">"+"</span>
                        " Add Item"
                    </A>
                </div>
            </div>

            // Items list (FR-006: clickable items)
            <div class="space-y-3">
                {move || {
                    let vm = view_model.get();
                    if vm.items.is_empty() {
                        view! {
                            <div class="bg-white rounded-xl border border-slate-200 p-8 text-center">
                                <p class="text-slate-400">"No items in your library yet."</p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <ul class="space-y-3" role="list" aria-label="Library items">
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
