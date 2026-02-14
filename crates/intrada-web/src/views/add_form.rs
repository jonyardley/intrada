use std::collections::HashMap;

use leptos::ev;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::domain::exercise::ExerciseEvent;
use intrada_core::domain::piece::PieceEvent;
use intrada_core::domain::types::{CreateExercise, CreatePiece};
use intrada_core::{Event, ViewModel};

use crate::components::{
    BackLink, Button, ButtonVariant, Card, PageHeading, TextArea, TextField, TypeTabs,
};
use crate::core_bridge::process_effects;
use crate::helpers::{parse_tags, parse_tempo};
use crate::types::{ItemType, SharedCore};
use crate::validation::validate_library_form;

#[component]
pub fn AddLibraryItemForm(view_model: RwSignal<ViewModel>, core: SharedCore) -> impl IntoView {
    let navigate = use_navigate();
    let navigate_cancel = navigate.clone();

    // Tab state — Piece is the default (FR-002)
    let active_tab = RwSignal::new(ItemType::Piece);

    // Shared field signals — persist across tab switches (FR-004)
    let title = RwSignal::new(String::new());
    let composer = RwSignal::new(String::new());
    let key_sig = RwSignal::new(String::new());
    let tempo_marking = RwSignal::new(String::new());
    let bpm = RwSignal::new(String::new());
    let notes = RwSignal::new(String::new());
    let tags_input = RwSignal::new(String::new());

    // Type-specific field — value preserved in memory when hidden (FR-005)
    let category = RwSignal::new(String::new());

    // Validation errors — cleared on tab switch (FR-007)
    let errors: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());

    view! {
        <div>
            <BackLink label="Cancel" href="/".to_string() />

            <PageHeading text="Add Library Item" />

            <Card>
                <form
                    class="space-y-5"
                    on:submit=move |ev: ev::SubmitEvent| {
                        ev.prevent_default();

                        let current_tab = active_tab.get();

                        // Validate using unified function (FR-006)
                        let validation_errors = validate_library_form(
                            current_tab,
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

                        // Build values
                        let title_val = title.get().trim().to_string();
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

                        // FR-008: Create correct item type based on active tab
                        let event = match current_tab {
                            ItemType::Piece => {
                                let composer_val = composer.get().trim().to_string();
                                Event::Piece(PieceEvent::Add(CreatePiece {
                                    title: title_val,
                                    composer: composer_val,
                                    key: key_val,
                                    tempo: tempo_val,
                                    notes: notes_val,
                                    tags: tags_val,
                                }))
                            }
                            ItemType::Exercise => {
                                let composer_val = {
                                    let c = composer.get().trim().to_string();
                                    if c.is_empty() { None } else { Some(c) }
                                };
                                let category_val = {
                                    let c = category.get().trim().to_string();
                                    if c.is_empty() { None } else { Some(c) }
                                };
                                Event::Exercise(ExerciseEvent::Add(CreateExercise {
                                    title: title_val,
                                    composer: composer_val,
                                    category: category_val,
                                    key: key_val,
                                    tempo: tempo_val,
                                    notes: notes_val,
                                    tags: tags_val,
                                }))
                            }
                        };

                        let core_ref = core.borrow();
                        let effects = core_ref.process_event(event);
                        process_effects(&core_ref, effects, &view_model);
                        navigate("/", NavigateOptions { replace: true, ..Default::default() });
                    }
                >
                    // Tab bar — interactive mode (FR-001)
                    <TypeTabs
                        active=Signal::derive(move || active_tab.get())
                        on_change=Callback::new(move |tab: ItemType| {
                            active_tab.set(tab);
                            errors.set(HashMap::new()); // FR-007: clear errors on tab switch
                        })
                    />

                    // Tab panel content
                    <div id="tabpanel-content" role="tabpanel">
                        // Title (required — shared)
                        <TextField id="add-title" label="Title *" value=title required=true field_name="title" errors=errors />

                        // Composer field — two TextFields sharing the same signal (research.md: Option B)
                        // Piece: required; Exercise: optional
                        {move || {
                            if active_tab.get() == ItemType::Piece {
                                view! {
                                    <TextField id="add-composer" label="Composer *" value=composer required=true field_name="composer" errors=errors />
                                }.into_any()
                            } else {
                                view! {
                                    <TextField id="add-composer" label="Composer" value=composer field_name="composer" errors=errors />
                                }.into_any()
                            }
                        }}

                        // Category — Exercise only, conditionally rendered (FR-005)
                        {move || {
                            if active_tab.get() == ItemType::Exercise {
                                Some(view! {
                                    <TextField id="add-category" label="Category" value=category placeholder="e.g. Technique, Scales" field_name="category" errors=errors />
                                })
                            } else {
                                None
                            }
                        }}

                        // Key (optional — shared)
                        <TextField id="add-key" label="Key" value=key_sig placeholder="e.g. C Major, Db Minor" field_name="key" errors=errors />

                        // Tempo: marking + BPM on one row (shared)
                        <div class="grid grid-cols-2 gap-4">
                            <TextField id="add-tempo-marking" label="Tempo Marking" value=tempo_marking placeholder="e.g. Allegro" field_name="tempo_marking" errors=errors />
                            <TextField id="add-bpm" label="BPM" value=bpm input_type="number" placeholder="1-400" field_name="bpm" errors=errors />
                        </div>

                        // Notes (optional — shared)
                        <TextArea id="add-notes" label="Notes" value=notes field_name="notes" errors=errors />

                        // Tags (shared)
                        <TextField id="add-tags" label="Tags" value=tags_input placeholder="Comma-separated, e.g. classical, piano" field_name="tags" errors=errors />

                        // Buttons
                        <div class="flex gap-3 pt-2">
                            <Button variant=ButtonVariant::Primary button_type="submit">"Save"</Button>
                            <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| {
                                navigate_cancel("/", NavigateOptions::default());
                            })>"Cancel"</Button>
                        </div>
                    </div>
                </form>
            </Card>
        </div>
    }
}
