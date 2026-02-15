# Data Model: Rework Sessions (Setlist Model)

**Feature**: 015-rework-sessions
**Date**: 2026-02-15

---

## Entities

### PracticeSession (replaces `Session`)

The top-level record for a completed practice session.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | `String` (ULID) | Yes | Unique identifier |
| `entries` | `Vec<SetlistEntry>` | Yes | Ordered list of items practised (min 1) |
| `session_notes` | `Option<String>` | No | Overall session notes (max 5,000 chars) |
| `started_at` | `DateTime<Utc>` | Yes | When the session timer first started |
| `completed_at` | `DateTime<Utc>` | Yes | When the session was saved from summary |
| `total_duration_secs` | `u64` | Yes | Sum of all entry durations in seconds |
| `completion_status` | `CompletionStatus` | Yes | Whether all items were completed or session ended early |

**Validation rules**:
- `entries` must contain at least 1 entry
- `session_notes` max 5,000 characters (reuses `MAX_NOTES`)
- `started_at` must be before `completed_at`
- `total_duration_secs` must equal sum of all entry `duration_secs`

**Relationships**:
- Contains 1..* `SetlistEntry` (ordered, owned)

---

### SetlistEntry

An individual item within a session's setlist.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | `String` (ULID) | Yes | Unique entry identifier |
| `item_id` | `String` | Yes | Reference to library item (piece/exercise) |
| `item_title` | `String` | Yes | Snapshot of item title at session time |
| `item_type` | `String` | Yes | Snapshot of item type ("piece" or "exercise") |
| `position` | `usize` | Yes | Order in setlist (0-indexed) |
| `duration_secs` | `u64` | Yes | Time spent practising in seconds |
| `status` | `EntryStatus` | Yes | Completion status of this entry |
| `notes` | `Option<String>` | No | Per-item practice notes (max 5,000 chars) |

**Validation rules**:
- `item_id` must not be empty
- `item_title` must not be empty, max 500 characters (reuses `MAX_TITLE`)
- `item_type` must be "piece" or "exercise"
- `notes` max 5,000 characters (reuses `MAX_NOTES`)
- `duration_secs` is 0 for skipped and not-attempted entries

**Relationships**:
- Belongs to 1 `PracticeSession`
- References 1 library item (Piece or Exercise) by `item_id`

---

### EntryStatus (enum)

| Variant | Description |
|---------|-------------|
| `Completed` | Item was practised and time was recorded |
| `Skipped` | Item was explicitly skipped (duration_secs = 0) |
| `NotAttempted` | Session ended early before reaching this item (duration_secs = 0) |

---

### CompletionStatus (enum)

| Variant | Description |
|---------|-------------|
| `Completed` | All items in the setlist were addressed (completed or skipped) |
| `EndedEarly` | Session was ended before all items were reached |

---

### SessionStatus (enum, transient — not persisted)

Models the lifecycle of a session in the core `Model`. This is internal state, not stored in localStorage.

| Variant | Data | Description |
|---------|------|-------------|
| `Idle` | — | No session in progress |
| `Building` | `BuildingSession` | Assembling the setlist before starting |
| `Active` | `ActiveSession` | Timer running, practising items |
| `Summary` | `SummarySession` | Session finished, reviewing results |

---

### BuildingSession (transient)

State during setlist assembly.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `entries` | `Vec<SetlistEntry>` | Yes | Items added so far (may be empty initially) |

---

### ActiveSession (transient, persisted to recovery key)

State during active practice. Persisted to `intrada:session-in-progress` for crash recovery.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | `String` (ULID) | Yes | Session ID (assigned when practice starts) |
| `entries` | `Vec<SetlistEntry>` | Yes | All entries with accumulated times |
| `current_index` | `usize` | Yes | Index of the currently active entry |
| `current_item_started_at` | `DateTime<Utc>` | Yes | When the current item's timer started |
| `session_started_at` | `DateTime<Utc>` | Yes | When the session was first started |

**Relationships**:
- `entries[current_index]` is the active entry
- Entries before `current_index` are completed or skipped
- Entries after `current_index` are pending

---

### SummarySession (transient)

State during post-session review.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | `String` (ULID) | Yes | Session ID |
| `entries` | `Vec<SetlistEntry>` | Yes | All entries with final times and statuses |
| `session_started_at` | `DateTime<Utc>` | Yes | When the session was started |
| `session_ended_at` | `DateTime<Utc>` | Yes | When the session transitioned to summary |
| `session_notes` | `Option<String>` | No | Overall notes (editable in summary) |

---

## Top-Level Storage Types

### SessionsData (replaces existing `SessionsData`)

```
SessionsData {
    sessions: Vec<PracticeSession>   // Completed sessions only
}
```

**Storage key**: `intrada:sessions` (same key, new schema)

### ActiveSession Recovery

```
Option<ActiveSession>
```

**Storage key**: `intrada:session-in-progress` (new key)

---

## State Transitions

```
Idle
  ──[StartBuilding]──→ Building
  ──[RecoverSession]──→ Active  (from intrada:session-in-progress)

Building
  ──[AddToSetlist(item)]──→ Building  (entry appended)
  ──[RemoveFromSetlist(entry_id)]──→ Building  (entry removed)
  ──[ReorderSetlist(entry_id, new_position)]──→ Building  (entry moved)
  ──[StartSession]──→ Active  (requires entries.len() >= 1)
  ──[CancelBuilding]──→ Idle

Active
  ──[NextItem]──→ Active  (records time, advances current_index)
  ──[SkipItem]──→ Active  (marks skipped, advances current_index)
  ──[AddItemMidSession(item)]──→ Active  (appends to entries)
  ──[FinishSession]──→ Summary  (from last item; records time)
  ──[EndSessionEarly]──→ Summary  (records current item time, marks remaining as NotAttempted)
  ──[NextItem on last item]──→ Summary  (equivalent to FinishSession)

Summary
  ──[UpdateEntryNotes(entry_id, notes)]──→ Summary
  ──[UpdateSessionNotes(notes)]──→ Summary
  ──[SaveSession]──→ Idle  (persists PracticeSession, clears in-progress)
  ──[DiscardSession]──→ Idle  (clears in-progress, does not persist)
```

---

## View Models (computed in `view()`)

### PracticeSessionView (replaces `SessionView`)

For the session history list.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | Session ID |
| `started_at` | `String` | ISO 8601 formatted |
| `total_duration_secs` | `u64` | Total practice time |
| `item_count` | `usize` | Number of entries |
| `items_practised` | `usize` | Entries with status Completed |
| `items_skipped` | `usize` | Entries with status Skipped |
| `completion_status` | `String` | "completed" or "ended_early" |
| `session_notes` | `Option<String>` | Overall session notes |
| `entries` | `Vec<SetlistEntryView>` | All entries for detail view |

### SetlistEntryView

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | Entry ID |
| `item_id` | `String` | Library item reference |
| `item_title` | `String` | Snapshot title |
| `item_type` | `String` | "piece" or "exercise" |
| `position` | `usize` | Order in setlist |
| `duration_secs` | `u64` | Time spent |
| `duration_display` | `String` | Human-readable (e.g., "5m 30s") |
| `status` | `String` | "completed", "skipped", or "not_attempted" |
| `notes` | `Option<String>` | Per-item notes |

### ActiveSessionView (new, optional in ViewModel)

For the in-session practice UI.

| Field | Type | Description |
|-------|------|-------------|
| `current_item_title` | `String` | Active item name |
| `current_item_type` | `String` | "piece" or "exercise" |
| `current_position` | `usize` | 0-indexed position |
| `total_items` | `usize` | Total setlist length |
| `progress_label` | `String` | e.g., "3 of 7" |
| `completed_entries` | `Vec<SetlistEntryView>` | Already-completed items |
| `upcoming_entries` | `Vec<SetlistEntryView>` | Remaining items |

### ItemPracticeSummary (existing, updated computation)

Unchanged structure, but `compute_practice_summary()` now aggregates from `PracticeSession.entries` where `entry.item_id == target_id` and `entry.status == Completed`.

| Field | Type | Description |
|-------|------|-------------|
| `session_count` | `usize` | Number of sessions where this item was practised |
| `total_minutes` | `u32` | Total practice time in minutes (rounded) |

---

## Updated ViewModel Shape

```rust
ViewModel {
    items: Vec<LibraryItemView>,           // Unchanged
    sessions: Vec<PracticeSessionView>,    // Replaces Vec<SessionView>
    active_session: Option<ActiveSessionView>,  // New: in-progress session
    session_status: String,                // "idle", "building", "active", "summary"
    building_setlist: Option<Vec<SetlistEntryView>>,  // New: building phase entries
    summary: Option<SummaryView>,          // New: summary phase data
    error: Option<String>,                 // Unchanged
}
```

### SummaryView

| Field | Type | Description |
|-------|------|-------------|
| `total_duration_secs` | `u64` | Total practice time |
| `total_duration_display` | `String` | Human-readable total |
| `entries` | `Vec<SetlistEntryView>` | All entries with final data |
| `session_notes` | `Option<String>` | Overall notes |
| `items_completed` | `usize` | Count of completed entries |
| `items_skipped` | `usize` | Count of skipped entries |
| `items_not_attempted` | `usize` | Count of not-attempted entries |
| `completion_status` | `String` | "completed" or "ended_early" |
