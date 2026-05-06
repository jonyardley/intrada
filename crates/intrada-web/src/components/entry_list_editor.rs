use leptos::prelude::*;

use intrada_core::ItemKind;

use crate::components::SetlistEntryRow;
use intrada_web::hooks::use_drag_reorder;

/// Minimal projection of a setlist or routine entry — just what the
/// editor row needs to render. Callers project from their domain
/// view-model (`SetlistEntryView`, `RoutineEntryView`) into this.
///
/// Keeps the shared component decoupled from the two divergent
/// view-model shapes (routines have 5 fields; sessions have 16). The
/// projection is trivial — `Vec::iter().map(|e| EditorEntry { ... })`
/// — so the cost of the abstraction is one tiny adapter at each call
/// site. The win is the editor doesn't have to know about the
/// session-only fields (status, score, rep state, etc.).
#[derive(Clone, Debug)]
pub struct EditorEntry {
    pub id: String,
    pub item_title: String,
    pub item_type: ItemKind,
    /// `None` hides the duration line on the row — routines don't carry
    /// planned durations, sessions do.
    pub duration_display: Option<String>,
}

/// Reorderable list of entries with the production drag pattern
/// (`use_drag_reorder` + `<SetlistEntryRow compact>` rows that
/// physically translate). The shared editor body used by:
///
/// - `<SessionReviewSheet>` for building a session setlist
/// - `<RoutineEditView>` for editing a saved routine
///
/// Both surfaces previously inlined the same drag-handle plumbing,
/// the same compact row markup, and the same "remove" wiring. This
/// component owns that shape; callers handle the dispatch.
///
/// **Reorder mechanism**: drag-only. Rows render in compact mode
/// (no `on_move_up`/`on_move_down` button affordances). If a future
/// caller needs button-based reorder, use `<SetlistEntryRow>`
/// directly — this primitive is committed to the drag idiom.
#[component]
pub fn EntryListEditor(
    /// Source of truth for the rows. Callers update this signal in
    /// response to their `on_reorder` / `on_remove` callbacks.
    #[prop(into)]
    entries: Signal<Vec<EditorEntry>>,
    /// Fired with `(entry_id, new_position)` when a drop commits a
    /// reorder. Callers either dispatch a Crux event or mutate a
    /// local signal directly.
    on_reorder: Callback<(String, usize)>,
    /// Fired with the entry id when the user taps the row's remove
    /// affordance. Optional — pass `None` to hide remove controls.
    #[prop(optional)]
    on_remove: Option<Callback<String>>,
) -> impl IntoView {
    let container_ref = NodeRef::<leptos::html::Div>::new();
    let drag = use_drag_reorder(on_reorder, container_ref);
    let dragged_id = drag.dragged_id;
    let on_drag_pointer_down = drag.on_pointer_down;
    let show_controls = on_remove.is_some();

    view! {
        <div node_ref=container_ref aria-roledescription="sortable" class="flex flex-col">
            {move || {
                entries.get().into_iter().enumerate().map(|(idx, entry)| {
                    let eid = entry.id.clone();
                    let is_dragging_this = Signal::derive(move || {
                        dragged_id.get().as_deref() == Some(eid.as_str())
                    });
                    let duration = entry.duration_display.clone().unwrap_or_default();
                    view! {
                        <div style=drag.row_style_for(idx) data-entry-index=idx.to_string()>
                            <SetlistEntryRow
                                id=entry.id.clone()
                                item_title=entry.item_title.clone()
                                item_type=entry.item_type.clone()
                                duration_display=duration
                                position=idx
                                on_remove=on_remove
                                show_controls=show_controls
                                is_dragging_this=is_dragging_this
                                on_drag_pointer_down=Some(on_drag_pointer_down)
                                index=idx
                                compact=true
                            />
                        </div>
                    }
                }).collect::<Vec<_>>()
            }}
        </div>
    }
}
