use leptos::prelude::*;
use leptos_router::components::A;

use intrada_core::model::GoalView;
use intrada_core::ViewModel;

use crate::components::{Card, LibraryItemCard};
use intrada_web::types::IsLoading;

#[component]
pub fn LibraryListView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();
    view! {
        <div class="space-y-6">
            // Active goals summary (hidden when no active goals)
            <ActiveGoalsSummary />

            // Hero section
            <section aria-labelledby="welcome-heading">
                <h2 id="welcome-heading" class="text-2xl font-bold text-primary mb-3 font-heading">
                    "Welcome to Intrada"
                </h2>
                <p class="text-sm text-secondary leading-relaxed max-w-2xl">
                    "Organize your music library, track your practice pieces and exercises, "
                    "and build better practice habits. Intrada helps musicians stay focused "
                    "on what matters \u{2014} making music."
                </p>
            </section>

            // Library section header
            <section aria-labelledby="library-heading">
                <div class="flex items-center justify-between mb-4">
                    <h2 id="library-heading" class="text-lg font-semibold text-primary">"Library"</h2>
                    <div class="flex items-center gap-3">
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

                        // Single "Add Item" button — navigates to unified form (FR-011)
                        <A href="/library/new" attr:class="cta-link">
                            <span aria-hidden="true" class="mr-1">"+"</span>
                            " Add Item"
                        </A>
                    </div>
                </div>

                // Items list
                <div>
                    {move || {
                        if is_loading.get() {
                            view! {
                                <div class="flex justify-center py-12">
                                    <div class="animate-spin rounded-full h-8 w-8 border-2 border-accent-focus border-t-transparent"></div>
                                </div>
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

/// Compact active goals summary card — shown on the library home page when there are active goals.
/// Displays up to 3 active goals with mini progress bars and a "View all goals" link.
#[component]
fn ActiveGoalsSummary() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();

    view! {
        {move || {
            let vm = view_model.get();
            let active: Vec<GoalView> = vm
                .goals
                .iter()
                .filter(|g| g.status == "active")
                .take(3)
                .cloned()
                .collect();

            if active.is_empty() {
                return view! { <div></div> }.into_any();
            }

            let total_active = vm.goals.iter().filter(|g| g.status == "active").count();

            view! {
                <Card>
                    <div class="space-y-3">
                        <div class="flex items-center justify-between">
                            <h3 class="card-title mb-0">"Active Goals"</h3>
                            <A href="/goals" attr:class="text-xs text-accent-text hover:text-accent-hover font-medium">
                                {if total_active > 3 {
                                    format!("View all {total_active} goals")
                                } else {
                                    "View all goals".to_string()
                                }}
                            </A>
                        </div>
                        <ul class="space-y-2.5" role="list">
                            {active.into_iter().map(|goal| {
                                let pct = goal.progress.as_ref().map(|p| p.percentage.clamp(0.0, 100.0)).unwrap_or(0.0);
                                let title = goal.title.clone();
                                view! {
                                    <li class="flex items-center gap-3">
                                        <div class="flex-1 min-w-0">
                                            <p class="text-sm text-secondary truncate">{title}</p>
                                            <div class="h-1.5 rounded-full bg-surface-secondary overflow-hidden mt-1">
                                                <div
                                                    class="h-full rounded-full bg-accent-focus transition-all duration-500"
                                                    style=format!("width: {pct:.0}%")
                                                />
                                            </div>
                                        </div>
                                        <span class="text-xs text-muted flex-shrink-0">{format!("{pct:.0}%")}</span>
                                    </li>
                                }
                            }).collect::<Vec<_>>()}
                        </ul>
                    </div>
                </Card>
            }.into_any()
        }}
    }
}
