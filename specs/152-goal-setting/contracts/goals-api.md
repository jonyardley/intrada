# API Contract: Goals

**Base path**: `/api/goals`
**Authentication**: Required (JWT Bearer token, user_id from `sub` claim)

## Endpoints

### List Goals

```
GET /api/goals
```

Returns all goals for the authenticated user, ordered by `created_at DESC`.

**Response** `200 OK`:
```json
[
  {
    "id": "01HXYZ...",
    "title": "Practise 5 days per week",
    "kind": {
      "type": "session_frequency",
      "target_days_per_week": 5
    },
    "status": "active",
    "deadline": null,
    "created_at": "2026-02-24T10:00:00Z",
    "updated_at": "2026-02-24T10:00:00Z",
    "completed_at": null
  },
  {
    "id": "01HXYZ...",
    "title": "Master Chopin Nocturne",
    "kind": {
      "type": "item_mastery",
      "item_id": "01HABCD...",
      "target_score": 4
    },
    "status": "active",
    "deadline": "2026-03-15T00:00:00Z",
    "created_at": "2026-02-20T08:00:00Z",
    "updated_at": "2026-02-20T08:00:00Z",
    "completed_at": null
  }
]
```

### Create Goal

```
POST /api/goals
Content-Type: application/json
```

**Request body**:
```json
{
  "title": "Practise 5 days per week",
  "kind": {
    "type": "session_frequency",
    "target_days_per_week": 5
  },
  "deadline": null
}
```

**Response** `201 Created`: Full Goal object (same shape as list items)

**Validation errors** `422 Unprocessable Entity`:
```json
{
  "error": "Validation",
  "field": "title",
  "message": "Title must be between 1 and 200 characters"
}
```

### Get Goal

```
GET /api/goals/{id}
```

**Response** `200 OK`: Full Goal object
**Response** `404 Not Found`: `{ "error": "Goal not found: {id}" }`

### Update Goal

```
PUT /api/goals/{id}
Content-Type: application/json
```

**Request body** (all fields optional — omit to skip, `null` to clear):
```json
{
  "title": "Practise 6 days per week",
  "status": "completed",
  "deadline": null
}
```

Editable fields: `title`, `status`, `deadline`, and the type-specific target value (e.g., `target_days_per_week` for frequency goals). Goal type (`kind.type`) is immutable.

**Status transitions**:
- `"status": "completed"` — marks goal complete (sets `completed_at`, final)
- `"status": "archived"` — archives goal
- `"status": "active"` — reactivates an archived goal (only valid from archived status)

**Response** `200 OK`: Updated Goal object
**Response** `404 Not Found`: `{ "error": "Goal not found: {id}" }`

### Delete Goal

```
DELETE /api/goals/{id}
```

**Response** `200 OK`: `{ "message": "Goal deleted" }`
**Response** `404 Not Found`: `{ "error": "Goal not found: {id}" }`

## Type Examples

### Session Frequency
```json
{
  "kind": {
    "type": "session_frequency",
    "target_days_per_week": 5
  }
}
```

### Practice Time
```json
{
  "kind": {
    "type": "practice_time",
    "target_minutes_per_week": 120
  }
}
```

### Item Mastery
```json
{
  "kind": {
    "type": "item_mastery",
    "item_id": "01HABCD...",
    "target_score": 4
  }
}
```

### Milestone
```json
{
  "kind": {
    "type": "milestone",
    "description": "Memorise the first movement of Moonlight Sonata"
  }
}
```
