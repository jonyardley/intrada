use leptos::prelude::*;

use intrada_core::{Event, SetEvent, ViewModel};

use crate::components::{AccentBar, AccentRow, Button, ButtonVariant, Icon, IconName};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Collapsible section listing saved Sets that can be loaded into the
/// current building setlist. Collapsed by default — shows a summary
/// button that expands to reveal the full list. Scales gracefully with
/// many saved Sets.
///
/// Mounted in the SetlistBuilder, gated by the caller on `setlist_empty`
/// — once the user has picked items, the loader hides to avoid the
/// "merge vs replace" decision (#390).
#[component]
pub fn SetLoader() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    let expanded = RwSignal::new(false);

    view! {
        {move || {
            let vm = view_model.get();
            if vm.sets.is_empty() {
                None
            } else {
                let set_count = vm.sets.len();
                let core_load = core.clone();
                Some(view! {
                    <div class="space-y-2">
                        <button
                            type="button"
                            class="flex items-center gap-2 w-full text-left"
                            on:click=move |_| expanded.set(!expanded.get_untracked())
                        >
                            <span class=move || if expanded.get() {
                                "inline-flex transition-transform rotate-90"
                            } else {
                                "inline-flex transition-transform"
                            }>
                                <Icon
                                    name=IconName::ChevronRight
                                    class="w-4 h-4 text-muted".to_string()
                                />
                            </span>
                            <span class="section-title" style="margin-bottom: 0">
                                {format!("Saved Sets ({})", set_count)}
                            </span>
                        </button>
                        <Show when=move || expanded.get()>
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
                        </Show>
                    </div>
                })
            }
        }}
    }
}
