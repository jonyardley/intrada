# API Contracts: Tempo Tracking

**Feature**: 105-tempo-tracking
**Date**: 2026-02-24

## Overview

Tempo tracking extends existing session endpoints. No new endpoints are introduced. The `achieved_tempo` field is added to session entry payloads in both request and response.

## Modified Endpoints

### POST /api/sessions — Save a completed session

**Change**: `entries[].achieved_tempo` field added to request body.

#### Request Body (entry-level addition)

```json
{
  "entries": [
    {
      "id": "entry-001",
      "item_id": "item-abc",
      "item_title": "Clair de Lune",
      "item_type": "piece",
      "position": 0,
      "duration_secs": 600,
      "status": "Completed",
      "notes": null,
      "score": 4,
      "intention": "Focus on dynamics",
      "rep_target": null,
      "rep_count": null,
      "rep_target_reached": null,
      "rep_history": null,
      "planned_duration_secs": 600,
      "achieved_tempo": 108
    }
  ],
  "session_notes": "Good session",
  "started_at": "2026-02-24T10:00:00Z",
  "completed_at": "2026-02-24T10:30:00Z",
  "total_duration_secs": 1800,
  "completion_status": "Completed",
  "session_intention": "Warm up and run through repertoire"
}
```

#### Validation Rules (entry-level)

| Field | Type | Required | Validation |
|-------|------|----------|------------|
| `achieved_tempo` | integer \| null | No | If present: 1 ≤ value ≤ 500. Only valid when `status` = "Completed". |

#### Error Response (validation failure)

```json
{
  "error": "Achieved tempo must be between 1 and 500"
}
```

Status: `400 Bad Request`

### GET /api/sessions — List all sessions

**Change**: `entries[].achieved_tempo` field included in response.

#### Response Body (entry-level addition)

```json
{
  "entries": [
    {
      "id": "entry-001",
      "item_id": "item-abc",
      "item_title": "Clair de Lune",
      "item_type": "piece",
      "position": 0,
      "duration_secs": 600,
      "status": "Completed",
      "notes": null,
      "score": 4,
      "intention": "Focus on dynamics",
      "rep_target": null,
      "rep_count": null,
      "rep_target_reached": null,
      "rep_history": null,
      "planned_duration_secs": 600,
      "achieved_tempo": 108
    }
  ]
}
```

### GET /api/sessions/{id} — Get a specific session

**Change**: Same as GET /api/sessions — `achieved_tempo` included per entry.

### DELETE /api/sessions/{id} — Delete a session

**Change**: No payload change. Cascade delete removes all entry data including `achieved_tempo`.

## Backward Compatibility

- **Existing clients sending requests without `achieved_tempo`**: The field is `Option<u16>`, deserialized with `#[serde(default)]`. Missing field → `None`. No breaking change.
- **Existing sessions in database**: `NULL` in the `achieved_tempo` column → `None` in Rust. No migration of existing data needed.
- **Existing sessions in response**: `achieved_tempo: null` for old sessions. Clients that don't know about the field will ignore it (standard JSON behavior).

## Database Column

| Column | Table | Type | Nullable | Default |
|--------|-------|------|----------|---------|
| `achieved_tempo` | `setlist_entries` | INTEGER | Yes | NULL |

Positional index in `ENTRY_COLUMNS`: 15 (0-based).
