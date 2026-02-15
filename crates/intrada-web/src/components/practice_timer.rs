use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use intrada_core::{Event, LogSession, SessionEvent, ViewModel};

use crate::components::{Button, ButtonVariant};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::SharedCore;

/// Client-side practice timer. Uses Leptos signals + web_sys intervals (not Crux core).
/// When stopped, rounds elapsed time to nearest minute and logs a session.
#[component]
pub fn PracticeTimer(item_id: String) -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();

    let running = RwSignal::new(false);
    let elapsed_secs = RwSignal::new(0u32);
    let notes = RwSignal::new(String::new());
    let show_save_form = RwSignal::new(false);
    let error_msg: RwSignal<Option<String>> = RwSignal::new(None);
    let interval_id: RwSignal<Option<i32>> = RwSignal::new(None);

    let item_id_for_save = item_id.clone();

    on_cleanup(move || {
        if let Some(id) = interval_id.get_untracked() {
            if let Some(window) = web_sys::window() {
                window.clear_interval_with_handle(id);
            }
        }
    });

    view! {
        <div class="mt-6">
            {move || {
                if show_save_form.get() {
                    let secs = elapsed_secs.get();
                    let minutes = (secs + 30) / 60;
                    let core_save = core.clone();
                    let item_id_save = item_id_for_save.clone();

                    view! {
                        <div class="rounded-lg border border-slate-200 bg-white p-4 space-y-3">
                            <div class="text-center">
                                <p class="text-sm text-slate-600">
                                    {format!("Practiced for {} min {} sec", secs / 60, secs % 60)}
                                </p>
                                {if minutes == 0 {
                                    view! {
                                        <p class="text-sm text-red-600 mt-1">
                                            "Session too short (less than 30 seconds). Cannot save."
                                        </p>
                                    }.into_any()
                                } else {
                                    view! {
                                        <p class="text-sm font-medium text-slate-800">
                                            {format!("Will log as {} minute{}", minutes, if minutes == 1 { "" } else { "s" })}
                                        </p>
                                    }.into_any()
                                }}
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-slate-700 mb-1" for="timer-notes">"Notes (optional)"</label>
                                <textarea
                                    id="timer-notes"
                                    rows="2"
                                    class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                                    bind:value=notes
                                />
                            </div>
                            {move || error_msg.get().map(|msg| {
                                view! {
                                    <p class="text-sm text-red-600">{msg}</p>
                                }
                            })}
                            <div class="flex gap-2">
                                {if minutes > 0 {
                                    let core_btn = core_save.clone();
                                    let item_id_btn = item_id_save.clone();
                                    Some(view! {
                                        <Button variant=ButtonVariant::Primary on_click=Callback::new(move |_| {
                                            let notes_val = notes.get();
                                            let session_notes = if notes_val.is_empty() { None } else { Some(notes_val) };
                                            let event = Event::Session(SessionEvent::Log(LogSession {
                                                item_id: item_id_btn.clone(),
                                                duration_minutes: minutes,
                                                notes: session_notes,
                                            }));
                                            let core_ref = core_btn.borrow();
                                            let effects = core_ref.process_event(event);
                                            process_effects(&core_ref, effects, &view_model);

                                            let vm = view_model.get_untracked();
                                            if let Some(err) = vm.error {
                                                error_msg.set(Some(err));
                                            } else {
                                                show_save_form.set(false);
                                                elapsed_secs.set(0);
                                                notes.set(String::new());
                                                error_msg.set(None);
                                            }
                                        })>
                                            "Save Session"
                                        </Button>
                                    })
                                } else {
                                    None
                                }}
                                <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| {
                                    show_save_form.set(false);
                                    elapsed_secs.set(0);
                                    notes.set(String::new());
                                    error_msg.set(None);
                                })>
                                    "Discard"
                                </Button>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="rounded-lg border border-slate-200 bg-white p-4">
                            <div class="flex items-center justify-between">
                                <div>
                                    <p class="text-2xl font-mono font-bold text-slate-900">
                                        {move || {
                                            let secs = elapsed_secs.get();
                                            format!("{:02}:{:02}", secs / 60, secs % 60)
                                        }}
                                    </p>
                                </div>
                                <div class="flex gap-2">
                                    {move || {
                                        if running.get() {
                                            view! {
                                                <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| {
                                                    running.set(false);
                                                    if let Some(id) = interval_id.get_untracked() {
                                                        if let Some(window) = web_sys::window() {
                                                            window.clear_interval_with_handle(id);
                                                        }
                                                        interval_id.set(None);
                                                    }
                                                    if elapsed_secs.get_untracked() > 0 {
                                                        show_save_form.set(true);
                                                    }
                                                })>
                                                    "Stop"
                                                </Button>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <Button variant=ButtonVariant::Primary on_click=Callback::new(move |_| {
                                                    elapsed_secs.set(0);
                                                    running.set(true);

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
                                                })>
                                                    "Start Practice"
                                                </Button>
                                            }.into_any()
                                        }
                                    }}
                                </div>
                            </div>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}
