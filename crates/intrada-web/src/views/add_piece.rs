use std::collections::HashMap;

use leptos::ev;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::domain::piece::PieceEvent;
use intrada_core::domain::types::CreatePiece;
use intrada_core::{Event, ViewModel};

use crate::components::{BackLink, Button, ButtonVariant, Card, PageHeading, TextArea, TextField};
use crate::core_bridge::process_effects;
use crate::helpers::{parse_tags, parse_tempo};
use crate::types::SharedCore;
use crate::validation::validate_piece_form;

#[component]
pub fn AddPieceForm(view_model: RwSignal<ViewModel>, core: SharedCore) -> impl IntoView {
    let navigate = use_navigate();

    let title = RwSignal::new(String::new());
    let composer = RwSignal::new(String::new());
    let key_sig = RwSignal::new(String::new());
    let tempo_marking = RwSignal::new(String::new());
    let bpm = RwSignal::new(String::new());
    let notes = RwSignal::new(String::new());
    let tags_input = RwSignal::new(String::new());
    let errors: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());

    view! {
        <div>
            <BackLink label="Cancel" href="/".to_string() />

            <PageHeading text="Add Piece" />

            <Card>
                <form
                class="space-y-5"
                on:submit=move |ev: ev::SubmitEvent| {
                    ev.prevent_default();

                    let validation_errors = validate_piece_form(
                        &title.get(),
                        &composer.get(),
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
                    let composer_val = composer.get().trim().to_string();
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

                    let event = Event::Piece(PieceEvent::Add(CreatePiece {
                        title: title_val,
                        composer: composer_val,
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
                <TextField id="piece-title" label="Title *" value=title required=true field_name="title" errors=errors />

                // Composer (required for pieces)
                <TextField id="piece-composer" label="Composer *" value=composer required=true field_name="composer" errors=errors />

                // Key (optional)
                <TextField id="piece-key" label="Key" value=key_sig placeholder="e.g. C Major, Db Minor" field_name="key" errors=errors />

                // Tempo: marking + BPM on one row
                <div class="grid grid-cols-2 gap-4">
                    <TextField id="piece-tempo-marking" label="Tempo Marking" value=tempo_marking placeholder="e.g. Allegro" field_name="tempo_marking" errors=errors />
                    <TextField id="piece-bpm" label="BPM" value=bpm input_type="number" placeholder="1-400" field_name="bpm" errors=errors />
                </div>

                // Notes (optional)
                <TextArea id="piece-notes" label="Notes" value=notes field_name="notes" errors=errors />

                // Tags (comma-separated)
                <TextField id="piece-tags" label="Tags" value=tags_input placeholder="Comma-separated, e.g. classical, piano" field_name="tags" errors=errors />

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
