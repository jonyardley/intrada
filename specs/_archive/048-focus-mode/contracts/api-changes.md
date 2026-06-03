# API Contract Changes: Focus Mode

**Date**: 2026-02-23
**Feature**: 048-focus-mode

## Overview

Focus mode is primarily a frontend feature. The only API change is adding
`planned_duration_secs` to the session entry payload so that planned durations
persist across devices and sessions.

## Modified Endpoints

### POST /api/sessions — Save Session

**Change**: `entries[].planned_duration_secs` field added to request body.

**Request body change** (entries array element):

```json
{
  "id": "01HXYZ...",
  "item_id": "01HABC...",
  "item_title": "Db Major Scale",
  "item_type": "exercise",
  "position": 0,
  "duration_secs": 312,
  "status": "completed",
  "notes": null,
  "score": 4,
  "intention": "Focus on evenness",
  "rep_target": 5,
  "rep_count": 5,
  "rep_target_reached": true,
  "rep_history": [{"action": "hit"}, {"action": "hit"}, {"action": "hit"}, {"action": "hit"}, {"action": "hit"}],
  "planned_duration_secs": 300
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `planned_duration_secs` | `integer \| null` | No | Planned duration in seconds. `null` or absent means no planned duration was set. Range: 60–3600. |

**Response**: No change — responses already include all entry fields.

**Backwards compatibility**: The field is optional (nullable). Existing clients that
don't send it will have `NULL` stored. Existing sessions retrieved from the API
will have `null` for this field.

### GET /api/sessions — List Sessions

**Change**: Response entries include `planned_duration_secs`.

Each entry in the response gains:

```json
{
  "planned_duration_secs": 300
}
```

Or `null` for entries without a planned duration.

### GET /api/sessions/:id — Get Session

Same change as List Sessions — entries include `planned_duration_secs`.

## No New Endpoints

Focus mode does not require any new API endpoints. The toggle state (focused vs
expanded) is ephemeral UI state and is not persisted.
