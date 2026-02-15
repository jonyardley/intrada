use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use intrada_core::{Event, SessionEvent, ViewModel};

use crate::components::{Button, ButtonVariant, Card, SetlistEntryRow, TypeBadge};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Active session timer: shows current item, elapsed time, progress, and controls.
#[component]
pub fn SessionTimer() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    let elapsed_secs = RwSignal::new(0u32);
    let interval_id: RwSignal<Option<i32>> = RwSignal::new(None);

    // Start the display timer
    {
        let closure = Closure::<dyn Fn()>::new(move || {
            elapsed_secs.update(|s| *s += 1);
        });
        if let Some(window) = web_sys::window() {
            if let Ok(id) = window.set_interval_with_callback_and_timeout_and_arguments_0(
                closure.as_ref().unchecked_ref(),
                1000,
            ) {
                interval_id.set(Some(id));
            }
        }
        closure.forget();
    }

    on_cleanup(move || {
        if let Some(id) = interval_id.get_untracked() {
            if let Some(window) = web_sys::window() {
                window.clear_interval_with_handle(id);
            }
        }
    });

    view! {
        <div class="space-y-4">
            {move || {
                let vm = view_model.get();
                match vm.active_session {
                    Some(ref active) => {
                        let core_next = core.clone();
                        let core_skip = core.clone();
                        let core_finish = core.clone();
                        let core_end = core.clone();
                        let current_title = active.current_item_title.clone();
                        let current_type = active.current_item_type.clone();
                        let position = active.current_position;
                        let total = active.total_items;
                        let is_last = position == total - 1;
                        let completed_entries: Vec<_> = active.entries.iter()
                            .filter(|e| e.status == "completed" || e.status == "skipped")
                            .cloned()
                            .collect();

                        view! {
                            // Current item card
                            <Card>
                                <div class="text-center space-y-3">
                                    <p class="text-xs text-gray-400 uppercase tracking-wider">
                                        {format!("Item {} of {}", position + 1, total)}
                                    </p>
                                    <h2 class="text-2xl font-bold text-white">{current_title}</h2>
                                    <TypeBadge item_type=current_type />
                                    <p class="text-4xl sm:text-6xl font-mono font-bold text-white mt-4">
                                        {move || {
                                            let secs = elapsed_secs.get();
                                            format!("{:02}:{:02}", secs / 60, secs % 60)
                                        }}
                                    </p>
                                </div>
                            </Card>

                            // Controls
                            <div class="flex flex-wrap gap-3 justify-center">
                                {if is_last {
                                    view! {
                                        <Button variant=ButtonVariant::Primary on_click=Callback::new(move |_| {
                                            let now = chrono::Utc::now();
                                            let event = Event::Session(SessionEvent::FinishSession { now });
                                            let core_ref = core_finish.borrow();
                                            let effects = core_ref.process_event(event);
                                            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                            elapsed_secs.set(0);
                                        })>
                                            "Finish Session"
                                        </Button>
                                    }.into_any()
                                } else {
                                    view! {
                                        <Button variant=ButtonVariant::Primary on_click=Callback::new(move |_| {
                                            let now = chrono::Utc::now();
                                            let event = Event::Session(SessionEvent::NextItem { now });
                                            let core_ref = core_next.borrow();
                                            let effects = core_ref.process_event(event);
                                            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                            elapsed_secs.set(0);
                                        })>
                                            "Next Item"
                                        </Button>
                                    }.into_any()
                                }}
                                <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| {
                                    let now = chrono::Utc::now();
                                    let event = Event::Session(SessionEvent::SkipItem { now });
                                    let core_ref = core_skip.borrow();
                                    let effects = core_ref.process_event(event);
                                    process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                    elapsed_secs.set(0);
                                })>
                                    "Skip"
                                </Button>
                                <Button variant=ButtonVariant::DangerOutline on_click=Callback::new(move |_| {
                                    let now = chrono::Utc::now();
                                    let event = Event::Session(SessionEvent::EndSessionEarly { now });
                                    let core_ref = core_end.borrow();
                                    let effects = core_ref.process_event(event);
                                    process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                    elapsed_secs.set(0);
                                })>
                                    "End Early"
                                </Button>
                            </div>

                            // Completed items
                            {if !completed_entries.is_empty() {
                                Some(view! {
                                    <div class="mt-4">
                                        <h4 class="text-sm font-medium text-gray-400 mb-2">"Completed"</h4>
                                        <div class="space-y-1">
                                            {completed_entries.into_iter().map(|entry| {
                                                view! {
                                                    <SetlistEntryRow entry=entry show_controls=false />
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    </div>
                                })
                            } else {
                                None
                            }}
                        }.into_any()
                    }
                    None => {
                        view! {
                            <p class="text-sm text-gray-400 text-center py-8">"No active session."</p>
                        }.into_any()
                    }
                }
            }}
        </div>
    }
}
