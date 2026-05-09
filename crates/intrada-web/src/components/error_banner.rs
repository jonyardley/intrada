use leptos::prelude::*;
use leptos::task::spawn_local;

use intrada_core::{Event, ViewModel};

use crate::components::{Icon, IconName};
use intrada_web::core_bridge::process_effects;
use intrada_web::haptics::haptic_selection;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Slide-out animation duration must match the `error-banner-slide-out`
/// keyframe in input.css. Bumping one without the other will either cut
/// the animation short or leave the banner sitting in its dismissing
/// state for a beat before unmounting.
const DISMISS_ANIM_MS: u32 = 280;

/// Global dismissible error banner that reads `ViewModel.error`.
///
/// On iOS, sits as a fixed-top notification with glass blur and a
/// circular X dismiss control; on web, stays as an inline alert. Both
/// platforms get a slide-out animation when dismissed — the banner
/// keeps its `is-dismissing` class for one animation cycle before
/// the underlying error state actually clears.
///
/// Mount stability matters (#346): the wrapper `<div class="error-banner">`
/// is rendered through `<Show>` so it stays mounted across Some→Some
/// transitions in the underlying error signal. Without this, Leptos would
/// reconcile by replacing the node and the slide-in keyframe would re-fire
/// each time a new error message arrived. The slide-in only plays on the
/// None → Some transition; subsequent message updates just swap the inner
/// text in place.
#[component]
pub fn ErrorBanner() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    let is_dismissing = RwSignal::new(false);

    let has_error = Memo::new(move |_| view_model.get().error.is_some());
    let error_text = Memo::new(move |_| view_model.get().error.unwrap_or_default());

    view! {
        <Show when=move || has_error.get() fallback=|| ()>
            <div
                class=move || {
                    let base = "error-banner rounded-lg bg-danger-surface border border-danger-text/20 p-4";
                    if is_dismissing.get() {
                        format!("{base} is-dismissing")
                    } else {
                        base.to_string()
                    }
                }
                role="alert"
            >
                <div class="flex items-start justify-between gap-3">
                    <p class="text-sm text-danger-text">
                        <span class="font-medium">"Error: "</span>
                        {move || error_text.get()}
                    </p>
                    <button
                        class="error-banner-dismiss shrink-0"
                        aria-label="Dismiss error"
                        on:click={
                            let core = core.clone();
                            move |_| {
                                if is_dismissing.get_untracked() {
                                    return;
                                }
                                haptic_selection();
                                is_dismissing.set(true);
                                let core = core.clone();
                                spawn_local(async move {
                                    gloo_timers::future::TimeoutFuture::new(DISMISS_ANIM_MS).await;
                                    is_dismissing.set(false);
                                    let core_ref = core.borrow();
                                    let effects = core_ref.process_event(Event::ClearError);
                                    process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                });
                            }
                        }
                    >
                        <Icon name=IconName::X class="w-4 h-4" />
                    </button>
                </div>
            </div>
        </Show>
    }
}
