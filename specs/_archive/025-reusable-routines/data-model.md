# Data Model: Reusable Routines

**Feature**: 025-reusable-routines
**Date**: 2026-02-18

## New Entities

### Routine (persisted)

A named, reusable setlist template containing an ordered list of library item references.

```
Routine
├── id: String (ULID)                    # unique identifier
├── name: String                         # 1–200 characters
├── entries: Vec<RoutineEntry>           # ordered list of items
├── created_at: DateTime<Utc>            # when the routine was first saved
└── updated_at: DateTime<Utc>            # when the routine was last modified
```

**Validation rules**:
- `name` is required: must be 1–200 characters (trimmed)
- `entries` must be non-empty: at least one entry required
- Duplicate names are allowed (routines identified by ID, not name)

### RoutineEntry (persisted, child of Routine)

A single item within a routine, representing a library piece or exercise.

```
RoutineEntry
├── id: String (ULID)                    # unique identifier
├── item_id: String (ULID)              # reference to the library item
├── item_title: String                   # denormalized snapshot of item title
├── item_type: String                    # "piece" | "exercise"
└── position: usize                      # 0-indexed order within routine
```

**Denormalization**: `item_title` and `item_type` are copied from the library item at save time. This ensures the routine remains readable even if the source item is renamed or deleted. This matches the `SetlistEntry` denormalization pattern.

## View Types

### RoutineView (view model)

Represents a routine for display in the UI.

```
RoutineView
├── id: String                           # routine ID
├── name: String                         # routine name
├── entry_count: usize                   # number of entries (for list display)
└── entries: Vec<RoutineEntryView>       # full entry list (for edit page, loader)
```

**Derivation**: Computed in core `view()` by mapping each `Routine` in `model.routines`.

### RoutineEntryView (view model)

Represents a single entry within a routine for display.

```
RoutineEntryView
├── id: String                           # entry ID
├── item_title: String                   # denormalized title
├── item_type: String                    # "piece" | "exercise"
└── position: usize                      # display order
```

### Modified: ViewModel

```
ViewModel
├── items: Vec<LibraryItemView>          # existing (unchanged)
├── sessions: Vec<PracticeSessionView>   # existing (unchanged)
├── active_session: Option<...>          # existing (unchanged)
├── building_setlist: Option<...>        # existing (unchanged)
├── summary: Option<...>                 # existing (unchanged)
├── session_status: String               # existing (unchanged)
├── error: Option<String>                # existing (unchanged)
├── analytics: Option<AnalyticsView>     # existing (unchanged)
└── routines: Vec<RoutineView>           # NEW — all saved routines
```

### Modified: Model

```
Model
├── pieces: Vec<Piece>                   # existing (unchanged)
├── exercises: Vec<Exercise>             # existing (unchanged)
├── sessions: Vec<PracticeSession>       # existing (unchanged)
├── session_status: SessionStatus        # existing (unchanged)
├── active_query: Option<ListQuery>      # existing (unchanged)
├── last_error: Option<String>           # existing (unchanged)
└── routines: Vec<Routine>               # NEW — all saved routines
```

## Database Schema

### New Table: routines

```sql
CREATE TABLE IF NOT EXISTS routines (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

### New Table: routine_entries

```sql
CREATE TABLE IF NOT EXISTS routine_entries (
    id TEXT PRIMARY KEY NOT NULL,
    routine_id TEXT NOT NULL REFERENCES routines(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL,
    item_title TEXT NOT NULL,
    item_type TEXT NOT NULL,
    position INTEGER NOT NULL
);
```

### New Index: routine_entries.routine_id

```sql
CREATE INDEX IF NOT EXISTS idx_routine_entries_routine_id ON routine_entries(routine_id);
```

**Notes**:
- `routine_entries.routine_id` has ON DELETE CASCADE — deleting a routine removes all its entries
- Index on `routine_id` supports efficient entry lookup when listing routines
- Timestamps stored as ISO 8601 text (consistent with existing tables)
- Three separate migrations (one per SQL statement, matching the single-statement-per-migration constraint)

## Event Changes

### New: RoutineEvent enum

```
RoutineEvent
├── SaveBuildingAsRoutine { name: String }
├── SaveSummaryAsRoutine { name: String }
├── LoadRoutineIntoSetlist { routine_id: String }
├── DeleteRoutine { id: String }
└── UpdateRoutine { id: String, name: String, entries: Vec<RoutineEntry> }
```

### Event Details

**SaveBuildingAsRoutine { name }**:
- Precondition: `session_status` must be `Building`, setlist must have entries
- Validates: name (1–200 chars), entries non-empty
- Creates `Routine` from `BuildingSession.entries` (copies item_id, item_title, item_type, reindexes positions)
- Pushes to `model.routines`
- Emits `StorageEffect::SaveRoutine(routine)`
- Does NOT change building state (user can continue building)

**SaveSummaryAsRoutine { name }**:
- Precondition: `session_status` must be `Summary`, summary must have entries
- Same validation and creation logic as above, using `SummarySession.entries` as source
- Does NOT change summary state

**LoadRoutineIntoSetlist { routine_id }**:
- Precondition: `session_status` must be `Building`
- Finds routine by ID in `model.routines`
- Creates new `SetlistEntry` objects with fresh ULIDs for each `RoutineEntry`
- Sets `duration_secs: 0`, `status: NotAttempted`, `notes: None`, `score: None`
- Appends to existing `BuildingSession.entries`, reindexes all positions
- Emits render (no storage effect — session not yet persisted)

**DeleteRoutine { id }**:
- Removes routine from `model.routines` by ID
- Emits `StorageEffect::DeleteRoutine { id }`

**UpdateRoutine { id, name, entries }**:
- Validates: name (1–200 chars), entries non-empty
- Finds routine by ID, replaces name and entries, updates `updated_at`
- Emits `StorageEffect::UpdateRoutine(routine)`

### New: StorageEffect variants

```
StorageEffect
├── ... (existing variants)
├── LoadRoutines                         # NEW — fetch all routines from API
├── SaveRoutine(Routine)                 # NEW — create routine via API
├── UpdateRoutine(Routine)               # NEW — update routine via API
└── DeleteRoutine { id: String }         # NEW — delete routine via API
```

### New: Event variant

```
Event
├── ... (existing variants)
├── Routine(RoutineEvent)                # NEW — wraps routine events
└── RoutinesLoaded { routines: Vec<Routine> }  # NEW — callback after LoadRoutines
```

## Relationships

```
Routine 1──* RoutineEntry (entries, ordered by position)
                │
                └── item_id ──► Library Item (piece or exercise, denormalized)

BuildingSession 1──* SetlistEntry
                          │
   LoadRoutineIntoSetlist ─┘ (creates new SetlistEntries from RoutineEntries)
```

## Data Flow

### Save Flow

```
User taps "Save as Routine" → enters name → taps Save
  → Event::Routine(SaveBuildingAsRoutine { name })
    → Core: validate name, create Routine from BuildingSession.entries
    → Core: push to model.routines
    → Effect: StorageEffect::SaveRoutine(routine)
      → Shell: POST /api/routines
      → Shell: refresh_routines() → Event::RoutinesLoaded
```

### Load Flow

```
User taps "Load" on a routine
  → Event::Routine(LoadRoutineIntoSetlist { routine_id })
    → Core: find routine in model.routines
    → Core: create SetlistEntry objects from RoutineEntry objects (fresh ULIDs)
    → Core: append to BuildingSession.entries, reindex positions
    → Effect: Render (no storage — session not yet saved)
```

### Startup Flow

```
App init → fetch_initial_data()
  → Parallel: fetch_pieces + fetch_exercises
  → Parallel: fetch_sessions
  → Parallel: fetch_routines
    → Event::RoutinesLoaded { routines }
      → Core: model.routines = routines
```
