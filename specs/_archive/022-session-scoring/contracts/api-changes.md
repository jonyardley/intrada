# API Contract Changes: Session Item Scoring

**Feature**: 022-session-scoring
**Date**: 2026-02-17

## Overview

No new endpoints. The existing session endpoints are extended to include the optional `score` field on setlist entries. All changes are backward compatible — the `score` field is optional in both requests and responses.

## Modified Endpoints

### POST /api/sessions

**Change**: `entries[].score` field added to request body.

**Request body** (updated `SaveSessionRequest`):

```json
{
  "entries": [
    {
      "id": "01HXYZ...",
      "item_id": "01HABC...",
      "item_title": "Clair de Lune",
      "item_type": "piece",
      "position": 0,
      "duration_secs": 300,
      "status": "Completed",
      "notes": "Good tempo control",
      "score": 4
    },
    {
      "id": "01HXYZ...",
      "item_id": "01HDEF...",
      "item_title": "C Major Scale",
      "item_type": "exercise",
      "position": 1,
      "duration_secs": 0,
      "status": "Skipped",
      "notes": null,
      "score": null
    }
  ],
  "session_notes": "Great session overall",
  "started_at": "2026-02-17T10:00:00Z",
  "completed_at": "2026-02-17T10:35:00Z",
  "total_duration_secs": 300,
  "completion_status": "Completed"
}
```

**Field: `entries[].score`**
- Type: `integer | null`
- Required: No (defaults to `null` if omitted)
- Constraints: When present, must be 1–5 inclusive
- Validation error: 400 Bad Request with message `"Score must be between 1 and 5"`

**Response**: Unchanged structure — `201 Created` with full `PracticeSession` including entries with `score` field.

---

### GET /api/sessions

**Change**: Response entries now include `score` field.

**Response body** (entries within each session):

```json
[
  {
    "id": "01HSESS...",
    "entries": [
      {
        "id": "01HENTRY...",
        "item_id": "01HABC...",
        "item_title": "Clair de Lune",
        "item_type": "piece",
        "position": 0,
        "duration_secs": 300,
        "status": "Completed",
        "notes": "Good tempo control",
        "score": 4
      }
    ],
    "session_notes": "Great session",
    "started_at": "2026-02-17T10:00:00Z",
    "completed_at": "2026-02-17T10:35:00Z",
    "total_duration_secs": 300,
    "completion_status": "Completed"
  }
]
```

**Field: `entries[].score`**
- Type: `integer | null`
- `null` for entries that were not scored, or for entries from sessions saved before this feature

---

### GET /api/sessions/{id}

**Change**: Same as GET /api/sessions — entries include `score` field.

---

### DELETE /api/sessions/{id}

**Change**: None. Cascade delete removes entries including their scores.

## Validation Rules (Server-Side)

Applied in the `save_session` route handler:

1. If `entry.score` is present (not null):
   - Value must be >= 1 and <= 5
   - If out of range: return `400 Bad Request` with `"Score must be between 1 and 5"`
2. If `entry.score` is null or absent: accepted (no score recorded)
3. No validation that score is only present on "Completed" entries at the API level — the core enforces this constraint during the session summary flow. The API stores whatever valid (1–5 or null) score is provided.

## Crux Event Contract

### New Event: `SessionEvent::UpdateEntryScore`

```
Event::Session(SessionEvent::UpdateEntryScore {
    entry_id: String,
    score: Option<u8>,
})
```

**Preconditions**:
- `session_status` must be `Summary`
- Entry with `entry_id` must exist in the summary's entries
- Entry must have `status == Completed`
- If `score` is `Some(n)`, then `n` must be 1–5

**Effects**:
- Updates the entry's `score` field in the `SummarySession`
- Triggers a re-render (existing render effect)

**Error behaviour**:
- If preconditions fail: no state change, no error surfaced (silent no-op, consistent with existing event handling pattern)

### Modified Effect: `StorageEffect::SavePracticeSession`

No structural change. The `PracticeSession` payload now includes entries with `score: Option<u8>`. The shell serializes this as part of the existing JSON body sent to `POST /api/sessions`.
