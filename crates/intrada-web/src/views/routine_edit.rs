use std::collections::HashSet;

use leptos::ev;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};
use leptos_router::NavigateOptions;

use intrada_core::validation::MAX_ROUTINE_NAME;
use intrada_core::{
    EntryStatus, Event, RoutineEntry, RoutineEntryView, RoutineEvent, SetlistEntryView, ViewModel,
};

use crate::components::{
    BackLink, BuilderItemRow, Button, ButtonVariant, PageHeading, SetlistEntryRow, SkeletonBlock,
    SkeletonLine,
};
use intrada_web::core_bridge::process_effects;
use intrada_web::hooks::use_drag_reorder;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Edit page for a single routine — update name, reorder/remove entries, add from library.
#[component]
pub fn RoutineEditView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let params = use_params_map();
    let id = params.read().get("id").unwrap_or_default();
    let navigate = use_navigate();

    // Find routine to edit
    let routine = view_model
        .get_untracked()
        .routines
        .into_iter()
        .find(|r| r.id == id);

    // If routine not found and still loading, show skeleton then re-check
    if routine.is_none() {
        let id = id.clone();
        return view! {
            <div class="sm:max-w-2xl sm:mx-auto">
                <BackLink label="Back to Routines" href="/routines".to_string() />
                <PageHeading text="Edit Routine" />
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
                        // Check if routine appeared after loading completed
                        let found = view_model.get().routines.iter().any(|r| r.id == id);
                        if found {
                            let url = format!("/routines/{}/edit", id);
                            let navigate = use_navigate();
                            navigate(&url, NavigateOptions { replace: true, ..Default::default() });
                            ().into_any()
                        } else {
                            view! {
                                <div class="text-center py-8">
                                    <p class="text-secondary mb-4">"Routine not found."</p>
                                    <A href="/routines" attr:class="text-accent-text hover:text-accent-hover font-medium">
                                        "\u{2190} Back to Routines"
                                    </A>
                                </div>
                            }.into_any()
                        }
                    }
                }}
            </div>
        }.into_any();
    }

    let routine = routine.expect("routine confirmed Some above");
    let routine_id = routine.id.clone();
    let name = RwSignal::new(routine.name.clone());
    let entries: RwSignal<Vec<RoutineEntryView>> = RwSignal::new(routine.entries.clone());
    let name_error = RwSignal::new(Option::<String>::None);

    let core_save = core;

    // --- Drag-and-drop setup for routine entries ---
    let entries_container_ref = NodeRef::<leptos::html::Div>::new();

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

    let drag = use_drag_reorder(on_reorder, entries_container_ref);
    let dragged_id = drag.dragged_id;
    let drag_source_index = drag.source_index;
    let drag_hover_index = drag.hover_index;
    let drag_live_offset_y = drag.live_offset_y;
    let drag_source_height = drag.source_height;
    let on_drag_pointer_down = drag.on_pointer_down;

    // Toggle handler — adds the item if not present, removes if it is.
    // Mirrors the setlist builder's on_toggle semantics so the row
    // primitive (BuilderItemRow) can be reused.
    let on_toggle_item = Callback::new(move |item_id: String| {
        let vm = view_model.get_untracked();
        let already_in = entries.get_untracked().iter().any(|e| e.item_id == item_id);
        if already_in {
            entries.update(|e| e.retain(|x| x.item_id != item_id));
        } else if let Some(item) = vm.items.iter().find(|i| i.id == item_id) {
            let new_entry = RoutineEntryView {
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
            <BackLink label="Back to Routines" href="/routines".to_string() />

            <PageHeading text="Edit Routine" />

            <div class="space-y-6">
                // Name + entries form — flat, no Card chrome (matches the
                // session builder / review-sheet vocabulary).
                <form class="space-y-4" on:submit={
                    let routine_id = routine_id.clone();
                    let navigate = navigate.clone();
                    move |ev: ev::SubmitEvent| {
                        ev.prevent_default();

                        let trimmed = name.get_untracked().trim().to_string();
                        if trimmed.is_empty() {
                            name_error.set(Some("Name is required".to_string()));
                            return;
                        }
                        if trimmed.len() > MAX_ROUTINE_NAME {
                            name_error.set(Some(format!("Name must be {MAX_ROUTINE_NAME} characters or fewer")));
                            return;
                        }
                        name_error.set(None);

                        let current_entries = entries.get_untracked();
                        if current_entries.is_empty() {
                            name_error.set(Some("Routine must have at least one entry".to_string()));
                            return;
                        }

                        // Build RoutineEntry Vec from the view entries
                        let routine_entries: Vec<RoutineEntry> = current_entries
                            .into_iter()
                            .enumerate()
                            .map(|(pos, e)| RoutineEntry {
                                id: e.id,
                                item_id: e.item_id,
                                item_title: e.item_title,
                                item_type: e.item_type,
                                position: pos,
                            })
                            .collect();

                        let event = Event::Routine(RoutineEvent::UpdateRoutine {
                            id: routine_id.clone(),
                            name: trimmed,
                            entries: routine_entries,
                        });
                        let core_ref = core_save.borrow();
                        let effects = core_ref.process_event(event);
                        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                        navigate("/routines", NavigateOptions { replace: true, ..Default::default() });
                    }
                }>
                    <div>
                        <label for="routine-name" class="form-label">"Routine name"</label>
                        <input
                            id="routine-name"
                            type="text"
                            class="input-base"
                            placeholder="e.g. Morning Warm-up"
                            bind:value=name
                        />
                    </div>
                    {move || name_error.get().map(|msg| view! {
                        <p class="text-xs text-danger-text">{msg}</p>
                    })}

                    // Entries — uses SetlistEntryRow compact mode and the
                    // same wrapper-transform reflow as the review sheet.
                    <div>
                        <label class="form-label">"Entries"</label>
                        {move || {
                            let current = entries.get();
                            if current.is_empty() {
                                view! {
                                    <p class="text-sm text-muted text-center py-4">"No entries. Add items from your library below."</p>
                                }.into_any()
                            } else {
                                view! {
                                    <div node_ref=entries_container_ref aria-roledescription="sortable" class="flex flex-col">
                                        {current.into_iter().enumerate().map(|(idx, entry)| {
                                            let entry_id_for_remove = entry.id.clone();
                                            let eid = entry.id.clone();
                                            let is_dragging_this = Signal::derive(move || {
                                                dragged_id.get().as_deref() == Some(eid.as_str())
                                            });

                                            let on_remove = Callback::new(move |_: String| {
                                                let id = entry_id_for_remove.clone();
                                                entries.update(|e| e.retain(|x| x.id != id));
                                            });

                                            // Routine entries have no per-entry duration today,
                                            // so populate the wider SetlistEntryView with defaults
                                            // for the unused fields. SetlistEntryRow only reads
                                            // item_title / item_type / duration_display in
                                            // compact mode, so the defaults don't surface.
                                            let setlist_entry = SetlistEntryView {
                                                id: entry.id.clone(),
                                                item_id: entry.item_id.clone(),
                                                item_title: entry.item_title.clone(),
                                                item_type: entry.item_type.clone(),
                                                position: idx,
                                                duration_display: String::new(),
                                                status: EntryStatus::NotAttempted,
                                                notes: None,
                                                score: None,
                                                intention: None,
                                                rep_target: None,
                                                rep_count: None,
                                                rep_target_reached: None,
                                                rep_history: None,
                                                planned_duration_secs: None,
                                                planned_duration_display: None,
                                                achieved_tempo: None,
                                            };

                                            // Wrapper transform: source row tracks the finger,
                                            // displaced rows slide by source_height. Same logic
                                            // as session_review_sheet.
                                            let row_style = move || {
                                                let Some(src) = drag_source_index.get() else {
                                                    return String::new();
                                                };
                                                if idx == src {
                                                    let off = drag_live_offset_y.get();
                                                    return format!(
                                                        "transform: translateY({off}px) scale(1.02); transition: none; position: relative; z-index: 10; box-shadow: 0 8px 20px rgba(0,0,0,0.35);"
                                                    );
                                                }
                                                let Some(hov) = drag_hover_index.get() else {
                                                    return String::new();
                                                };
                                                let h = drag_source_height.get();
                                                let displaced_down = hov > src && idx > src && idx <= hov;
                                                let displaced_up = hov < src && idx >= hov && idx < src;
                                                if displaced_down {
                                                    format!(
                                                        "transform: translateY(-{h}px); transition: transform 200ms cubic-bezier(0.4, 0, 0.2, 1);"
                                                    )
                                                } else if displaced_up {
                                                    format!(
                                                        "transform: translateY({h}px); transition: transform 200ms cubic-bezier(0.4, 0, 0.2, 1);"
                                                    )
                                                } else {
                                                    "transform: translateY(0); transition: transform 200ms cubic-bezier(0.4, 0, 0.2, 1);".to_string()
                                                }
                                            };

                                            view! {
                                                <div style=row_style data-entry-index=idx.to_string()>
                                                    <SetlistEntryRow
                                                        entry=setlist_entry
                                                        on_remove=Some(on_remove)
                                                        show_controls=true
                                                        is_dragging_this=is_dragging_this
                                                        on_drag_pointer_down=Some(on_drag_pointer_down)
                                                        index=idx
                                                        compact=true
                                                    />
                                                </div>
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
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
                            let added_ids: HashSet<String> = entries
                                .get()
                                .iter()
                                .map(|e| e.item_id.clone())
                                .collect();
                            view! {
                                <div class="space-y-2">
                                    {vm.items.iter().map(|item| {
                                        let item_id_clone = item.id.clone();
                                        let added_ids_for_signal = added_ids.clone();
                                        let is_selected = Signal::derive(move || {
                                            added_ids_for_signal.contains(&item_id_clone)
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
