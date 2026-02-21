# API Contract Changes: Repetition Counter

**Branch**: `103-repetition-counter` | **Date**: 2026-02-21

## Summary

No new API endpoints are required. The existing `POST /api/sessions` and `GET /api/sessions` endpoints carry the new repetition fields transparently via extended structs. All new fields use `#[serde(default)]` for backward compatibility.

## Modified Endpoints

### POST /api/sessions

**Change**: `SaveSessionEntry` struct gains 3 new optional fields.

#### Request Body (extended fields only)

```json
{
  "entries": [
    {
      "item_id": "...",
      "item_title": "...",
      "item_type": "piece",
      "position": 0,
      "duration_secs": 120,
      "status": "completed",
      "notes": null,
      "score": null,
      "intention": null,
      "rep_target": 5,
      "rep_count": 5,
      "rep_target_reached": true
    }
  ]
}
```

| New Field | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `rep_target` | `integer \| null` | No | `null` | Configured repetition target (3–10). `null` = counter not used. |
| `rep_count` | `integer \| null` | No | `null` | Final consecutive correct count at save time. `null` = counter not used. |
| `rep_target_reached` | `boolean \| null` | No | `null` | Whether target was reached. `null` = counter not used. |

#### Validation Rules

- If `rep_target` is present, it must be in range `3..=10`
- If `rep_target` is `null`, `rep_count` and `rep_target_reached` should also be `null`
- If `rep_count` is present, it must be in range `0..=rep_target`

### GET /api/sessions

**Change**: Response includes new fields on each entry.

#### Response Body (extended fields only)

```json
{
  "sessions": [
    {
      "entries": [
        {
          "rep_target": 5,
          "rep_count": 5,
          "rep_target_reached": true
        }
      ]
    }
  ]
}
```

Fields follow the same schema as the request body. Existing sessions return `null` for all three fields.

### GET /api/sessions/:id

**Change**: Same as `GET /api/sessions` — entries include the new fields.

## Backward Compatibility

- Clients that don't send `rep_target`, `rep_count`, or `rep_target_reached` will have them default to `null` via `#[serde(default)]`
- Existing saved sessions return `null` for all three fields (database columns are nullable with no default)
- No API version bump needed — purely additive change
