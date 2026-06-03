# Data Model: Drag-and-Drop Session Builder

**Feature**: 026-drag-drop-builder
**Date**: 2026-02-18

## No Data Model Changes

This feature is a **shell-only UI interaction change**. No domain entities, database tables, API contracts, or persistence formats are modified.

### Existing Entities (unchanged)

| Entity | Crate | Impact |
|---|---|---|
| `SetlistEntry` | intrada-core | None — `position` field updated via existing `ReorderSetlist` event |
| `BuildingSession` | intrada-core | None — `entries: Vec<SetlistEntry>` managed by existing core logic |
| `RoutineEntryView` | intrada-core | None — reordered via local `RwSignal<Vec>` in routine edit view |
| Library items (`Piece`, `Exercise`) | intrada-core | None — tap-to-add uses existing `AddToSetlist` event |

### Existing Events (reused as-is)

| Event | Core Module | Reused By |
|---|---|---|
| `SessionEvent::ReorderSetlist { entry_id, new_position }` | `domain/session.rs` | Session builder drag-and-drop |
| `SessionEvent::AddToSetlist { item_id }` | `domain/session.rs` | Library row tap-to-add |

### New Transient State (shell only, not persisted)

A `DragState` struct is introduced in `intrada-web` to track the in-progress drag interaction:

| Field | Type | Purpose |
|---|---|---|
| `dragged_entry_id` | `String` | ULID of the entry being dragged |
| `pointer_id` | `i32` | For `setPointerCapture` / `releasePointerCapture` |
| `start_y` | `f64` | clientY at pointerdown |
| `current_y` | `f64` | clientY during pointermove |
| `source_index` | `usize` | Original position in the list |
| `hover_index` | `Option<usize>` | Computed drop-target position |

This state lives as `RwSignal<Option<DragState>>` in the web shell. It is never serialised, never persisted, and is reset to `None` on `pointerup`/`pointercancel`.

### API Contracts

No API changes. No new endpoints. No modified request/response schemas.
