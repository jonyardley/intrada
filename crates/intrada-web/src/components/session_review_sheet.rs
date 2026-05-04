use leptos::prelude::*;

use intrada_core::{Event, SessionEvent, ViewModel};

use crate::components::{BottomSheet, DropIndicator, SetlistEntryRow};
use intrada_web::core_bridge::process_effects;
use intrada_web::hooks::use_drag_reorder;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Bottom sheet that opens from the Review session CTA in the builder.
/// Stripped to the essentials per the Pencil `AYx23` design: session
/// intention, drag-reorderable entry list with a per-row remove button,
/// and a total. The Start action lives in the sheet's nav bar (right side,
/// opposite Cancel) — iOS Mail-compose pattern.
///
/// Per-entry rep target / planned duration controls and Save-as-Routine
/// are intentionally not here — see #389 (per-entry controls) and #390
/// (save/load routine) for the planned re-introductions.
#[component]
pub fn SessionReviewSheet(open: Signal<bool>, on_close: Callback<()>) -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    let core_start = core.clone();

    let setlist_empty = Signal::derive(move || {
        view_model
            .get()
            .building_setlist
            .map(|s| s.entries.is_empty())
            .unwrap_or(true)
    });

    let on_start = Callback::new(move |_| {
        let now = chrono::Utc::now();
        let event = Event::Session(SessionEvent::StartSession { now });
        let core_ref = core_start.borrow();
        let effects = core_ref.process_event(event);
        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
    });

    view! {
        <BottomSheet
            open=open
            on_close=on_close
            nav_title="Review session".to_string()
            nav_action_label="Start".to_string()
            on_nav_action=on_start
            nav_action_disabled=setlist_empty
        >
            <ReviewSheetBody />
        </BottomSheet>
    }
}

/// Body of the review sheet rendered inside [`BottomSheet`]'s `ChildrenFn`.
/// Lives in its own component so per-entry move-closures don't have to
/// satisfy `Fn` for the BottomSheet's children prop — the Leptos component
/// boundary breaks the closure-trait dependency chain.
#[component]
fn ReviewSheetBody() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    let core_intention = core.clone();
    let core_remove = core.clone();
    let core_drag = core.clone();

    let setlist_container_ref = NodeRef::<leptos::html::Div>::new();

    let item_count = Signal::derive(move || {
        view_model
            .get()
            .building_setlist
            .as_ref()
            .map(|s| s.entries.len())
            .unwrap_or(0)
    });

    let on_reorder = Callback::new(move |(entry_id, new_position): (String, usize)| {
        let event = Event::Session(SessionEvent::ReorderSetlist {
            entry_id,
            new_position,
        });
        let core_ref = core_drag.borrow();
        let effects = core_ref.process_event(event);
        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
    });

    let drag = use_drag_reorder(on_reorder, item_count, setlist_container_ref);
    let dragged_id = drag.dragged_id;
    let drag_hover_index = drag.hover_index;
    let on_drag_pointer_down = drag.on_pointer_down;

    // Local intention signal seeded from VM each open.
    let session_intention_value = RwSignal::new(String::new());
    Effect::new(move |_| {
        let vm = view_model.get();
        let current = vm
            .building_setlist
            .as_ref()
            .and_then(|s| s.session_intention.clone())
            .unwrap_or_default();
        if session_intention_value.get_untracked() != current {
            session_intention_value.set(current);
        }
    });

    view! {
        <div class="flex flex-col gap-5 pb-2">
            <div>
                <label class="form-label" for="session-intention">"Session intention"</label>
                <input
                    id="session-intention"
                    type="text"
                    class="input-base"
                    placeholder="e.g. Focus on dynamics"
                    bind:value=session_intention_value
                    on:input=move |_| {
                        let value = session_intention_value.get();
                        let intention = if value.is_empty() { None } else { Some(value) };
                        let event = Event::Session(SessionEvent::SetSessionIntention { intention });
                        let core_ref = core_intention.borrow();
                        let effects = core_ref.process_event(event);
                        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                    }
                />
            </div>

            {move || {
                let vm = view_model.get();
                match vm.building_setlist {
                    Some(ref setlist) if !setlist.entries.is_empty() => {
                        let entries = setlist.entries.clone();
                        let entry_count = entries.len();
                        let total_mins: u32 = setlist
                            .entries
                            .iter()
                            .map(|e| {
                                e.planned_duration_secs.unwrap_or(
                                    intrada_core::validation::DEFAULT_PLANNED_DURATION_SECS,
                                )
                            })
                            .sum::<u32>() / 60;
                        let core_r = core_remove.clone();
                        view! {
                            <div node_ref=setlist_container_ref aria-roledescription="sortable" class="flex flex-col">
                                {entries.into_iter().enumerate().map(|(idx, entry)| {
                                    let core_r2 = core_r.clone();
                                    let on_remove = Callback::new(move |entry_id: String| {
                                        let event = Event::Session(SessionEvent::RemoveFromSetlist { entry_id });
                                        let core_ref = core_r2.borrow();
                                        let effects = core_ref.process_event(event);
                                        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                    });

                                    let eid = entry.id.clone();
                                    let is_dragging_this = Signal::derive(move || {
                                        dragged_id.get().as_deref() == Some(eid.as_str())
                                    });
                                    let drop_before_visible = Signal::derive(move || drag_hover_index.get() == Some(idx));
                                    let is_last = idx == entry_count - 1;
                                    let drop_after_visible = Signal::derive(move || is_last && drag_hover_index.get() == Some(entry_count));

                                    view! {
                                        <DropIndicator visible=drop_before_visible />
                                        <SetlistEntryRow
                                            entry=entry
                                            on_remove=Some(on_remove)
                                            show_controls=true
                                            is_dragging_this=is_dragging_this
                                            on_drag_pointer_down=Some(on_drag_pointer_down)
                                            index=idx
                                            compact=true
                                        />
                                        {if is_last {
                                            Some(view! { <DropIndicator visible=drop_after_visible /> })
                                        } else {
                                            None
                                        }}
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                            <div class="flex justify-end pt-2">
                                <span class="text-xs font-medium text-muted">
                                    {format!("Total: {total_mins} min")}
                                </span>
                            </div>
                        }.into_any()
                    }
                    _ => view! {
                        <p class="text-sm text-muted text-center py-6">
                            "No items in your setlist yet. Tap items in the library to add them."
                        </p>
                    }.into_any(),
                }
            }}

            {move || {
                let vm = view_model.get();
                vm.error.map(|err| view! {
                    <p class="text-sm text-danger-text">{err}</p>
                })
            }}
        </div>
    }
}
