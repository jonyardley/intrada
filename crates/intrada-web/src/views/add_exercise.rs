use std::collections::HashMap;

use leptos::ev;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::domain::exercise::ExerciseEvent;
use intrada_core::domain::types::CreateExercise;
use intrada_core::{Event, ViewModel};

use crate::components::{BackLink, Button, ButtonVariant, Card, PageHeading, TextArea, TextField};
use crate::core_bridge::process_effects;
use crate::helpers::{parse_tags, parse_tempo};
use crate::types::SharedCore;
use crate::validation::validate_exercise_form;

#[component]
pub fn AddExerciseForm(view_model: RwSignal<ViewModel>, core: SharedCore) -> impl IntoView {
    let navigate = use_navigate();

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
            <BackLink label="Cancel" href="/".to_string() />

            <PageHeading text="Add Exercise" />

            <Card>
                <form
                class="space-y-5"
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
                    navigate("/", NavigateOptions { replace: true, ..Default::default() });
                }
            >
                // Title (required)
                <TextField id="exercise-title" label="Title *" value=title required=true field_name="title" errors=errors />

                // Composer (optional for exercises)
                <TextField id="exercise-composer" label="Composer" value=composer field_name="composer" errors=errors />

                // Category (optional, exercises only)
                <TextField id="exercise-category" label="Category" value=category placeholder="e.g. Technique, Scales" field_name="category" errors=errors />

                // Key
                <TextField id="exercise-key" label="Key" value=key_sig placeholder="e.g. C Major" field_name="key" errors=errors />

                // Tempo row
                <div class="grid grid-cols-2 gap-4">
                    <TextField id="exercise-tempo-marking" label="Tempo Marking" value=tempo_marking placeholder="e.g. Moderato" field_name="tempo_marking" errors=errors />
                    <TextField id="exercise-bpm" label="BPM" value=bpm input_type="number" placeholder="1-400" field_name="bpm" errors=errors />
                </div>

                // Notes
                <TextArea id="exercise-notes" label="Notes" value=notes field_name="notes" errors=errors />

                // Tags
                <TextField id="exercise-tags" label="Tags" value=tags_input placeholder="Comma-separated, e.g. technique, warm-up" field_name="tags" errors=errors />

                // Buttons
                <div class="flex gap-3 pt-2">
                    <Button variant=ButtonVariant::Primary button_type="submit">"Save"</Button>
                    <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| {
                        let navigate = use_navigate();
                        navigate("/", NavigateOptions::default());
                    })>"Cancel"</Button>
                </div>
            </form>
            </Card>
        </div>
    }
}
