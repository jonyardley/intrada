use leptos::prelude::*;

use intrada_core::ViewModel;

use crate::components::{BackLink, PageHeading, SkeletonCardList};
use crate::views::sessions::SessionRow;
use intrada_web::types::{IsLoading, SharedCore};

/// Full chronological session list — all completed sessions.
///
/// Accessed via "Show all sessions" link from the week strip view.
/// Shows every session in the same card format as the week view,
/// ordered newest-first (same as ViewModel.sessions default order).
#[component]
pub fn SessionsAllView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();

    view! {
        <div>
            <BackLink href="/sessions".to_string() label="Back to week view" />

            <div class="mb-6">
                <PageHeading text="All Sessions" subtitle="Complete chronological history of your sessions." />
            </div>

            {move || {
                if is_loading.get() {
                    return view! {
                        <SkeletonCardList count=5 />
                    }.into_any();
                }

                let vm = view_model.get();

                if vm.sessions.is_empty() {
                    view! {
                        <div class="text-center py-12 px-4 sm:px-6 lg:px-0">
                            <p class="text-muted">"No sessions recorded yet."</p>
                            <p class="text-sm text-faint mt-2">"Start a session to begin tracking your progress."</p>
                        </div>
                    }.into_any()
                } else {
                    let core = core.clone();
                    let session_count = vm.sessions.len();
                    view! {
                        <div class="space-y-3">
                            {vm.sessions.iter().map(|session| {
                                view! {
                                    <SessionRow
                                        session=session.clone()
                                        core=core.clone()
                                        view_model=view_model
                                    />
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                        <p class="text-sm text-muted mt-4">
                            {format!("{} session{}", session_count, if session_count == 1 { "" } else { "s" })}
                        </p>
                    }.into_any()
                }
            }}
        </div>
    }
}
