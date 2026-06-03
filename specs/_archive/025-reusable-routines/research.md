# Research: Reusable Routines

**Feature**: 025-reusable-routines
**Date**: 2026-02-18

## Overview

This feature introduces a new `Routine` domain entity for saving and loading reusable setlist templates. The research below resolves all technical decisions needed before design and implementation.

## Decision 1: Routine Domain Structure

**Decision**: New `Routine` struct with `Vec<RoutineEntry>` children, following the `PracticeSession` + `Vec<SetlistEntry>` parent/child pattern. Routine is a top-level domain entity in its own module (`domain/routine.rs`), not a variant of existing types.

**Rationale**: Routines are conceptually separate from sessions — a routine is a template, not a record of practice. They have different lifecycles (routines are edited and reused; sessions are immutable once completed). Using the parent/child pattern from sessions ensures consistency in database design, API structure, and core event handling. A separate module keeps domain boundaries clean.

**Alternatives considered**:
- Reuse `PracticeSession` with a "template" flag: Conflates two distinct concepts. Sessions have timestamps, duration, completion status, and scores — none of which apply to routines.
- Store routines as JSON blobs: Loses the ability to query or validate individual entries at the database level.
- Embed routines inside the session domain module: Violates single responsibility. Routines have their own CRUD lifecycle independent of sessions.

## Decision 2: RoutineEntry Fields

**Decision**: `RoutineEntry` stores `id` (ULID), `item_id`, `item_title` (denormalized), `item_type` ("piece" | "exercise"), and `position` (0-indexed). No duration, status, notes, or score fields.

**Rationale**: A routine entry is a reference to a library item with an ordering, not a practice record. Duration, status, notes, and scores are session-time concepts that belong on `SetlistEntry`, not `RoutineEntry`. Denormalizing `item_title` and `item_type` ensures routines remain readable even if the source library item is renamed or deleted — consistent with how `SetlistEntry` denormalizes the same fields.

**Alternatives considered**:
- Include `duration_secs` for default practice duration per item: Over-engineering for MVP. Musicians set duration during the session, not when defining a routine.
- Reference by `item_id` only (no denormalization): Routine would break or show blank titles if a library item is deleted. Denormalization is the established pattern.
- Include `notes` field: Routine entries are templates, not practice records. Notes belong on the session entry when actually practising.

## Decision 3: Event Architecture

**Decision**: New `RoutineEvent` enum with five variants: `SaveBuildingAsRoutine { name }`, `SaveSummaryAsRoutine { name }`, `LoadRoutineIntoSetlist { routine_id }`, `DeleteRoutine { id }`, `UpdateRoutine { id, name, entries }`. Wrapped in `Event::Routine(RoutineEvent)` at the app level.

**Rationale**: Each event corresponds to a distinct user action from the spec. Save has two variants (building vs. summary) because the data source differs — `BuildingSession.entries` vs. `SummarySession.entries`. Both validate the same way (name required, 1–200 chars, entries non-empty). `LoadRoutineIntoSetlist` creates fresh `SetlistEntry` objects with new ULIDs from routine entries, appended to the existing building setlist. This follows the existing pattern where each domain area has its own event enum.

**Alternatives considered**:
- Single `SaveAsRoutine` event that auto-detects source from session status: Implicit behaviour makes testing harder and creates ambiguous failure modes.
- Separate `RenameRoutine` and `ReorderRoutineEntries` events: Over-granular for MVP. A single `UpdateRoutine` event replaces name and entries atomically, matching the API's PUT semantics.

## Decision 4: StorageEffect Strategy

**Decision**: Four new `StorageEffect` variants: `LoadRoutines`, `SaveRoutine(Routine)`, `UpdateRoutine(Routine)`, `DeleteRoutine { id }`. Plus a new `Event::RoutinesLoaded { routines }` for the load callback.

**Rationale**: Follows the exact pattern established by pieces, exercises, and sessions. Each mutation effect triggers a server-side API call in the shell, followed by a full refresh of routines from the server. The load effect is dispatched on app startup and triggers `RoutinesLoaded` when data arrives. Keeping routine effects separate from existing effects (rather than reusing `DeleteItem`) makes the shell's effect processing clearer and avoids coupling routine deletion to the library item deletion flow.

**Alternatives considered**:
- Reuse `DeleteItem { id, item_type: "routine" }`: Would require the shell's `DeleteItem` handler to know about the routines API endpoint. Currently `DeleteItem` only handles library items (pieces/exercises). Mixing concerns.
- No `LoadRoutines` effect (fetch routines as part of `LoadAll`): Would require modifying the existing `DataLoaded` event. Adding a separate load/loaded pair keeps the change additive and avoids touching the library data flow.

## Decision 5: ViewModel Representation

**Decision**: Add `routines: Vec<RoutineView>` to `ViewModel`. `RoutineView` contains `id`, `name`, `entry_count`, and `entries: Vec<RoutineEntryView>`. Routines are NOT added to the `items` list (they are not library items).

**Rationale**: Routines are a distinct concept from library items. They should not appear in library search/filter results or on the library list page. The SetlistBuilder and routine management pages read from `vm.routines` directly. Keeping routines out of `items` avoids polluting the library view and simplifies filtering logic. Entry count is pre-computed for display efficiency.

**Alternatives considered**:
- Add routines to the `items` list as `item_type: "routine"`: Would show routines on the library page, which is confusing — routines are templates, not things to practise.
- Only store routine names (no entries) in the ViewModel: The edit page and loader need entry details. Pre-computing the view avoids the shell needing to re-query core.

## Decision 6: Database Schema

**Decision**: Two new tables: `routines` (id, name, created_at, updated_at) and `routine_entries` (id, routine_id, item_id, item_title, item_type, position). Foreign key with ON DELETE CASCADE. Two migrations (one per table, matching the single-statement-per-migration constraint).

**Rationale**: Mirrors the `sessions` + `setlist_entries` pattern exactly. Parent/child with foreign key cascade ensures orphaned entries are cleaned up on routine deletion. The single-statement constraint is enforced by an existing test in `migrations.rs`. An index on `routine_entries.routine_id` supports efficient entry lookups when listing routines with their entries.

**Alternatives considered**:
- Single table with JSON entries column: Loses ability to query individual entries, no referential integrity.
- Three tables (routines, routine_entries, routine_entry_details): Unnecessary normalization for this use case.

## Decision 7: API Design

**Decision**: Standard REST CRUD nested under `/api/routines`. `GET /` returns all routines with entries. `POST /` creates a routine (201 Created). `GET /:id` returns one routine with entries. `PUT /:id` replaces name and entries (full replacement, not PATCH). `DELETE /:id` removes routine and cascading entries.

**Rationale**: Follows the sessions API pattern. Full replacement on PUT simplifies the API — the client sends the complete routine state (name + all entries), and the server replaces entries in a transaction. This avoids the complexity of partial updates (add entry, remove entry, reorder entries as separate operations). At the scale of a single user with dozens of routines, this is efficient.

**Alternatives considered**:
- PATCH for partial updates: More complex server-side logic (diff entries, handle reordering). Not worth the complexity at this scale.
- Separate `/api/routines/:id/entries` sub-resource: Over-engineered. Entries are always managed as part of the routine, not independently.

## Decision 8: Loading Semantics

**Decision**: When loading a routine into a building setlist, create new `SetlistEntry` objects with fresh ULID IDs and positions appended after existing entries. Copy `item_id`, `item_title`, and `item_type` from `RoutineEntry`. Set `duration_secs: 0`, `status: NotAttempted`, `notes: None`, `score: None`.

**Rationale**: Fresh IDs prevent conflicts with existing setlist entries or entries from multiple loads of the same routine. Appending preserves the user's existing setlist (additive loading per spec). Default values for duration/status/notes/score match how new entries are created during normal setlist building. The routine provides the "what to practise" template; the session provides the "how it went" data.

**Alternatives considered**:
- Clone routine entry IDs into setlist entries: Would cause ID collisions when loading the same routine twice.
- Replace existing setlist: Spec explicitly states additive loading. Replacement would lose manually added items.

## Decision 9: Web Shell Integration Points

**Decision**: Two new shared components (`RoutineSaveForm`, `RoutineLoader`) used across SetlistBuilder and Session Summary. Two new view pages (`RoutinesListView`, `RoutineEditView`). The save form is an inline expand/collapse pattern (tap "Save as Routine" → name input + Save/Cancel buttons appear).

**Rationale**: Extracting the save form into a shared component avoids duplicating the name input + validation + event dispatch logic between SetlistBuilder and Session Summary. The loader component encapsulates the routine list display and load-button dispatch. This follows the project's component extraction pattern (TextField, TextArea, BackLink, etc.). The inline save form avoids modal dialogs, keeping the interaction lightweight and consistent with the app's existing form patterns.

**Alternatives considered**:
- Modal dialog for save: Adds UI complexity (overlay, focus trap). Inline is simpler and consistent with how notes are entered on entries.
- Inline save directly in SetlistBuilder/Summary without extraction: Would duplicate logic across two locations. Extraction follows the established component pattern.
