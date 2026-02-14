use std::collections::HashMap;

use leptos::ev;
use leptos::prelude::*;

use intrada_core::domain::exercise::ExerciseEvent;
use intrada_core::domain::types::UpdateExercise;
use intrada_core::{Event, ViewModel};

use crate::components::{BackLink, Button, ButtonVariant, Card, PageHeading, TextArea, TextField};
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
            <BackLink label="Cancel" on_click={
                let id_back = item_id.clone();
                Callback::new(move |_| { view_state.set(ViewState::Detail(id_back.clone())); })
            } />

            <PageHeading text="Edit Exercise" />

            <Card>
                <form
                class="space-y-5"
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
                <TextField id="edit-exercise-title" label="Title *" value=title required=true field_name="title" errors=errors />

                // Composer (optional)
                <TextField id="edit-exercise-composer" label="Composer" value=composer field_name="composer" errors=errors />

                // Category
                <TextField id="edit-exercise-category" label="Category" value=category placeholder="e.g. Technique, Scales" field_name="category" errors=errors />

                // Key
                <TextField id="edit-exercise-key" label="Key" value=key_sig placeholder="e.g. C Major" field_name="key" errors=errors />

                // Tempo row
                <div class="grid grid-cols-2 gap-4">
                    <TextField id="edit-exercise-tempo-marking" label="Tempo Marking" value=tempo_marking placeholder="e.g. Moderato" field_name="tempo_marking" errors=errors />
                    <TextField id="edit-exercise-bpm" label="BPM" value=bpm input_type="number" placeholder="1-400" field_name="bpm" errors=errors />
                </div>

                // Notes
                <TextArea id="edit-exercise-notes" label="Notes" value=notes field_name="notes" errors=errors />

                // Tags
                <TextField id="edit-exercise-tags" label="Tags" value=tags_input placeholder="Comma-separated" field_name="tags" errors=errors />

                // Buttons
                <div class="flex gap-3 pt-2">
                    <Button variant=ButtonVariant::Primary button_type="submit">"Save"</Button>
                    <Button variant=ButtonVariant::Secondary on_click={
                        let id_cancel = item_id.clone();
                        Callback::new(move |_| { view_state.set(ViewState::Detail(id_cancel.clone())); })
                    }>"Cancel"</Button>
                </div>
            </form>
            </Card>
        </div>
    }.into_any()
}
