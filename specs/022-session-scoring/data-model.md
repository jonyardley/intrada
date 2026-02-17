# Data Model: Session Item Scoring

**Feature**: 022-session-scoring
**Date**: 2026-02-17

## Entity Changes

### Modified: SetlistEntry (domain)

The core domain entity `SetlistEntry` gains a new optional `score` field.

```
SetlistEntry
├── id: String (ULID)                    # existing
├── item_id: String (ULID)               # existing
├── item_title: String                   # existing (snapshot)
├── item_type: String                    # existing ("piece" | "exercise")
├── position: usize                      # existing (0-indexed)
├── duration_secs: u64                   # existing
├── status: EntryStatus                  # existing (Completed | Skipped | NotAttempted)
├── notes: Option<String>                # existing
└── score: Option<u8>                    # NEW — confidence score 1–5, None = not scored
```

**Validation rules**:
- Score must be `None` or a value in range 1–5 (inclusive)
- Score is only meaningful when `status == Completed`; core ignores score on Skipped/NotAttempted entries
- Score defaults to `None` when entry is created

### Modified: SummarySession (transient)

The `SummarySession` struct is transient (exists only during session review, not persisted). It contains `Vec<SetlistEntry>`, so it inherits the score field automatically. No structural changes needed — the existing `UpdateEntryNotes` pattern is mirrored by a new `UpdateEntryScore` event.

### Modified: ActiveSession (transient, localStorage)

The `ActiveSession` struct is persisted to localStorage for crash recovery. It contains `Vec<SetlistEntry>`, so it inherits the score field. Since scoring only happens during the Summary phase (after Active), the score field will always be `None` during the Active phase. Backward compatible — deserialization of old ActiveSession data (without score) will default to `None` via `serde`.

### Unchanged: PracticeSession (persisted)

`PracticeSession` contains `Vec<SetlistEntry>`, so it inherits the score field automatically. No structural changes needed.

## New View Types

### ScoreHistoryEntry (view model)

A single data point in the progress history for a library item.

```
ScoreHistoryEntry
├── session_date: String                 # RFC3339 timestamp of the session's started_at
├── score: u8                            # confidence score 1–5
└── session_id: String                   # reference to the parent session (for potential linking)
```

**Derivation**: Computed in core `view()` by filtering all session entries where `item_id` matches and `score.is_some()`, ordered by `session.started_at` descending (most recent first).

### Modified: SetlistEntryView

```
SetlistEntryView
├── id: String                           # existing
├── item_id: String                      # existing
├── item_title: String                   # existing
├── item_type: String                    # existing
├── position: usize                      # existing
├── duration_display: String             # existing
├── status: String                       # existing
├── notes: Option<String>                # existing
└── score: Option<u8>                    # NEW — raw score value for display
```

### Modified: SummaryView

```
SummaryView
├── total_duration_display: String       # existing
├── completion_status: String            # existing
├── notes: Option<String>                # existing
└── entries: Vec<SetlistEntryView>       # existing (entries now include score)
```

No structural change — entries already carry the updated `SetlistEntryView`.

### Modified: ItemPracticeSummary

```
ItemPracticeSummary
├── session_count: usize                 # existing
├── total_minutes: u32                   # existing
├── latest_score: Option<u8>            # NEW — most recent confidence score, None if never scored
└── score_history: Vec<ScoreHistoryEntry> # NEW — all scored entries, most recent first
```

### Modified: LibraryItemView

No structural change. `LibraryItemView` already contains `practice: Option<ItemPracticeSummary>`, which now includes the new fields. Per FR-011, score data is NOT displayed on the library list — only on the detail page where `ItemPracticeSummary` is rendered fully.

## Database Schema Changes

### Migration: Add score column

```sql
ALTER TABLE setlist_entries ADD COLUMN score INTEGER;
```

- Column is nullable (no `NOT NULL` constraint)
- No default value — existing rows get `NULL`
- No new indexes needed (queries filter by `session_id`, not by `score`)
- Check constraint not added at DB level — validated in application code (core + API)

### Updated: setlist_entries table (post-migration)

```
setlist_entries
├── id TEXT PRIMARY KEY NOT NULL
├── session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE
├── item_id TEXT NOT NULL
├── item_title TEXT NOT NULL
├── item_type TEXT NOT NULL
├── position INTEGER NOT NULL
├── duration_secs INTEGER NOT NULL
├── status TEXT NOT NULL
├── notes TEXT
└── score INTEGER                        # NEW — nullable, values 1–5 or NULL
```

## Event Changes

### New: SessionEvent::UpdateEntryScore

```
UpdateEntryScore { entry_id: String, score: Option<u8> }
```

- Dispatched from the summary screen when user taps a score button
- Passing `Some(n)` sets the score; passing `None` clears it (toggle-to-deselect)
- Core validates: if `score.is_some()`, value must be 1–5; entry must have status `Completed`
- Only valid during `SessionStatus::Summary`

## Relationships

```
PracticeSession 1──* SetlistEntry (entries, ordered by position)
     │                    │
     │                    └── score: Option<u8> (NEW)
     │
Library Item ◄──── SetlistEntry.item_id (many entries reference one item)
     │
     └── ItemPracticeSummary (computed view)
              ├── latest_score (derived from most recent scored entry)
              └── score_history (derived from all scored entries)
```

## Backward Compatibility

- **Existing sessions in DB**: `score` column is nullable → existing rows have `NULL` → deserialized as `Option<u8> = None` → displayed as "not scored"
- **Existing sessions in localStorage** (crash recovery): `serde` deserialization of `ActiveSession` with missing `score` field defaults to `None` via `#[serde(default)]`
- **API responses**: Existing `GET /sessions` and `GET /sessions/{id}` return entries with `score: null` for pre-existing data — JSON clients handle this naturally
- **API requests**: `POST /sessions` accepts entries with or without `score` field — `serde` defaults missing field to `None`
