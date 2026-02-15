use leptos::prelude::*;

use intrada_core::{Event, SessionEvent, ViewModel};

use crate::components::{Button, ButtonVariant, Card, SetlistEntryRow};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::SharedCore;

/// Setlist builder component: shows library items to add, current setlist, and controls.
#[component]
pub fn SetlistBuilder() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();

    let core_setlist = core.clone();
    let core_actions = core.clone();
    let core_library = core.clone();

    view! {
        <div class="space-y-6">
            // Current setlist
            <Card>
                <h3 class="text-lg font-semibold text-slate-900 mb-4">"Your Setlist"</h3>
                {move || {
                    let vm = view_model.get();
                    match vm.building_setlist {
                        Some(ref setlist) if !setlist.entries.is_empty() => {
                            let core_remove = core_setlist.clone();
                            let core_up = core.clone();
                            let core_down = core.clone();
                            let entries = setlist.entries.clone();
                            let entry_count = entries.len();
                            view! {
                                <div class="space-y-2">
                                    {entries.into_iter().enumerate().map(|(idx, entry)| {
                                        let core_r = core_remove.clone();
                                        let core_u = core_up.clone();
                                        let core_d = core_down.clone();
                                        let on_remove = Callback::new(move |entry_id: String| {
                                            let event = Event::Session(SessionEvent::RemoveFromSetlist { entry_id });
                                            let core_ref = core_r.borrow();
                                            let effects = core_ref.process_event(event);
                                            process_effects(&core_ref, effects, &view_model);
                                        });
                                        let on_move_up = if idx > 0 {
                                            let core_mu = core_u.clone();
                                            Some(Callback::new(move |entry_id: String| {
                                                let event = Event::Session(SessionEvent::ReorderSetlist { entry_id, new_position: idx - 1 });
                                                let core_ref = core_mu.borrow();
                                                let effects = core_ref.process_event(event);
                                                process_effects(&core_ref, effects, &view_model);
                                            }))
                                        } else {
                                            None
                                        };
                                        let on_move_down = if idx < entry_count - 1 {
                                            let core_md = core_d.clone();
                                            Some(Callback::new(move |entry_id: String| {
                                                let event = Event::Session(SessionEvent::ReorderSetlist { entry_id, new_position: idx + 1 });
                                                let core_ref = core_md.borrow();
                                                let effects = core_ref.process_event(event);
                                                process_effects(&core_ref, effects, &view_model);
                                            }))
                                        } else {
                                            None
                                        };
                                        view! {
                                            <SetlistEntryRow
                                                entry=entry
                                                on_remove=Some(on_remove)
                                                on_move_up=on_move_up
                                                on_move_down=on_move_down
                                            />
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                        _ => {
                            view! {
                                <p class="text-sm text-slate-500 text-center py-4">
                                    "No items added yet. Select items from your library below."
                                </p>
                            }.into_any()
                        }
                    }
                }}
            </Card>

            // Action buttons
            <div class="flex gap-3">
                {
                    let core_start = core_actions.clone();
                    let core_cancel = core_actions.clone();
                    view! {
                        <Button variant=ButtonVariant::Primary on_click=Callback::new(move |_| {
                            let now = chrono::Utc::now();
                            let event = Event::Session(SessionEvent::StartSession { now });
                            let core_ref = core_start.borrow();
                            let effects = core_ref.process_event(event);
                            process_effects(&core_ref, effects, &view_model);
                        })>
                            "Start Session"
                        </Button>
                        <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| {
                            let event = Event::Session(SessionEvent::CancelBuilding);
                            let core_ref = core_cancel.borrow();
                            let effects = core_ref.process_event(event);
                            process_effects(&core_ref, effects, &view_model);
                        })>
                            "Cancel"
                        </Button>
                    }
                }
            </div>

            // Error display
            {move || {
                let vm = view_model.get();
                vm.error.map(|err| {
                    view! {
                        <p class="text-sm text-red-600">{err}</p>
                    }
                })
            }}

            // Library items to add
            <Card>
                <h3 class="text-lg font-semibold text-slate-900 mb-4">"Library Items"</h3>
                {move || {
                    let vm = view_model.get();
                    if vm.items.is_empty() {
                        view! {
                            <p class="text-sm text-slate-500">"No library items available."</p>
                        }.into_any()
                    } else {
                        let core_add = core_library.clone();
                        view! {
                            <div class="space-y-2">
                                {vm.items.iter().map(|item| {
                                    let item_id = item.id.clone();
                                    let title = item.title.clone();
                                    let item_type = item.item_type.clone();
                                    let core_a = core_add.clone();
                                    view! {
                                        <div class="flex items-center justify-between rounded-lg border border-slate-100 px-3 py-2 hover:bg-slate-50">
                                            <div class="flex items-center gap-2">
                                                <span class="text-sm text-slate-900">{title}</span>
                                                <span class="text-xs text-slate-400">{item_type}</span>
                                            </div>
                                            <button
                                                class="text-xs font-medium text-indigo-600 hover:text-indigo-800"
                                                on:click=move |_| {
                                                    let event = Event::Session(SessionEvent::AddToSetlist { item_id: item_id.clone() });
                                                    let core_ref = core_a.borrow();
                                                    let effects = core_ref.process_event(event);
                                                    process_effects(&core_ref, effects, &view_model);
                                                }
                                            >
                                                "+ Add"
                                            </button>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    }
                }}
            </Card>
        </div>
    }
}
