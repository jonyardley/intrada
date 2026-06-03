# API Contract: Practice Sessions

**Base Path**: `/api/sessions`

---

## List Sessions

**Endpoint**: `GET /api/sessions`

**Response** `200 OK`:
```json
[
  {
    "id": "01JMABCD1234567890ABCDEF",
    "entries": [
      {
        "id": "01JMENTRY00000000000001",
        "item_id": "01JMPIECE00000000000001",
        "item_title": "Moonlight Sonata",
        "item_type": "piece",
        "position": 0,
        "duration_secs": 600,
        "status": "Completed",
        "notes": "Good dynamics today"
      },
      {
        "id": "01JMENTRY00000000000002",
        "item_id": "01JMEXERC00000000000001",
        "item_title": "Hanon No. 1",
        "item_type": "exercise",
        "position": 1,
        "duration_secs": 300,
        "status": "Skipped",
        "notes": null
      }
    ],
    "session_notes": "Productive session, focused on Beethoven",
    "started_at": "2026-02-15T09:00:00Z",
    "completed_at": "2026-02-15T09:45:00Z",
    "total_duration_secs": 2700,
    "completion_status": "Completed"
  }
]
```

**Ordering**: Reverse chronological (newest `completed_at` first).

**Note**: List includes full entries for each session. Given the expected dataset size (hundreds of sessions), this avoids the need for separate detail fetches. Pagination can be added later if needed.

---

## Get Session

**Endpoint**: `GET /api/sessions/{id}`

**Response** `200 OK`: Full session object (same shape as list item).

**Response** `404 Not Found`:
```json
{
  "error": "Session not found: 01JMABCD1234567890ABCDEF"
}
```

---

## Save Session

**Endpoint**: `POST /api/sessions`

Sessions are write-once. The client sends a fully formed completed session.

**Request Body**:
```json
{
  "entries": [
    {
      "id": "01JMENTRY00000000000001",
      "item_id": "01JMPIECE00000000000001",
      "item_title": "Moonlight Sonata",
      "item_type": "piece",
      "position": 0,
      "duration_secs": 600,
      "status": "Completed",
      "notes": "Good dynamics today"
    }
  ],
  "session_notes": "Productive session",
  "started_at": "2026-02-15T09:00:00Z",
  "completed_at": "2026-02-15T09:45:00Z",
  "total_duration_secs": 2700,
  "completion_status": "Completed"
}
```

**Required fields**: `entries` (non-empty), `started_at`, `completed_at`, `total_duration_secs`, `completion_status`
**Optional fields**: `session_notes`

**Entry required fields**: `id`, `item_id`, `item_title`, `item_type`, `position`, `duration_secs`, `status`
**Entry optional fields**: `notes`

**Note**: The session `id` is generated server-side. Entry `id`s are provided by the client (they were generated client-side during the active session).

**Response** `201 Created`: Full session object with server-generated `id`.

**Response** `400 Bad Request`:
```json
{
  "error": "Setlist must have at least one entry"
}
```

---

## Delete Session

**Endpoint**: `DELETE /api/sessions/{id}`

**Response** `200 OK`:
```json
{
  "message": "Session deleted"
}
```

**Response** `404 Not Found`:
```json
{
  "error": "Session not found: 01JMABCD1234567890ABCDEF"
}
```

---

## Notes

- Sessions cannot be updated after creation (write-once).
- The `item_id` in setlist entries is denormalised — no foreign key constraint. If the referenced piece/exercise is deleted, the session remains readable because `item_title` preserves the name.
- `completion_status` must be one of: `"Completed"`, `"EndedEarly"`.
- `status` in entries must be one of: `"Completed"`, `"Skipped"`, `"NotAttempted"`.
