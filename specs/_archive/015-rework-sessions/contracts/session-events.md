# Session Event Contracts

**Feature**: 015-rework-sessions
**Date**: 2026-02-15

This document defines the Crux events and storage effects that drive the setlist-based session model. These replace the existing `SessionEvent` enum.

---

## Events (replace `SessionEvent`)

### SessionEvent Enum

```rust
pub enum SessionEvent {
    // === Building Phase ===
    StartBuilding,
    AddToSetlist { item_id: String },
    AddNewItemToSetlist { title: String, item_type: String },
    RemoveFromSetlist { entry_id: String },
    ReorderSetlist { entry_id: String, new_position: usize },
    StartSession,
    CancelBuilding,

    // === Active Phase ===
    NextItem { now: DateTime<Utc> },
    SkipItem { now: DateTime<Utc> },
    AddItemMidSession { item_id: String },
    AddNewItemMidSession { title: String, item_type: String },
    FinishSession { now: DateTime<Utc> },
    EndSessionEarly { now: DateTime<Utc> },

    // === Summary Phase ===
    UpdateEntryNotes { entry_id: String, notes: Option<String> },
    UpdateSessionNotes { notes: Option<String> },
    SaveSession { now: DateTime<Utc> },
    DiscardSession,

    // === Recovery ===
    RecoverSession { session: ActiveSession },

    // === History ===
    DeleteSession { id: String },
}
```

---

## Event Contracts

### StartBuilding

**Precondition**: `session_status == Idle`
**Postcondition**: `session_status == Building`, empty entries list
**Effects**: Render
**Error**: If session already in progress → `last_error` set

### AddToSetlist { item_id }

**Precondition**: `session_status == Building`, item exists in library
**Postcondition**: New `SetlistEntry` appended with snapshot of item title/type, position = entries.len()
**Effects**: Render
**Error**: If item_id not found → `last_error` set

### AddNewItemToSetlist { title, item_type }

**Precondition**: `session_status == Building`
**Postcondition**: New library item created (Piece or Exercise with title only), new `SetlistEntry` appended
**Effects**: StorageEffect::SavePiece/SaveExercise + Render
**Error**: Validation failure → `last_error` set

### RemoveFromSetlist { entry_id }

**Precondition**: `session_status == Building`, entry exists
**Postcondition**: Entry removed, remaining entries re-indexed
**Effects**: Render
**Error**: If entry not found → `last_error` set

### ReorderSetlist { entry_id, new_position }

**Precondition**: `session_status == Building`, entry exists, new_position valid
**Postcondition**: Entry moved to new_position, other entries re-indexed
**Effects**: Render
**Error**: Invalid position or entry not found → `last_error` set

### StartSession

**Precondition**: `session_status == Building`, entries.len() >= 1
**Postcondition**: `session_status == Active`, current_index = 0, session_started_at = now, current_item_started_at = now
**Effects**: StorageEffect::SaveSessionInProgress + Render
**Error**: Empty setlist → `last_error` set (FR-004)

### CancelBuilding

**Precondition**: `session_status == Building`
**Postcondition**: `session_status == Idle`, building state cleared
**Effects**: Render

### NextItem { now }

**Precondition**: `session_status == Active`, current_index < entries.len() - 1 (not last item)
**Postcondition**: Current entry gets duration_secs = (now - current_item_started_at).as_secs(), status = Completed. current_index += 1, current_item_started_at = now
**Effects**: StorageEffect::SaveSessionInProgress + Render

### SkipItem { now }

**Precondition**: `session_status == Active`
**Postcondition**: Current entry gets duration_secs = 0, status = Skipped. If not last item: current_index += 1, current_item_started_at = now. If last item: transitions to Summary.
**Effects**: StorageEffect::SaveSessionInProgress + Render

### AddItemMidSession { item_id }

**Precondition**: `session_status == Active`, item exists in library
**Postcondition**: New `SetlistEntry` appended to end of entries with status NotAttempted. Timer for current item NOT interrupted (SC-004).
**Effects**: StorageEffect::SaveSessionInProgress + Render
**Error**: Item not found → `last_error` set

### AddNewItemMidSession { title, item_type }

**Precondition**: `session_status == Active`
**Postcondition**: New library item created, new `SetlistEntry` appended. Timer NOT interrupted.
**Effects**: StorageEffect::SavePiece/SaveExercise + StorageEffect::SaveSessionInProgress + Render
**Error**: Validation failure → `last_error` set

### FinishSession { now }

**Precondition**: `session_status == Active`, current_index == entries.len() - 1 (last item)
**Postcondition**: Last entry gets duration_secs and status = Completed. `session_status == Summary`.
**Effects**: StorageEffect::SaveSessionInProgress + Render

### EndSessionEarly { now }

**Precondition**: `session_status == Active`
**Postcondition**: Current entry gets duration_secs up to now, status = Completed. All entries after current_index get status = NotAttempted, duration_secs = 0. `session_status == Summary`, completion_status = EndedEarly.
**Effects**: StorageEffect::SaveSessionInProgress + Render

### UpdateEntryNotes { entry_id, notes }

**Precondition**: `session_status == Summary`, entry exists
**Postcondition**: Entry notes updated
**Effects**: Render
**Error**: Notes too long (>5000) or entry not found → `last_error` set

### UpdateSessionNotes { notes }

**Precondition**: `session_status == Summary`
**Postcondition**: Session notes updated
**Effects**: Render
**Error**: Notes too long (>5000) → `last_error` set

### SaveSession { now }

**Precondition**: `session_status == Summary`
**Postcondition**: `PracticeSession` constructed and appended to model.sessions. `session_status == Idle`. In-progress key cleared.
**Effects**: StorageEffect::SavePracticeSession + StorageEffect::ClearSessionInProgress + Render

### DiscardSession

**Precondition**: `session_status == Summary`
**Postcondition**: `session_status == Idle`. Summary state discarded. In-progress key cleared.
**Effects**: StorageEffect::ClearSessionInProgress + Render

### RecoverSession { session }

**Precondition**: `session_status == Idle`
**Postcondition**: `session_status == Active`, model populated from recovered ActiveSession
**Effects**: Render

### DeleteSession { id }

**Precondition**: Session with given id exists in model.sessions
**Postcondition**: Session removed from model.sessions
**Effects**: StorageEffect::DeletePracticeSession { id } + Render
**Error**: Not found → `last_error` set

---

## Storage Effects (replace session-related variants)

### Updated StorageEffect Enum

```rust
pub enum StorageEffect {
    // Library operations (UNCHANGED)
    LoadAll,
    SavePiece(Piece),
    SaveExercise(Exercise),
    UpdatePiece(Piece),
    UpdateExercise(Exercise),
    DeleteItem { id: String },

    // Session operations (REPLACED)
    LoadSessions,                          // Same name, loads Vec<PracticeSession>
    SavePracticeSession(PracticeSession),   // Replaces SaveSession(Session)
    DeletePracticeSession { id: String },   // Replaces DeleteSession { id }

    // In-progress session (NEW)
    SaveSessionInProgress(ActiveSession),   // Persist to intrada:session-in-progress
    ClearSessionInProgress,                 // Remove intrada:session-in-progress
    LoadSessionInProgress,                  // Load from intrada:session-in-progress
}
```

### Removed Variants

- `SaveSession(Session)` → replaced by `SavePracticeSession(PracticeSession)`
- `UpdateSession(Session)` → removed (sessions are immutable after save)
- `DeleteSession { id }` → replaced by `DeletePracticeSession { id }`

### Shell Handling Notes

- `LoadSessions`: Shell reads `intrada:sessions`, detects old schema (objects with `item_id` + `duration_minutes` fields), wipes and replaces with empty new-schema data. Emits `Event::SessionsLoaded { sessions }` with `Vec<PracticeSession>`.
- `SaveSessionInProgress`: Shell serializes `ActiveSession` to `intrada:session-in-progress`.
- `LoadSessionInProgress`: Shell reads `intrada:session-in-progress`, if present emits `Event::Session(SessionEvent::RecoverSession { session })`.
- `ClearSessionInProgress`: Shell removes `intrada:session-in-progress` key.

---

## Updated Root Events

```rust
pub enum Event {
    Piece(PieceEvent),           // Unchanged
    Exercise(ExerciseEvent),     // Unchanged
    Session(SessionEvent),       // Updated enum above
    DataLoaded { ... },          // Unchanged
    SessionsLoaded {
        sessions: Vec<PracticeSession>,  // Changed from Vec<Session>
    },
    LoadFailed(String),          // Unchanged
    ClearError,                  // Unchanged
    SetQuery(Option<ListQuery>), // Unchanged
}
```
