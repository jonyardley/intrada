use leptos::prelude::*;

use intrada_core::{Event, SetEvent, ViewModel};

use crate::components::Card;
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Shows saved sets and lets the user load one into the current setlist.
///
/// Each set is displayed as a row with name, entry count, and a Load button.
/// Only visible when at least one set exists.
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
                    <Card>
                        <h3 class="section-title">"Saved Sets"</h3>
                        <div class="space-y-2">
                            {vm.sets.iter().map(|set| {
                                let set_id = set.id.clone();
                                let name = set.name.clone();
                                let entry_count = set.entry_count;
                                let core_l = core_load.clone();
                                view! {
                                    <div class="flex items-center justify-between rounded-lg bg-surface-secondary px-3 py-2 hover:bg-surface-hover">
                                        <div class="flex items-center gap-2">
                                            <span class="text-sm text-primary font-medium">{name}</span>
                                            <span class="badge badge--accent">
                                                {format!("{} item{}", entry_count, if entry_count == 1 { "" } else { "s" })}
                                            </span>
                                        </div>
                                        <button
                                            class="text-xs font-medium text-accent-text hover:text-accent-hover px-2 py-1 rounded hover:bg-surface-secondary motion-safe:transition-colors motion-safe:duration-150"
                                            on:click=move |_| {
                                                let event = Event::Set(SetEvent::LoadSetIntoSetlist {
                                                    set_id: set_id.clone(),
                                                });
                                                let core_ref = core_l.borrow();
                                                let effects = core_ref.process_event(event);
                                                process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                            }
                                        >
                                            "Load"
                                        </button>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </Card>
                })
            }
        }}
    }
}
