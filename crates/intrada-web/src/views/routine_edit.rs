use leptos::ev;
use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};
use leptos_router::NavigateOptions;

use intrada_core::validation::MAX_ROUTINE_NAME;
use intrada_core::{Event, RoutineEntry, RoutineEntryView, RoutineEvent, ViewModel};

use crate::components::{
    BackLink, Button, ButtonVariant, Card, DragHandle, DropIndicator, PageHeading,
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

    let Some(routine) = routine else {
        return view! {
            <div class="text-center py-8">
                <p class="text-secondary mb-4">"Routine not found."</p>
                <A href="/routines" attr:class="text-accent-text hover:text-accent-hover font-medium">
                    "\u{2190} Back to Routines"
                </A>
            </div>
        }
        .into_any();
    };

    let routine_id = routine.id.clone();
    let name = RwSignal::new(routine.name.clone());
    let entries: RwSignal<Vec<RoutineEntryView>> = RwSignal::new(routine.entries.clone());
    let name_error = RwSignal::new(Option::<String>::None);

    let core_save = core;

    // --- Drag-and-drop setup for routine entries ---
    let entries_container_ref = NodeRef::<leptos::html::Div>::new();

    let item_count = Signal::derive(move || entries.get().len());

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

    let drag = use_drag_reorder(on_reorder, item_count, entries_container_ref);
    let dragged_id = drag.dragged_id;
    let drag_hover_index = drag.hover_index;
    let on_drag_pointer_down = drag.on_pointer_down;

    view! {
        <div class="sm:max-w-2xl sm:mx-auto">
            <BackLink label="Back to Routines" href="/routines".to_string() />

            <PageHeading text="Edit Routine" />

            <div class="space-y-6">
                // Name field
                <Card>
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
                            <label for="routine-name" class="form-label">"Routine Name"</label>
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

                        // Current entries with drag handle + reorder/remove (T021, T022)
                        <div>
                            <h4 class="text-sm font-medium text-secondary mb-2">"Entries"</h4>
                            {move || {
                                let current = entries.get();
                                if current.is_empty() {
                                    view! {
                                        <p class="text-sm text-muted text-center py-4">"No entries. Add items from your library below."</p>
                                    }.into_any()
                                } else {
                                    let len = current.len();
                                    view! {
                                        <div node_ref=entries_container_ref aria-roledescription="sortable">
                                            {current.into_iter().enumerate().map(|(idx, entry)| {
                                                let entry_id = entry.id.clone();
                                                let entry_id_up = entry.id.clone();
                                                let entry_id_down = entry.id.clone();

                                                // Drag state for this entry
                                                let eid = entry.id.clone();
                                                let is_dragging_this = Signal::derive(move || {
                                                    dragged_id.get().as_deref() == Some(eid.as_str())
                                                });

                                                // Drop indicator before this entry
                                                let drop_before_visible = Signal::derive(move || {
                                                    drag_hover_index.get() == Some(idx)
                                                });

                                                // Drop indicator after last entry
                                                let is_last = idx == len - 1;
                                                let drop_after_visible = Signal::derive(move || {
                                                    is_last && drag_hover_index.get() == Some(len)
                                                });

                                                view! {
                                                    <DropIndicator visible=drop_before_visible />
                                                    <div
                                                        class=move || {
                                                            if is_dragging_this.get() {
                                                                "flex items-center justify-between rounded-lg bg-surface-secondary px-3 py-2 drag-active ring-2 ring-accent-focus"
                                                            } else {
                                                                "flex items-center justify-between rounded-lg bg-surface-secondary px-3 py-2"
                                                            }
                                                        }
                                                        data-entry-index=idx.to_string()
                                                    >
                                                        <div class="flex items-center gap-2">
                                                            // Drag handle (leftmost)
                                                            <DragHandle
                                                                entry_id=entry.id.clone()
                                                                index=idx
                                                                on_pointer_down=on_drag_pointer_down
                                                            />
                                                            <span class="text-xs text-faint w-5 text-center">{idx + 1}</span>
                                                            <span class="text-sm text-primary">{entry.item_title}</span>
                                                            <span class="text-xs text-faint">{entry.item_type}</span>
                                                        </div>
                                                        <div class="flex items-center gap-1">
                                                            // Up/down arrow buttons always visible (FR-012)
                                                            {if idx > 0 {
                                                                Some(view! {
                                                                    <button
                                                                        type="button"
                                                                        class="text-xs text-muted hover:text-primary p-1"
                                                                        title="Move up"
                                                                        on:click=move |_| {
                                                                            entries.update(|e| {
                                                                                if let Some(pos) = e.iter().position(|x| x.id == entry_id_up) {
                                                                                    if pos > 0 {
                                                                                        e.swap(pos, pos - 1);
                                                                                    }
                                                                                }
                                                                            });
                                                                        }
                                                                    >
                                                                        "\u{2191}"
                                                                    </button>
                                                                })
                                                            } else {
                                                                None
                                                            }}
                                                            {if idx < len - 1 {
                                                                Some(view! {
                                                                    <button
                                                                        type="button"
                                                                        class="text-xs text-muted hover:text-primary p-1"
                                                                        title="Move down"
                                                                        on:click=move |_| {
                                                                            entries.update(|e| {
                                                                                if let Some(pos) = e.iter().position(|x| x.id == entry_id_down) {
                                                                                    if pos < e.len() - 1 {
                                                                                        e.swap(pos, pos + 1);
                                                                                    }
                                                                                }
                                                                            });
                                                                        }
                                                                    >
                                                                        "\u{2193}"
                                                                    </button>
                                                                })
                                                            } else {
                                                                None
                                                            }}
                                                            <button
                                                                type="button"
                                                                class="text-xs text-danger-text hover:text-danger-hover p-1"
                                                                title="Remove"
                                                                on:click=move |_| {
                                                                    entries.update(|e| {
                                                                        e.retain(|x| x.id != entry_id);
                                                                    });
                                                                }
                                                            >
                                                                "\u{2715}"
                                                            </button>
                                                        </div>
                                                    </div>
                                                    {if is_last {
                                                        Some(view! { <DropIndicator visible=drop_after_visible /> })
                                                    } else {
                                                        None
                                                    }}
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
                </Card>

                // Add from Library (T014: whole row is clickable)
                <Card>
                    <h3 class="section-title">"Add from Library"</h3>
                    {move || {
                        let vm = view_model.get();
                        if vm.items.is_empty() {
                            view! {
                                <p class="text-sm text-muted">"No library items available."</p>
                            }.into_any()
                        } else {
                            // Collect item IDs already in the routine to dim them
                            let added_ids: std::collections::HashSet<String> = entries
                                .get()
                                .iter()
                                .map(|e| e.item_id.clone())
                                .collect();
                            view! {
                                <div class="space-y-2">
                                    {vm.items.iter().map(|item| {
                                        let already_added = added_ids.contains(&item.id);
                                        let title = item.title.clone();
                                        let item_type = item.item_type.clone();
                                        let id_for_entry = item.id.clone();
                                        let title_for_entry = item.title.clone();
                                        let type_for_entry = item.item_type.clone();
                                        view! {
                                            <div
                                                class={if already_added {
                                                    "flex items-center justify-between rounded-lg bg-surface-secondary px-3 py-2 opacity-50"
                                                } else {
                                                    "flex items-center justify-between rounded-lg bg-surface-secondary px-3 py-2 hover:bg-surface-hover cursor-pointer"
                                                }}
                                                on:click=move |_| {
                                                    let new_entry = RoutineEntryView {
                                                        id: ulid::Ulid::new().to_string(),
                                                        item_id: id_for_entry.clone(),
                                                        item_title: title_for_entry.clone(),
                                                        item_type: type_for_entry.clone(),
                                                        position: entries.get_untracked().len(),
                                                    };
                                                    entries.update(|e| e.push(new_entry));
                                                }
                                            >
                                                <div class="flex items-center gap-2">
                                                    <span class="text-sm text-primary">{title}</span>
                                                    <span class="text-xs text-faint">{item_type}</span>
                                                </div>
                                                <span class={if already_added {
                                                    "text-xs font-medium text-muted"
                                                } else {
                                                    "text-xs font-medium text-accent-text"
                                                }}>
                                                    {if already_added { "Added" } else { "+ Add" }}
                                                </span>
                                            </div>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        }
                    }}
                </Card>
            </div>
        </div>
    }.into_any()
}
