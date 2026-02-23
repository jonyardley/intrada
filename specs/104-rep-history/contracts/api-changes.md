# API Contract Changes: Rep History Tracking

**Feature**: 104-rep-history
**Date**: 2026-02-21

## Modified Endpoint: POST /sessions

### Request Body Change

The `entries` array gains a new optional field `rep_history`:

```json
{
  "started_at": "2026-02-21T10:00:00Z",
  "finished_at": "2026-02-21T10:30:00Z",
  "total_duration_secs": 1800,
  "status": "completed",
  "session_notes": null,
  "session_intention": "Focus on intonation",
  "entries": [
    {
      "id": "01JMXX...",
      "item_id": "01JMXX...",
      "item_title": "Bach Cello Suite No. 1",
      "item_type": "piece",
      "position": 0,
      "duration_secs": 600,
      "status": "completed",
      "notes": null,
      "score": 4,
      "intention": "Smooth bow changes",
      "rep_target": 5,
      "rep_count": 5,
      "rep_target_reached": true,
      "rep_history": [1, 1, -1, 1, 1, 1, 1, 1]
    },
    {
      "id": "01JMXY...",
      "item_id": "01JMXY...",
      "item_title": "Scales",
      "item_type": "exercise",
      "position": 1,
      "duration_secs": 300,
      "status": "completed",
      "notes": null,
      "score": null,
      "intention": null,
      "rep_target": null,
      "rep_count": null,
      "rep_target_reached": null,
      "rep_history": null
    }
  ]
}
```

### Validation Rules (400 Bad Request)

| Rule | Error Message |
|------|---------------|
| `rep_target` is `None` but `rep_history` is `Some` | `"rep_history requires rep_target"` |

Existing rules unchanged:
- `rep_count` cannot exceed `rep_target`
- `rep_count` and `rep_target_reached` require `rep_target`

### Response Body Change

`GET /sessions/:id` and `GET /sessions` responses include `rep_history` in each entry:

```json
{
  "rep_target": 5,
  "rep_count": 5,
  "rep_target_reached": true,
  "rep_history": [1, 1, -1, 1, 1, 1, 1, 1]
}
```

When no counter was active: all four fields are `null`.

### Backward Compatibility

- Existing sessions saved without `rep_history` return `null` for the field (SQLite column default).
- The `#[serde(default)]` attribute on the request struct ensures old clients that don't send `rep_history` work without errors.
