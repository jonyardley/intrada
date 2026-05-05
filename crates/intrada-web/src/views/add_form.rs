use std::collections::HashMap;

use leptos::ev;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::{CreateItem, Event, ItemEvent, ItemKind, ViewModel};

use crate::components::{
    AutocompleteTextField, BackLink, Button, ButtonSize, ButtonVariant, Card, PageHeading,
    TagInput, TextArea, TextField, TypeTabs,
};
use intrada_web::core_bridge::process_effects;
use intrada_web::helpers::{parse_tempo, unique_composers, unique_tags};
use intrada_web::types::{IsLoading, IsSubmitting, ItemType, SharedCore};
use intrada_web::validation::{validate_library_form, FormData};

#[component]
pub fn AddLibraryItemForm(
    /// When rendered inside a BottomSheet (vs as a standalone route), drop
    /// the back-link / page heading / card chrome — the sheet provides its
    /// own. Cancel + Save call `on_dismiss` instead of navigating.
    #[prop(optional)]
    in_sheet: bool,
    /// Fired when the user successfully saves or cancels. Required when
    /// `in_sheet` is true; ignored otherwise (route mode navigates instead).
    #[prop(optional, into)]
    on_dismiss: Option<Callback<()>>,
    /// Optional ref to the underlying `<form>` element. The sheet that owns
    /// this form passes one in so its nav-bar Save action can trigger
    /// `requestSubmit()` — the form's existing `on:submit` handler then
    /// runs validation + dispatch. The bottom in-form button still works
    /// the same; this just adds a second trigger.
    #[prop(optional, into)]
    form_ref: Option<NodeRef<leptos::html::Form>>,
) -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
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
    let tags = RwSignal::new(Vec::<String>::new());

    // Validation errors — cleared on tab switch (FR-007)
    let errors: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());

    // Derive autocomplete suggestions from library data
    let all_tags_signal = Signal::derive(move || unique_tags(&view_model.get().items));
    let all_composers_signal = Signal::derive(move || unique_composers(&view_model.get().items));

    let dismiss_save = on_dismiss;
    let dismiss_cancel = on_dismiss;

    // Allow either an externally-passed form ref (sheet mode, so the
    // sheet's nav Save can trigger requestSubmit) or our own internal one
    // for route mode.
    let form_ref = form_ref.unwrap_or_default();

    let form_body = view! {
        <form
            node_ref=form_ref
            class="space-y-4"
            on:submit=move |ev: ev::SubmitEvent| {
                ev.prevent_default();

                        let current_tab = active_tab.get();

                        // Build tags string for validation (validation expects comma-separated)
                        let tags_str = tags.get().join(", ");

                        // Validate using unified function (FR-006)
                        let validation_errors = validate_library_form(
                            current_tab,
                            &FormData {
                                title: &title.get(),
                                composer: &composer.get(),
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
                        let tags_val = tags.get();

                        // FR-008: Create correct item type based on active tab
                        let (kind, composer_val) = match current_tab {
                            ItemType::Piece => {
                                let c = composer.get().trim().to_string();
                                (ItemKind::Piece, Some(c))
                            }
                            ItemType::Exercise => {
                                let c = composer.get().trim().to_string();
                                let composer_opt = if c.is_empty() { None } else { Some(c) };
                                (ItemKind::Exercise, composer_opt)
                            }
                        };

                        let event = Event::Item(ItemEvent::Add(CreateItem {
                            title: title_val,
                            kind,
                            composer: composer_val,
                            key: key_val,
                            tempo: tempo_val,
                            notes: notes_val,
                            tags: tags_val,
                        }));

                        let core_ref = core.borrow();
                        let effects = core_ref.process_event(event);
                        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                        if let Some(cb) = dismiss_save {
                            cb.run(());
                        } else {
                            navigate("/", NavigateOptions { replace: true, ..Default::default() });
                        }
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
                    <div id="tabpanel-content" role="tabpanel" class="space-y-4">
                        // Title (required — shared)
                        <TextField id="add-title" label="Title *" value=title required=true field_name="title" errors=errors />

                        // Composer field with autocomplete suggestions
                        // Piece: required; Exercise: optional
                        {move || {
                            if active_tab.get() == ItemType::Piece {
                                view! {
                                    <AutocompleteTextField id="add-composer" label="Composer *" value=composer suggestions=all_composers_signal required=true field_name="composer" errors=errors />
                                }.into_any()
                            } else {
                                view! {
                                    <AutocompleteTextField id="add-composer" label="Composer (optional)" value=composer suggestions=all_composers_signal field_name="composer" errors=errors />
                                }.into_any()
                            }
                        }}

                        // Key (optional — shared)
                        <TextField id="add-key" label="Key" value=key_sig hint="e.g. C Major, Db Minor" field_name="key" errors=errors />

                        // Tempo: marking + BPM on one row (shared)
                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                            <TextField id="add-tempo-marking" label="Tempo Marking" value=tempo_marking placeholder="e.g. Allegro" field_name="tempo_marking" errors=errors />
                            <TextField id="add-bpm" label="BPM" value=bpm input_type="number" input_mode="numeric" placeholder="1-400" field_name="bpm" errors=errors />
                        </div>

                        // Notes (optional — shared)
                        <TextArea id="add-notes" label="Notes" value=notes hint="Practice notes, goals, or reminders" field_name="notes" errors=errors />

                        // Tags — chip-based input with autocomplete
                        <TagInput id="add-tags" tags=tags available_tags=all_tags_signal field_name="tags" errors=errors />

                        // Hero-size primary CTA — Pencil "Add piece" frame
                        // (MFuDt) shows this as the sticky bottom action.
                        // The sheet's nav-bar Save provides a second submit
                        // trigger that calls requestSubmit() on the form.
                        // No bottom Cancel — sheet's nav Cancel handles it.
                        <div class="flex flex-col pt-2">
                            <Button
                                variant=ButtonVariant::Primary
                                button_type="submit"
                                size=ButtonSize::Hero
                                full_width=true
                                loading=Signal::derive(move || is_submitting.get())
                            >
                                {move || if is_submitting.get() { "Saving\u{2026}" } else { "Add to Library" }}
                            </Button>
                            // Sheet mode: nav Cancel handles dismiss. Route
                            // mode: keep the explicit Cancel so the user has
                            // a way back without browser-back.
                            {(!in_sheet).then(|| view! {
                                <div class="mt-3">
                                    <Button variant=ButtonVariant::Secondary full_width=true on_click=Callback::new(move |_| {
                                        if let Some(cb) = dismiss_cancel {
                                            cb.run(());
                                        } else {
                                            navigate_cancel("/", NavigateOptions::default());
                                        }
                                    })>"Cancel"</Button>
                                </div>
                            })}
                        </div>
                    </div>
                </form>
    };

    if in_sheet {
        // Inside a BottomSheet: the sheet provides nav chrome; we just need
        // the form body.
        form_body.into_any()
    } else {
        view! {
            <div class="sm:max-w-2xl sm:mx-auto">
                <BackLink label="Cancel" href="/".to_string() />
                <PageHeading text="Add Library Item" />
                <Card>{form_body}</Card>
            </div>
        }
        .into_any()
    }
}
