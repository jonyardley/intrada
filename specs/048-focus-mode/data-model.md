# Data Model: Focus Mode

**Date**: 2026-02-23
**Feature**: 048-focus-mode

## Modified Entities

### SetlistEntry (Core Domain)

**File**: `intrada-core/src/domain/session.rs`

| Field | Type | Change | Description |
|-------|------|--------|-------------|
| `planned_duration_secs` | `Option<u32>` | **NEW** | Optional planned duration in seconds. Set during building phase. `None` means no planned duration (timer counts up without progress ring). |

All existing fields remain unchanged.

### SetlistEntryView (View Model)

**File**: `intrada-core/src/model.rs`

| Field | Type | Change | Description |
|-------|------|--------|-------------|
| `planned_duration_secs` | `Option<u32>` | **NEW** | Raw planned duration for the shell to use in progress calculations. |
| `planned_duration_display` | `Option<String>` | **NEW** | Human-readable display (e.g., "5 min"). Used in builder and completed items list. |

### ActiveSessionView (View Model)

**File**: `intrada-core/src/model.rs`

| Field | Type | Change | Description |
|-------|------|--------|-------------|
| `current_planned_duration_secs` | `Option<u32>` | **NEW** | Current item's planned duration. Shortcut so the shell doesn't need to look up the current entry. |
| `next_item_title` | `Option<String>` | **NEW** | Title of the next item in the session. `None` when on the last item. Used by the transition prompt. |

### SaveSessionEntry (API Request/Response)

**File**: `intrada-api/src/routes/sessions.rs`, `intrada-api/src/db/sessions.rs`

| Field | Type | Change | Description |
|-------|------|--------|-------------|
| `planned_duration_secs` | `Option<u32>` | **NEW** | Persisted to DB. Nullable — backwards compatible with existing sessions. |

## New Events

### SessionEvent::SetEntryDuration

**File**: `intrada-core/src/domain/session.rs`

```
SetEntryDuration { entry_id: String, duration_secs: Option<u32> }
```

Dispatched from the session builder when the musician sets or clears a planned duration for a setlist entry. Sets `entry.planned_duration_secs`.

## Database Schema Change

### Migration 0025: Add planned_duration_secs

```sql
ALTER TABLE setlist_entries ADD COLUMN planned_duration_secs INTEGER;
```

- Nullable integer column
- Backwards compatible — existing rows get `NULL`
- Position: after `rep_history` (column index 15)

## State Boundary

| State | Location | Type |
|-------|----------|------|
| `planned_duration_secs` | Crux `Model` → `ViewModel` | Domain data — persisted, survives crash recovery |
| Focus mode toggle (expanded/focused) | Leptos `RwSignal<bool>` | UI interaction — ephemeral, resets to focused on mount |
| Duration elapsed flag | Leptos `RwSignal<bool>` | UI interaction — computed from elapsed vs planned, resets per item |
| Timer elapsed seconds | Leptos `RwSignal<u32>` | UI interaction — already exists, drives progress ring calculation |

## Validation Rules

**File**: `intrada-core/src/validation.rs`

| Constant | Value | Description |
|----------|-------|-------------|
| `MIN_PLANNED_DURATION_SECS` | `60` | Minimum 1 minute |
| `MAX_PLANNED_DURATION_SECS` | `3600` | Maximum 60 minutes |
