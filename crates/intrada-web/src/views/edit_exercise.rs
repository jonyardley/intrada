use std::collections::HashMap;

use leptos::ev;
use leptos::prelude::*;

use intrada_core::domain::exercise::ExerciseEvent;
use intrada_core::domain::types::UpdateExercise;
use intrada_core::{Event, ViewModel};

use crate::components::FormFieldError;
use crate::core_bridge::process_effects;
use crate::helpers::{parse_tags, parse_tempo, parse_tempo_display};
use crate::types::{SharedCore, ViewState};
use crate::validation::validate_exercise_form;

#[component]
pub fn EditExerciseForm(
    id: String,
    view_model: RwSignal<ViewModel>,
    view_state: RwSignal<ViewState>,
    core: SharedCore,
) -> impl IntoView {
    let item = view_model
        .get_untracked()
        .items
        .into_iter()
        .find(|i| i.id == id);

    let Some(item) = item else {
        view_state.set(ViewState::List);
        return view! { <p>"Item not found."</p> }.into_any();
    };

    let item_id = item.id.clone();

    // For exercises: subtitle = category.or(composer), so we read category and need
    // to figure out composer. We use the category field directly from ViewModel.
    // The subtitle may be category OR composer. We check: if category is Some, subtitle = category;
    // otherwise subtitle = composer. But we don't have a separate composer field in ViewModel.
    // Strategy: Use subtitle as composer IF category is None. If category is Some, we don't
    // know the composer from ViewModel alone. For editing, we'll use subtitle as best-effort.
    // The Crux core's view() builds subtitle as: category.or(composer).unwrap_or_default()
    // So if category is set, subtitle = category; composer is hidden.
    // We pre-populate category from item.category, and leave composer empty if category was used
    // as subtitle. This is the documented limitation (U2 note).
    let composer_initial = if item.category.is_some() {
        // Subtitle is category, not composer — we can't recover composer from ViewModel
        String::new()
    } else {
        // No category, subtitle is composer (or empty)
        item.subtitle.clone()
    };

    let title = RwSignal::new(item.title.clone());
    let composer = RwSignal::new(composer_initial);
    let category = RwSignal::new(item.category.clone().unwrap_or_default());
    let key_sig = RwSignal::new(item.key.clone().unwrap_or_default());
    let (initial_marking, initial_bpm) = parse_tempo_display(&item.tempo);
    let tempo_marking = RwSignal::new(initial_marking);
    let bpm = RwSignal::new(initial_bpm);
    let notes = RwSignal::new(item.notes.clone().unwrap_or_default());
    let tags_input = RwSignal::new(item.tags.join(", "));
    let errors: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());

    view! {
        <div>
            <button
                class="mb-6 inline-flex items-center gap-1 text-sm text-slate-500 hover:text-slate-700 transition-colors"
                on:click={
                    let id_back = item_id.clone();
                    move |_| { view_state.set(ViewState::Detail(id_back.clone())); }
                }
            >
                "\u{2190} Cancel"
            </button>

            <h2 class="text-2xl font-bold text-slate-900 mb-6">"Edit Exercise"</h2>

            <form
                class="bg-white rounded-xl shadow-sm border border-slate-200 p-6 space-y-5"
                on:submit={
                    let item_id = item_id.clone();
                    move |ev: ev::SubmitEvent| {
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
                            if c.is_empty() { Some(None) } else { Some(Some(c)) }
                        };
                        let category_val = {
                            let c = category.get().trim().to_string();
                            if c.is_empty() { Some(None) } else { Some(Some(c)) }
                        };
                        let key_val = {
                            let k = key_sig.get().trim().to_string();
                            if k.is_empty() { Some(None) } else { Some(Some(k)) }
                        };
                        let tempo_val = {
                            let t = parse_tempo(&tempo_marking.get(), &bpm.get());
                            match t {
                                None => Some(None),
                                Some(v) => Some(Some(v)),
                            }
                        };
                        let notes_val = {
                            let n = notes.get().trim().to_string();
                            if n.is_empty() { Some(None) } else { Some(Some(n)) }
                        };
                        let tags_val = parse_tags(&tags_input.get());

                        let input = UpdateExercise {
                            title: Some(title_val),
                            composer: composer_val,
                            category: category_val,
                            key: key_val,
                            tempo: tempo_val,
                            notes: notes_val,
                            tags: Some(tags_val),
                        };

                        let event = Event::Exercise(ExerciseEvent::Update {
                            id: item_id.clone(),
                            input,
                        });

                        let core_ref = core.borrow();
                        let effects = core_ref.process_event(event);
                        process_effects(&core_ref, effects, &view_model);
                        view_state.set(ViewState::Detail(item_id.clone()));
                    }
                }
            >
                // Title
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-exercise-title">"Title *"</label>
                    <input
                        id="edit-exercise-title"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        prop:value=move || title.get()
                        on:input=move |ev| { title.set(event_target_value(&ev)); }
                        required
                    />
                    <FormFieldError field="title".to_string() errors=errors />
                </div>

                // Composer (optional)
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-exercise-composer">"Composer"</label>
                    <input
                        id="edit-exercise-composer"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        prop:value=move || composer.get()
                        on:input=move |ev| { composer.set(event_target_value(&ev)); }
                    />
                    <FormFieldError field="composer".to_string() errors=errors />
                </div>

                // Category
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-exercise-category">"Category"</label>
                    <input
                        id="edit-exercise-category"
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
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-exercise-key">"Key"</label>
                    <input
                        id="edit-exercise-key"
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
                        <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-exercise-tempo-marking">"Tempo Marking"</label>
                        <input
                            id="edit-exercise-tempo-marking"
                            type="text"
                            class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                            placeholder="e.g. Moderato"
                            prop:value=move || tempo_marking.get()
                            on:input=move |ev| { tempo_marking.set(event_target_value(&ev)); }
                        />
                        <FormFieldError field="tempo_marking".to_string() errors=errors />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-exercise-bpm">"BPM"</label>
                        <input
                            id="edit-exercise-bpm"
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
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-exercise-notes">"Notes"</label>
                    <textarea
                        id="edit-exercise-notes"
                        rows="3"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        prop:value=move || notes.get()
                        on:input=move |ev| { notes.set(event_target_value(&ev)); }
                    />
                    <FormFieldError field="notes".to_string() errors=errors />
                </div>

                // Tags
                <div>
                    <label class="block text-sm font-medium text-slate-700 mb-1" for="edit-exercise-tags">"Tags"</label>
                    <input
                        id="edit-exercise-tags"
                        type="text"
                        class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                        placeholder="Comma-separated"
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
                        on:click={
                            let id_cancel = item_id.clone();
                            move |_| { view_state.set(ViewState::Detail(id_cancel.clone())); }
                        }
                    >
                        "Cancel"
                    </button>
                </div>
            </form>
        </div>
    }.into_any()
}
