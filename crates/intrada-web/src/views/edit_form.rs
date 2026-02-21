use std::collections::HashMap;

use leptos::ev;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};
use leptos_router::NavigateOptions;

use intrada_core::{Event, ItemEvent, UpdateItem, ViewModel};

use crate::components::{
    AutocompleteTextField, BackLink, Button, ButtonVariant, Card, PageHeading, TagInput, TextArea,
    TextField, TypeTabs,
};
use intrada_web::core_bridge::process_effects;
use intrada_web::helpers::{parse_tempo, parse_tempo_display, unique_composers, unique_tags};
use intrada_web::types::{IsLoading, IsSubmitting, ItemType, SharedCore};
use intrada_web::validation::{validate_library_form, FormData};

#[component]
pub fn EditLibraryItemForm() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let params = use_params_map();
    let id = params.read().get("id").unwrap_or_default();
    let navigate = use_navigate();

    // Find item to pre-populate
    let item = view_model
        .get_untracked()
        .items
        .into_iter()
        .find(|i| i.id == id);

    let Some(item) = item else {
        return view! {
            <div class="text-center py-8">
                <p class="text-secondary mb-4">"Item not found."</p>
                <A href="/" attr:class="text-accent-text hover:text-accent-hover font-medium">
                    "← Back to Library"
                </A>
            </div>
        }
        .into_any();
    };

    let item_id = item.id.clone();
    let back_href = format!("/library/{}", item_id);

    // Determine item type — plain value, not a signal (display-only tabs)
    let item_type = if item.item_type == "piece" {
        ItemType::Piece
    } else {
        ItemType::Exercise
    };

    // Pre-populate signals from ViewModel
    let title = RwSignal::new(item.title.clone());
    let key_sig = RwSignal::new(item.key.clone().unwrap_or_default());
    let (initial_marking, initial_bpm) = parse_tempo_display(&item.tempo);
    let tempo_marking = RwSignal::new(initial_marking);
    let bpm = RwSignal::new(initial_bpm);
    let notes = RwSignal::new(item.notes.clone().unwrap_or_default());
    let tags = RwSignal::new(item.tags.clone());

    // Pre-populate composer based on item type
    // For Piece: subtitle is always the composer
    // For Exercise: subtitle = category.or(composer). If category is set, we can't recover
    // the composer from ViewModel alone (documented limitation).
    let composer_initial = match item_type {
        ItemType::Piece => item.subtitle.clone(),
        ItemType::Exercise => {
            if item.category.is_some() {
                // Subtitle is category, not composer — can't recover composer
                String::new()
            } else {
                // No category, subtitle is composer (or empty)
                item.subtitle.clone()
            }
        }
    };
    let composer = RwSignal::new(composer_initial);
    let category = RwSignal::new(item.category.clone().unwrap_or_default());

    let errors: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());

    // Derive autocomplete suggestions from library data
    let all_tags_signal = Signal::derive(move || unique_tags(&view_model.get().items));
    let all_composers_signal = Signal::derive(move || unique_composers(&view_model.get().items));

    let cancel_href = back_href.clone();

    view! {
        <div class="sm:max-w-2xl sm:mx-auto">
            <BackLink label="Cancel" href=back_href />

            <PageHeading text="Edit Library Item" />

            <Card>
                <form
                    class="space-y-4"
                    on:submit={
                        let item_id = item_id.clone();
                        move |ev: ev::SubmitEvent| {
                            ev.prevent_default();

                            // Build tags string for validation (validation expects comma-separated)
                            let tags_str = tags.get().join(", ");

                            // Validate using unified function
                            let validation_errors = validate_library_form(
                                item_type,
                                &FormData {
                                    title: &title.get(),
                                    composer: &composer.get(),
                                    category: &category.get(),
                                    notes: &notes.get(),
                                    bpm_str: &bpm.get(),
                                    tempo_marking: &tempo_marking.get(),
                                    tags_str: &tags_str,
                                },
                            );

                            if !validation_errors.is_empty() {
                                errors.set(validation_errors);
                                return;
                            }
                            errors.set(HashMap::new());

                            // Build common values
                            let title_val = title.get().trim().to_string();
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
                            let tags_val = tags.get();

                            // Build unified update event
                            let composer_val = {
                                let c = composer.get().trim().to_string();
                                match item_type {
                                    ItemType::Piece => Some(Some(c)),
                                    ItemType::Exercise => {
                                        if c.is_empty() { Some(None) } else { Some(Some(c)) }
                                    }
                                }
                            };
                            let category_val = {
                                let c = category.get().trim().to_string();
                                if c.is_empty() { Some(None) } else { Some(Some(c)) }
                            };

                            let input = UpdateItem {
                                title: Some(title_val),
                                composer: composer_val,
                                category: category_val,
                                key: key_val,
                                tempo: tempo_val,
                                notes: notes_val,
                                tags: Some(tags_val),
                            };
                            let event = Event::Item(ItemEvent::Update {
                                id: item_id.clone(),
                                input,
                            });

                            let core_ref = core.borrow();
                            let effects = core_ref.process_event(event);
                            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                            let detail_url = format!("/library/{}", item_id);
                            navigate(&detail_url, NavigateOptions { replace: true, ..Default::default() });
                        }
                    }
                >
                    // Tab bar — display-only mode (FR-015): on_change is None
                    <TypeTabs
                        active=Signal::derive(move || item_type)
                    />

                    // Tab panel content
                    <div id="tabpanel-content" role="tabpanel" class="space-y-4">
                        // Title (required — shared)
                        <TextField id="edit-title" label="Title *" value=title required=true field_name="title" errors=errors />

                        // Composer field with autocomplete — static conditional (item type is fixed for edits)
                        // Piece: required; Exercise: optional
                        {if item_type == ItemType::Piece {
                            view! {
                                <AutocompleteTextField id="edit-composer" label="Composer *" value=composer suggestions=all_composers_signal required=true field_name="composer" errors=errors />
                            }.into_any()
                        } else {
                            view! {
                                <AutocompleteTextField id="edit-composer" label="Composer (optional)" value=composer suggestions=all_composers_signal field_name="composer" errors=errors />
                            }.into_any()
                        }}

                        // Category — Exercise only
                        {if item_type == ItemType::Exercise {
                            Some(view! {
                                <TextField id="edit-category" label="Category" value=category placeholder="e.g. Technique, Scales" field_name="category" errors=errors />
                            })
                        } else {
                            None
                        }}

                        // Key (optional — shared)
                        <TextField id="edit-key" label="Key" value=key_sig hint="e.g. C Major, Db Minor" field_name="key" errors=errors />

                        // Tempo: marking + BPM on one row (shared)
                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                            <TextField id="edit-tempo-marking" label="Tempo Marking" value=tempo_marking placeholder="e.g. Allegro" field_name="tempo_marking" errors=errors />
                            <TextField id="edit-bpm" label="BPM" value=bpm input_type="number" placeholder="1-400" field_name="bpm" errors=errors />
                        </div>

                        // Notes (optional — shared)
                        <TextArea id="edit-notes" label="Notes" value=notes hint="Practice notes, goals, or reminders" field_name="notes" errors=errors />

                        // Tags — chip-based input with autocomplete
                        <TagInput id="edit-tags" tags=tags available_tags=all_tags_signal field_name="tags" errors=errors />

                        // Buttons
                        <div class="flex flex-col sm:flex-row gap-3 pt-2">
                            <Button
                                variant=ButtonVariant::Primary
                                button_type="submit"
                                loading=Signal::derive(move || is_submitting.get())
                            >
                                {move || if is_submitting.get() { "Saving\u{2026}" } else { "Save" }}
                            </Button>
                            <Button variant=ButtonVariant::Secondary on_click={
                                let cancel_href = cancel_href.clone();
                                let navigate = navigate.clone();
                                Callback::new(move |_| {
                                    navigate(&cancel_href, NavigateOptions::default());
                                })
                            }>"Cancel"</Button>
                        </div>
                    </div>
                </form>
            </Card>
        </div>
    }.into_any()
}
