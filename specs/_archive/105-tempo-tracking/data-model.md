# Data Model: Tempo Tracking

**Feature**: 105-tempo-tracking
**Date**: 2026-02-24

## Entities

### Achieved Tempo (new field on existing SetlistEntry)

An optional BPM value recorded per session entry during the summary phase.

| Field | Type | Constraints | Notes |
|-------|------|-------------|-------|
| `achieved_tempo` | `Option<u16>` | 1–500 inclusive, or None | Only valid when entry status is `Completed` |

**Belongs to**: `SetlistEntry` (existing entity)
**Relationship**: Each `SetlistEntry` has zero or one achieved tempo. Multiple entries in the same session for the same item may have different achieved tempos.

### TempoHistoryEntry (new struct)

A single tempo data point for an item's progress history. Mirrors `ScoreHistoryEntry`.

| Field | Type | Constraints | Notes |
|-------|------|-------------|-------|
| `session_date` | `String` | RFC 3339 format | From `PracticeSession.started_at` |
| `tempo` | `u16` | 1–500 | The achieved BPM value |
| `session_id` | `String` | Valid session ID | Reference to source session |

**Derived from**: Aggregation of `SetlistEntry.achieved_tempo` across all sessions for a given `item_id`.
**Ordering**: Most recent first (descending by `session_date`).

### ItemPracticeSummary (extended)

Two new fields added to the existing struct.

| Field | Type | Constraints | Notes |
|-------|------|-------------|-------|
| `latest_tempo` | `Option<u16>` | 1–500 or None | Most recent achieved tempo for the item |
| `tempo_history` | `Vec<TempoHistoryEntry>` | Ordered descending by date | All recorded tempos for the item |

### LibraryItemView (extended)

One new field for the library list display.

| Field | Type | Notes |
|-------|------|-------|
| `latest_achieved_tempo` | `Option<u16>` | From `ItemPracticeSummary.latest_tempo`; shown alongside target tempo in library list |

### SetlistEntryView (extended)

One new field for the summary and history views.

| Field | Type | Notes |
|-------|------|-------|
| `achieved_tempo` | `Option<u16>` | Displayed in session summary and session history |

## Validation Rules

### Achieved Tempo Validation

| Rule | Constant | Value |
|------|----------|-------|
| Minimum achieved tempo | `MIN_ACHIEVED_TEMPO` | 1 |
| Maximum achieved tempo | `MAX_ACHIEVED_TEMPO` | 500 |

**Validation function**: `validate_achieved_tempo(tempo: &Option<u16>) -> Result<(), LibraryError>`
- `None` → valid (tempo is optional)
- `Some(v)` where `v` in `MIN_ACHIEVED_TEMPO..=MAX_ACHIEVED_TEMPO` → valid
- `Some(v)` outside range → `LibraryError::Validation { field: "achieved_tempo", message: "..." }`

**Validation points**:
1. Core: `validate_achieved_tempo()` in `validation.rs` (single source of truth)
2. API: Called during `save_session` request validation
3. Web: Client-side validation in session summary form

### Entry Status Gating

Achieved tempo is only valid for `EntryStatus::Completed` entries. The `UpdateEntryTempo` event is only processed when the entry's status is `Completed` (same pattern as `UpdateEntryScore`).

## Database Changes

### Migration: Add achieved_tempo column

```sql
ALTER TABLE setlist_entries ADD COLUMN achieved_tempo INTEGER
```

- Column position: After `planned_duration_secs` (last column)
- Nullable: Yes (existing rows default to NULL)
- Type: INTEGER (maps to `Option<u16>` in Rust via `Option<i64>` read + cast)

### Updated column list

`ENTRY_COLUMNS` const in `db/sessions.rs` adds `achieved_tempo` at the end:

```
id, item_id, item_title, item_type, position, duration_secs, status, notes, score, intention, rep_target, rep_count, rep_target_reached, rep_history, planned_duration_secs, achieved_tempo
```

Column index: 15 (0-based)

## State Transitions

### Session Summary Phase

```
Entry completed → Summary phase begins
  → Musician optionally enters achieved tempo (numeric input)
  → SessionEvent::UpdateEntryTempo { entry_id, tempo: Option<u16> }
  → Core validates range and entry status
  → Model updated (entry.achieved_tempo = tempo)
  → Musician saves session
  → SessionEvent::SaveSession { now }
  → PracticeSession built with achieved_tempo on each entry
  → Effect::SavePracticeSession emitted
  → Shell sends to API
  → API validates and persists
  → Sessions re-fetched
  → practice_summaries cache rebuilt (includes tempo history)
```

### Tempo History Aggregation

```
Sessions loaded/saved/deleted
  → build_practice_summaries() iterates all sessions
  → For each entry with achieved_tempo:
    → TempoHistoryEntry added to item's accumulator
  → History sorted descending by date
  → latest_tempo = first entry's tempo (or None)
  → Stored in Model.practice_summaries HashMap
```
