# API Contract Changes: Reusable Routines

**Feature**: 025-reusable-routines
**Date**: 2026-02-18

## Overview

Five new endpoints under `/api/routines` for full CRUD on routines. No modifications to existing endpoints. All endpoints handle routines as a parent entity with nested entries. The PUT endpoint uses full replacement semantics (replaces name and all entries in a single transaction).

## New Endpoints

### GET /api/routines

**Purpose**: List all routines with their entries.

**Response**: `200 OK`

```json
[
  {
    "id": "01J...",
    "name": "Morning Warm-up",
    "entries": [
      {
        "id": "01J...",
        "item_id": "01HABC...",
        "item_title": "Long Tones",
        "item_type": "exercise",
        "position": 0
      },
      {
        "id": "01J...",
        "item_id": "01HDEF...",
        "item_title": "C Major Scale",
        "item_type": "exercise",
        "position": 1
      }
    ],
    "created_at": "2026-02-18T10:00:00Z",
    "updated_at": "2026-02-18T10:00:00Z"
  }
]
```

**Notes**: Returns empty array `[]` if no routines exist. Entries are ordered by `position`.

---

### POST /api/routines

**Purpose**: Create a new routine.

**Request body** (`CreateRoutineRequest`):

```json
{
  "name": "Morning Warm-up",
  "entries": [
    {
      "item_id": "01HABC...",
      "item_title": "Long Tones",
      "item_type": "exercise"
    },
    {
      "item_id": "01HDEF...",
      "item_title": "C Major Scale",
      "item_type": "exercise"
    }
  ]
}
```

| Field | Type | Required | Constraints |
| ----- | ---- | -------- | ----------- |
| `name` | string | Yes | 1–200 characters |
| `entries` | array | Yes | At least 1 entry |
| `entries[].item_id` | string | Yes | Non-empty |
| `entries[].item_title` | string | Yes | Non-empty |
| `entries[].item_type` | string | Yes | "piece" or "exercise" |

**Response**: `201 Created`

```json
{
  "id": "01J...",
  "name": "Morning Warm-up",
  "entries": [
    {
      "id": "01J...",
      "item_id": "01HABC...",
      "item_title": "Long Tones",
      "item_type": "exercise",
      "position": 0
    },
    {
      "id": "01J...",
      "item_id": "01HDEF...",
      "item_title": "C Major Scale",
      "item_type": "exercise",
      "position": 1
    }
  ],
  "created_at": "2026-02-18T10:00:00Z",
  "updated_at": "2026-02-18T10:00:00Z"
}
```

**Notes**: Server generates IDs (ULID) for the routine and each entry. Server assigns positions (0-indexed) based on array order. Server sets `created_at` and `updated_at` to current time.

**Errors**:
- `400 Bad Request`: `"Routine name is required"` — name is empty
- `400 Bad Request`: `"Routine name must not exceed 200 characters"` — name too long
- `400 Bad Request`: `"Routine must have at least one entry"` — entries array empty

---

### GET /api/routines/:id

**Purpose**: Get a single routine with entries.

**Response**: `200 OK` — same structure as individual items in GET /api/routines.

**Errors**:
- `404 Not Found`: `"Routine not found"` — no routine with given ID

---

### PUT /api/routines/:id

**Purpose**: Update a routine (full replacement of name and entries).

**Request body** (`UpdateRoutineRequest`):

```json
{
  "name": "Updated Warm-up",
  "entries": [
    {
      "item_id": "01HDEF...",
      "item_title": "C Major Scale",
      "item_type": "exercise"
    },
    {
      "item_id": "01HGHI...",
      "item_title": "Clair de Lune",
      "item_type": "piece"
    }
  ]
}
```

| Field | Type | Required | Constraints |
| ----- | ---- | -------- | ----------- |
| `name` | string | Yes | 1–200 characters |
| `entries` | array | Yes | At least 1 entry |
| `entries[].item_id` | string | Yes | Non-empty |
| `entries[].item_title` | string | Yes | Non-empty |
| `entries[].item_type` | string | Yes | "piece" or "exercise" |

**Response**: `200 OK` — full routine with new entries and updated `updated_at`.

**Behaviour**: Deletes all existing entries for this routine, then inserts new entries from the request body. This is done in a transaction. Server generates new entry IDs and assigns positions based on array order. Server updates `updated_at` to current time. `created_at` is preserved.

**Errors**:
- `400 Bad Request`: Same validation as POST
- `404 Not Found`: `"Routine not found"` — no routine with given ID

---

### DELETE /api/routines/:id

**Purpose**: Delete a routine and all its entries.

**Response**: `200 OK`

```json
{
  "deleted": true
}
```

**Errors**:
- `404 Not Found`: `"Routine not found"` — no routine with given ID

**Notes**: Entries are deleted via ON DELETE CASCADE.

## Validation Rules (Server-Side)

Applied in route handlers:

1. `name` must not be empty (trimmed) → 400: `"Routine name is required"`
2. `name` must not exceed 200 characters → 400: `"Routine name must not exceed 200 characters"`
3. `entries` must not be empty → 400: `"Routine must have at least one entry"`
4. Each `entries[].item_type` must be "piece" or "exercise" → 400: `"Invalid item type"`

Validation constants (`MAX_ROUTINE_NAME = 200`) are imported from `intrada-core::validation`.

## Crux Event Contracts

### New Event: `Event::Routine(RoutineEvent::SaveBuildingAsRoutine { name })`

**Preconditions**:
- `session_status` must be `Building`
- `BuildingSession.entries` must be non-empty
- `name` must be 1–200 characters

**Effects**:
- Creates `Routine` in `model.routines`
- Emits `StorageEffect::SaveRoutine(routine)`
- Triggers re-render

**Error behaviour**: Sets `model.last_error` if validation fails. No state change on error.

### New Event: `Event::Routine(RoutineEvent::SaveSummaryAsRoutine { name })`

**Preconditions**:
- `session_status` must be `Summary`
- `SummarySession.entries` must be non-empty

**Effects**: Same as `SaveBuildingAsRoutine`.

### New Event: `Event::Routine(RoutineEvent::LoadRoutineIntoSetlist { routine_id })`

**Preconditions**:
- `session_status` must be `Building`
- Routine with `routine_id` must exist in `model.routines`

**Effects**:
- Creates new `SetlistEntry` objects from routine entries (fresh ULIDs)
- Appends to `BuildingSession.entries`
- Reindexes all positions
- Triggers re-render
- No storage effect (session not yet persisted)

**Error behaviour**: If routine not found, sets `model.last_error`. No state change on error.

### New Event: `Event::Routine(RoutineEvent::DeleteRoutine { id })`

**Effects**:
- Removes routine from `model.routines`
- Emits `StorageEffect::DeleteRoutine { id }`

### New Event: `Event::Routine(RoutineEvent::UpdateRoutine { id, name, entries })`

**Effects**:
- Validates name and entries
- Updates routine in `model.routines`
- Emits `StorageEffect::UpdateRoutine(routine)`

### New Event: `Event::RoutinesLoaded { routines }`

**Effects**:
- Sets `model.routines = routines`
- Triggers re-render
