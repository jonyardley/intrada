use std::collections::HashMap;

use leptos::ev;
use leptos::prelude::*;

use intrada_core::domain::exercise::ExerciseEvent;
use intrada_core::domain::types::CreateExercise;
use intrada_core::{Event, ViewModel};

use crate::components::FormFieldError;
use crate::core_bridge::process_effects;
use crate::helpers::{parse_tags, parse_tempo};
use crate::types::{SharedCore, ViewState};
use crate::validation::validate_exercise_form;

#[component]
pub fn AddExerciseForm(
    view_model: RwSignal<ViewModel>,
    view_state: RwSignal<ViewState>,
    core: SharedCore,
) -> impl IntoView {
    let title = RwSignal::new(String::new());
    let composer = RwSignal::new(String::new());
    let category = RwSignal::new(String::new());
    let key_sig = RwSignal::new(String::new());
    let tempo_marking = RwSignal::new(String::new());
    let bpm = RwSignal::new(String::new());
    let notes = RwSignal::new(String::new());
    let tags_input = RwSignal::new(String::new());
    let errors: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());

    view! {
        <div>
            <button
                class="mb-6 inline-flex items-center gap-1 text-sm text-slate-500 hover:text-slate-700 transition-colors"
                on:click=move |_| { view_state.set(ViewState::List); }
            >
                "\u{2190} Cancel"
            </button>

            <h2 class="text-2xl font-bold text-slate-900 mb-6">"Add Exercise"</h2>

            <form
                class="bg-white rounded-xl shadow-sm border border-slate-200 p-6 space-y-5"
                on:submit=move |ev: ev::SubmitEvent| {
                    ev.prevent_default();

                    let validation_errors = validate_exercise_form(
                        &title.get(),
                        &composer.get(),
                        &category.get(),
                        &notes.get(),
                        &bpm.get(),
                        &tempo_marking.get(),
                        &tags_input.get(),
                    );

                    if !validation_errors.is_empty() {
                        errors.set(validation_errors);
                        return;
                    }
                    errors.set(HashMap::new());

                    let title_val = title.get().trim().to_string();
                    let composer_val = {
                        let c = composer.get().trim().to_string();
                        if c.is_empty() { None } else { Some(c) }
                    };
                    let category_val = {
                        let c = category.get().trim().to_string();
                        if c.is_empty() { None } else { Some(c) }
                    };
                    let key_val = {
                        let k = key_sig.get().trim().to_string();
                        if k.is_empty() { None } else { Some(k) }
                    };
                    let tempo_val = parse_tempo(&tempo_marking.get(), &bpm.get());
                    let notes_val = {
                        let n = notes.get().trim().to_string();
                        if n.is_empty() { None } else { Some(n) }
                    };
                    let tags_val = parse_tags(&tags_input.get());

                    let event = Event::Exercise(ExerciseEvent::Add(CreateExercise {
                        title: title_val,
                        composer: composer_val,
                        category: category_val,
                        key: key_val,
                        tempo: tempo_val,
                        notes: notes_val,
                        tags: tags_val,
                    }));

                    let core_ref = core.borrow();
                    let effects = core_ref.process_event(event);
                    process_effects(&core_ref, effects, &view_model);
                    view_state.set(ViewState::List);
                }
            >
                // Title (required)
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="exercise-title">"Title *"</label>
                    <input
                        id="exercise-title"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        prop:value=move || title.get()
                        on:input=move |ev| { title.set(event_target_value(&ev)); }
                        required
                    />
                    <FormFieldError field="title".to_string() errors=errors />
                </div>

                // Composer (optional for exercises)
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="exercise-composer">"Composer"</label>
                    <input
                        id="exercise-composer"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        prop:value=move || composer.get()
                        on:input=move |ev| { composer.set(event_target_value(&ev)); }
                    />
                    <FormFieldError field="composer".to_string() errors=errors />
                </div>

                // Category (optional, exercises only)
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="exercise-category">"Category"</label>
                    <input
                        id="exercise-category"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        placeholder="e.g. Technique, Scales"
                        prop:value=move || category.get()
                        on:input=move |ev| { category.set(event_target_value(&ev)); }
                    />
                    <FormFieldError field="category".to_string() errors=errors />
                </div>

                // Key
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="exercise-key">"Key"</label>
                    <input
                        id="exercise-key"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        placeholder="e.g. C Major"
                        prop:value=move || key_sig.get()
                        on:input=move |ev| { key_sig.set(event_target_value(&ev)); }
                    />
                </div>

                // Tempo row
                <div class="grid grid-cols-2 gap-4">
                    <div>
                        <label class="block text-sm font-medium text-slate-700 mb-1" for="exercise-tempo-marking">"Tempo Marking"</label>
                        <input
                            id="exercise-tempo-marking"
                            type="text"
                            class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                            placeholder="e.g. Moderato"
                            prop:value=move || tempo_marking.get()
                            on:input=move |ev| { tempo_marking.set(event_target_value(&ev)); }
                        />
                        <FormFieldError field="tempo_marking".to_string() errors=errors />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-slate-700 mb-1" for="exercise-bpm">"BPM"</label>
                        <input
                            id="exercise-bpm"
                            type="number"
                            class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                            placeholder="1-400"
                            prop:value=move || bpm.get()
                            on:input=move |ev| { bpm.set(event_target_value(&ev)); }
                        />
                        <FormFieldError field="bpm".to_string() errors=errors />
                    </div>
                </div>

                // Notes
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="exercise-notes">"Notes"</label>
                    <textarea
                        id="exercise-notes"
                        rows="3"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        prop:value=move || notes.get()
                        on:input=move |ev| { notes.set(event_target_value(&ev)); }
                    />
                    <FormFieldError field="notes".to_string() errors=errors />
                </div>

                // Tags
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="exercise-tags">"Tags"</label>
                    <input
                        id="exercise-tags"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        placeholder="Comma-separated, e.g. technique, warm-up"
                        prop:value=move || tags_input.get()
                        on:input=move |ev| { tags_input.set(event_target_value(&ev)); }
                    />
                    <FormFieldError field="tags".to_string() errors=errors />
                </div>

                // Buttons
                <div class="flex gap-3 pt-2">
                    <button
                        type="submit"
                        class="rounded-lg bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-500 transition-colors"
                    >
                        "Save"
                    </button>
                    <button
                        type="button"
                        class="rounded-lg bg-white px-4 py-2 text-sm font-medium text-slate-700 border border-slate-300 hover:bg-slate-50 transition-colors"
                        on:click=move |_| { view_state.set(ViewState::List); }
                    >
                        "Cancel"
                    </button>
                </div>
            </form>
        </div>
    }
}
