use leptos::prelude::*;

use intrada_core::{Event, SetEvent, ViewModel};

use crate::components::{AccentBar, AccentRow, Button, ButtonVariant};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Lists the user's saved Sets and lets them load one into the current
/// (empty) building setlist. Renders nothing when there are no saved Sets.
///
/// Mounted at the top of the SetlistBuilder, gated by the caller on
/// `setlist_empty` — once the user has picked items, the loader hides so
/// the "load vs current selection" decision doesn't arise. Merge/replace
/// is deferred until that flow is needed (see #390).
///
/// Each row mirrors the LibrarySetCard chrome (AccentRow with the Teal
/// bar that signals Set content) so saved Sets read consistently across
/// the Library and the builder.
#[component]
pub fn SetLoader() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    view! {
        {move || {
            let vm = view_model.get();
            if vm.sets.is_empty() {
                None
            } else {
                let core_load = core.clone();
                Some(view! {
                    <div class="space-y-2">
                        <h3 class="section-title">"Saved Sets"</h3>
                        <ul class="space-y-2">
                            {vm.sets.iter().map(|set| {
                                let set_id = set.id.clone();
                                let name = set.name.clone();
                                let entry_count = set.entry_count;
                                let count_label = if entry_count == 1 {
                                    "1 item".to_string()
                                } else {
                                    format!("{entry_count} items")
                                };
                                let core_l = core_load.clone();
                                let on_load = Callback::new(move |_| {
                                    let event = Event::Set(SetEvent::LoadSetIntoSetlist {
                                        set_id: set_id.clone(),
                                    });
                                    let core_ref = core_l.borrow();
                                    let effects = core_ref.process_event(event);
                                    process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                });
                                view! {
                                    <li>
                                        <AccentRow bar=AccentBar::Teal>
                                            <div class="flex flex-col flex-1 min-w-0 gap-0.5">
                                                <span class="text-sm font-semibold text-primary truncate">{name}</span>
                                                <span class="text-xs text-muted truncate">{count_label}</span>
                                            </div>
                                            <Button
                                                variant=ButtonVariant::Secondary
                                                on_click=on_load
                                            >
                                                "Load"
                                            </Button>
                                        </AccentRow>
                                    </li>
                                }
                            }).collect::<Vec<_>>()}
                        </ul>
                    </div>
                })
            }
        }}
    }
}
