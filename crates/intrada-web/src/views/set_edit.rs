use std::collections::HashSet;
use std::sync::Arc;

use leptos::ev;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};
use leptos_router::NavigateOptions;

use intrada_core::validation::MAX_SET_NAME;
use intrada_core::{Event, SetEntry, SetEntryView, SetEvent, ViewModel};

use crate::components::{
    BackLink, BuilderItemRow, Button, ButtonVariant, EditorEntry, EntryListEditor, PageHeading,
    SkeletonBlock, SkeletonLine,
};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Edit page for a single set — update name, reorder/remove entries, add from library.
#[component]
pub fn SetEditView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let params = use_params_map();
    let id = params.read().get("id").unwrap_or_default();
    let navigate = use_navigate();

    // Find set to edit
    let set = view_model
        .get_untracked()
        .sets
        .into_iter()
        .find(|r| r.id == id);

    // If set not found and still loading, show skeleton then re-check
    if set.is_none() {
        let id = id.clone();
        return view! {
            <div class="sm:max-w-2xl sm:mx-auto">
                <BackLink label="Back to Sets" href="/routines".to_string() />
                <PageHeading text="Edit Set" />
                {move || {
                    if is_loading.get() {
                        view! {
                            <div class="space-y-4 animate-pulse">
                                <SkeletonLine width="w-1/3" height="h-6" />
                                <SkeletonLine width="w-full" height="h-10" />
                                <SkeletonBlock height="h-32" />
                            </div>
                        }.into_any()
                    } else {
                        // Check if set appeared after loading completed
                        let found = view_model.get().sets.iter().any(|r| r.id == id);
                        if found {
                            let url = format!("/routines/{}/edit", id);
                            let navigate = use_navigate();
                            navigate(&url, NavigateOptions { replace: true, ..Default::default() });
                            ().into_any()
                        } else {
                            view! {
                                <div class="text-center py-8">
                                    <p class="text-secondary mb-4">"Set not found."</p>
                                    <A href="/routines" attr:class="text-accent-text hover:text-accent-hover font-medium">
                                        "\u{2190} Back to Sets"
                                    </A>
                                </div>
                            }.into_any()
                        }
                    }
                }}
            </div>
        }.into_any();
    }

    let set = set.expect("set confirmed Some above");
    let set_id = set.id.clone();
    let name = RwSignal::new(set.name.clone());
    let entries: RwSignal<Vec<SetEntryView>> = RwSignal::new(set.entries.clone());
    let form_error = RwSignal::new(Option::<String>::None);

    let core_save = core;

    // Drag-and-drop reorder fires on the local signal; <EntryListEditor>
    // owns the hook + container ref. Set edits stay shell-side until
    // the user hits Save (we don't dispatch a Crux event per drop).
    let on_reorder = Callback::new(move |(entry_id, new_position): (String, usize)| {
        entries.update(|e| {
            if let Some(src_idx) = e.iter().position(|x| x.id == entry_id) {
                let entry = e.remove(src_idx);
                // Clamp new_position to valid range
                let dst = new_position.min(e.len());
                e.insert(dst, entry);
            }
        });
    });

    let on_remove_entry = Callback::new(move |entry_id: String| {
        entries.update(|e| e.retain(|x| x.id != entry_id));
    });

    // Project `Vec<SetEntryView>` (5 fields) down to the minimal
    // `Vec<EditorEntry>` shape the shared editor consumes. Sets
    // don't carry planned durations, so `duration_display` is `None`.
    let editor_entries = Signal::derive(move || {
        entries
            .get()
            .into_iter()
            .map(|e| EditorEntry {
                id: e.id,
                item_title: e.item_title,
                item_type: e.item_type,
                duration_display: None,
            })
            .collect::<Vec<_>>()
    });

    // Toggle handler — adds the item if not present, removes if it is.
    // Mirrors the setlist builder's on_toggle semantics so the row
    // primitive (BuilderItemRow) can be reused.
    let on_toggle_item = Callback::new(move |item_id: String| {
        let vm = view_model.get_untracked();
        let already_in = entries.get_untracked().iter().any(|e| e.item_id == item_id);
        if already_in {
            entries.update(|e| e.retain(|x| x.item_id != item_id));
        } else if let Some(item) = vm.items.iter().find(|i| i.id == item_id) {
            let new_entry = SetEntryView {
                id: ulid::Ulid::new().to_string(),
                item_id: item.id.clone(),
                item_title: item.title.clone(),
                item_type: item.item_type.clone(),
                position: entries.get_untracked().len(),
            };
            entries.update(|e| e.push(new_entry));
        }
    });

    view! {
        <div class="sm:max-w-2xl sm:mx-auto">
            <BackLink label="Back to Sets" href="/routines".to_string() />

            <PageHeading text="Edit Set" />

            <div class="space-y-6">
                // Name + entries form — flat, no Card chrome (matches the
                // session builder / review-sheet vocabulary).
                <form class="space-y-4" on:submit={
                    let set_id = set_id.clone();
                    let navigate = navigate.clone();
                    move |ev: ev::SubmitEvent| {
                        ev.prevent_default();

                        let trimmed = name.get_untracked().trim().to_string();
                        if trimmed.is_empty() {
                            form_error.set(Some("Name is required".to_string()));
                            return;
                        }
                        if trimmed.len() > MAX_SET_NAME {
                            form_error.set(Some(format!("Name must be {MAX_SET_NAME} characters or fewer")));
                            return;
                        }
                        form_error.set(None);

                        let current_entries = entries.get_untracked();
                        if current_entries.is_empty() {
                            form_error.set(Some("Set must have at least one entry".to_string()));
                            return;
                        }

                        // Build SetEntry Vec from the view entries
                        let set_entries: Vec<SetEntry> = current_entries
                            .into_iter()
                            .enumerate()
                            .map(|(pos, e)| SetEntry {
                                id: e.id,
                                item_id: e.item_id,
                                item_title: e.item_title,
                                item_type: e.item_type,
                                position: pos,
                            })
                            .collect();

                        let event = Event::Set(SetEvent::UpdateSet {
                            id: set_id.clone(),
                            name: trimmed,
                            entries: set_entries,
                        });
                        let core_ref = core_save.borrow();
                        let effects = core_ref.process_event(event);
                        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                        navigate("/routines", NavigateOptions { replace: true, ..Default::default() });
                    }
                }>
                    <div>
                        <label for="set-name" class="form-label">"Set name"</label>
                        <input
                            id="set-name"
                            type="text"
                            class="input-base"
                            placeholder="e.g. Morning Warm-up"
                            bind:value=name
                        />
                    </div>
                    {move || form_error.get().map(|msg| view! {
                        <p class="text-xs text-danger-text">{msg}</p>
                    })}

                    // Entries — shared <EntryListEditor> shape (drag-reorder
                    // + compact rows + remove). Same primitive the session
                    // review sheet uses; the only divergence is the local
                    // signal vs Crux dispatch and that sets don't show
                    // per-entry duration.
                    <div>
                        <label class="form-label">"Entries"</label>
                        {move || {
                            if entries.get().is_empty() {
                                view! {
                                    <p class="text-sm text-muted text-center py-4">"No entries. Add items from your library below."</p>
                                }.into_any()
                            } else {
                                view! {
                                    <EntryListEditor
                                        entries=editor_entries
                                        on_reorder=on_reorder
                                        on_remove=on_remove_entry
                                    />
                                }.into_any()
                            }
                        }}
                    </div>

                    // Save / Cancel buttons
                    <div class="flex flex-col sm:flex-row gap-3 pt-2">
                        <Button
                            variant=ButtonVariant::Primary
                            button_type="submit"
                            loading=Signal::derive(move || is_submitting.get())
                        >
                            {move || if is_submitting.get() { "Saving\u{2026}" } else { "Save Changes" }}
                        </Button>
                        <Button variant=ButtonVariant::Secondary on_click={
                            let navigate = navigate.clone();
                            Callback::new(move |_| {
                                navigate("/routines", NavigateOptions::default());
                            })
                        }>"Cancel"</Button>
                    </div>
                </form>

                // Add from Library — flat, uses BuilderItemRow.
                <div>
                    <h3 class="section-title">"Add from Library"</h3>
                    {move || {
                        let vm = view_model.get();
                        if vm.items.is_empty() {
                            view! {
                                <p class="text-sm text-muted">"No library items available."</p>
                            }.into_any()
                        } else {
                            // Wrap in Arc so each per-row Signal::derive clones
                            // a cheap pointer rather than the whole HashSet.
                            // Arc rather than Rc because Signal::derive's closure
                            // bound requires Send + Sync.
                            let added_ids: Arc<HashSet<String>> = Arc::new(
                                entries.get().iter().map(|e| e.item_id.clone()).collect(),
                            );
                            view! {
                                <div class="space-y-2">
                                    {vm.items.iter().map(|item| {
                                        let item_id_clone = item.id.clone();
                                        let added_ids = added_ids.clone();
                                        let is_selected = Signal::derive(move || {
                                            added_ids.contains(&item_id_clone)
                                        });
                                        view! {
                                            <BuilderItemRow
                                                item=item.clone()
                                                is_selected=is_selected
                                                on_toggle=on_toggle_item
                                            />
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }.into_any()
}
