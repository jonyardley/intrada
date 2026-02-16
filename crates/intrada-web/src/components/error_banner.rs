use leptos::prelude::*;

use intrada_core::{Event, ViewModel};

use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Global dismissible error banner that reads `ViewModel.error` and shows
/// a styled error message with a dismiss button.
#[component]
pub fn ErrorBanner() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    view! {
        {move || {
            view_model.get().error.map(|err| {
                let core = core.clone();
                view! {
                    <div class="mb-6 rounded-lg bg-red-500/10 border border-red-400/20 p-4" role="alert">
                        <div class="flex items-start justify-between gap-3">
                            <p class="text-sm text-red-300">
                                <span class="font-medium">"Error: "</span>{err}
                            </p>
                            <button
                                class="shrink-0 text-red-400 hover:text-red-300 text-xs font-medium"
                                on:click=move |_| {
                                    let core_ref = core.borrow();
                                    let effects = core_ref.process_event(Event::ClearError);
                                    process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                }
                            >
                                "Dismiss"
                            </button>
                        </div>
                    </div>
                }
            })
        }}
    }
}
