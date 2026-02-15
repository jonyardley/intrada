# API Contracts: Shuttle API Server

**Feature**: 017-shuttle-api-deploy
**Date**: 2026-02-15
**Base URL**: `https://<app-name>.shuttleapp.rs`

## Common Patterns

### Content Type
All API requests and responses use `Content-Type: application/json`.

### Error Response Format
All error responses follow a consistent shape:
```json
{
  "error": "Human-readable error message"
}
```

### Status Codes
| Code | Meaning |
|------|---------|
| 200 | Success (read, update, delete) |
| 201 | Created (new resource) |
| 400 | Validation error (invalid input) |
| 404 | Resource not found |
| 500 | Internal server error |

### ID Format
All resource IDs are ULIDs (26-character, lexicographically sortable strings), generated server-side on creation.

---

## Health Check

### `GET /api/health`

Returns server health status.

**Response** `200 OK`:
```json
{
  "status": "ok"
}
```

---

## Pieces

### `GET /api/pieces`

List all pieces. Returns an array sorted by `created_at` descending (newest first).

**Response** `200 OK`:
```json
[
  {
    "id": "01HQXYZ...",
    "title": "Moonlight Sonata",
    "composer": "Beethoven",
    "key": "C# minor",
    "tempo": {
      "marking": "Adagio sostenuto",
      "bpm": 52
    },
    "notes": "Focus on dynamics in the first movement",
    "tags": ["classical", "sonata"],
    "created_at": "2026-02-15T10:30:00Z",
    "updated_at": "2026-02-15T10:30:00Z"
  }
]
```

### `GET /api/pieces/{id}`

Get a single piece by ID.

**Response** `200 OK`: Single piece object (same shape as array element above).

**Response** `404 Not Found`:
```json
{
  "error": "Piece not found: 01HQXYZ..."
}
```

### `POST /api/pieces`

Create a new piece. Server generates `id`, `created_at`, and `updated_at`.

**Request Body** (maps to `CreatePiece`):
```json
{
  "title": "Moonlight Sonata",
  "composer": "Beethoven",
  "key": "C# minor",
  "tempo": {
    "marking": "Adagio sostenuto",
    "bpm": 52
  },
  "notes": "Focus on dynamics",
  "tags": ["classical", "sonata"]
}
```

**Required fields**: `title`, `composer`
**Optional fields**: `key`, `tempo`, `notes`, `tags` (defaults to `[]`)

**Response** `201 Created`: The created piece object with server-generated fields.

**Response** `400 Bad Request`:
```json
{
  "error": "Validation failed: title must be between 1 and 500 characters"
}
```

### `PUT /api/pieces/{id}`

Update an existing piece. Uses PATCH-style semantics (only provided fields are updated).

**Request Body** (maps to `UpdatePiece`):
```json
{
  "title": "Moonlight Sonata, Op. 27 No. 2",
  "key": null,
  "tempo": {
    "marking": "Adagio sostenuto",
    "bpm": 54
  }
}
```

**Three-state field semantics** (for optional fields like `key`, `tempo`, `notes`):
- Field absent from JSON → field is not updated
- Field present with value → field is set to new value
- Field present with `null` → field is cleared

**Response** `200 OK`: The updated piece object.

**Response** `400 Bad Request`: Validation error.
**Response** `404 Not Found`: Piece does not exist.

### `DELETE /api/pieces/{id}`

Delete a piece by ID.

**Response** `200 OK`:
```json
{
  "message": "Piece deleted"
}
```

**Response** `404 Not Found`: Piece does not exist.

---

## Exercises

### `GET /api/exercises`

List all exercises. Returns an array sorted by `created_at` descending.

**Response** `200 OK`:
```json
[
  {
    "id": "01HQABC...",
    "title": "C Major Scale",
    "composer": null,
    "category": "Scales",
    "key": "C major",
    "tempo": {
      "marking": null,
      "bpm": 120
    },
    "notes": null,
    "tags": ["scales", "technique"],
    "created_at": "2026-02-15T10:30:00Z",
    "updated_at": "2026-02-15T10:30:00Z"
  }
]
```

### `GET /api/exercises/{id}`

Get a single exercise by ID.

**Response** `200 OK`: Single exercise object.
**Response** `404 Not Found`: Exercise does not exist.

### `POST /api/exercises`

Create a new exercise. Server generates `id`, `created_at`, and `updated_at`.

**Request Body** (maps to `CreateExercise`):
```json
{
  "title": "C Major Scale",
  "composer": null,
  "category": "Scales",
  "key": "C major",
  "tempo": {
    "bpm": 120
  },
  "notes": null,
  "tags": ["scales"]
}
```

**Required fields**: `title`
**Optional fields**: `composer`, `category`, `key`, `tempo`, `notes`, `tags` (defaults to `[]`)

**Response** `201 Created`: The created exercise object.
**Response** `400 Bad Request`: Validation error.

### `PUT /api/exercises/{id}`

Update an existing exercise. Same three-state PATCH semantics as pieces.

**Request Body** (maps to `UpdateExercise`):
```json
{
  "title": "C Major Scale (2 octaves)",
  "category": null
}
```

**Response** `200 OK`: The updated exercise object.
**Response** `400 Bad Request`: Validation error.
**Response** `404 Not Found`: Exercise does not exist.

### `DELETE /api/exercises/{id}`

Delete an exercise by ID.

**Response** `200 OK`:
```json
{
  "message": "Exercise deleted"
}
```

**Response** `404 Not Found`: Exercise does not exist.

---

## Practice Sessions

Sessions are **immutable once saved** — create and read only (no update or delete endpoints per FR-005).

### `GET /api/sessions`

List all practice sessions. Returns an array sorted by `completed_at` descending (most recent first).

**Response** `200 OK`:
```json
[
  {
    "id": "01HQDEF...",
    "entries": [
      {
        "id": "01HQGHI...",
        "item_id": "01HQXYZ...",
        "item_title": "Moonlight Sonata",
        "item_type": "piece",
        "position": 0,
        "duration_secs": 1200,
        "status": "completed",
        "notes": "Good tempo control"
      },
      {
        "id": "01HQJKL...",
        "item_id": "01HQABC...",
        "item_title": "C Major Scale",
        "item_type": "exercise",
        "position": 1,
        "duration_secs": 300,
        "status": "completed",
        "notes": null
      }
    ],
    "session_notes": "Focused session, good progress",
    "started_at": "2026-02-15T09:00:00Z",
    "completed_at": "2026-02-15T09:25:00Z",
    "total_duration_secs": 1500,
    "completion_status": "completed"
  }
]
```

### `GET /api/sessions/{id}`

Get a single session by ID, including all setlist entries.

**Response** `200 OK`: Single session object (same shape as array element above).
**Response** `404 Not Found`: Session does not exist.

### `POST /api/sessions`

Save a completed practice session. The client sends the full session object (id, entries, timestamps generated client-side during the practice flow). The server validates and persists it.

**Request Body** (maps to `PracticeSession`):
```json
{
  "id": "01HQDEF...",
  "entries": [
    {
      "id": "01HQGHI...",
      "item_id": "01HQXYZ...",
      "item_title": "Moonlight Sonata",
      "item_type": "piece",
      "position": 0,
      "duration_secs": 1200,
      "status": "completed",
      "notes": "Good tempo control"
    }
  ],
  "session_notes": "Focused session",
  "started_at": "2026-02-15T09:00:00Z",
  "completed_at": "2026-02-15T09:25:00Z",
  "total_duration_secs": 1500,
  "completion_status": "completed"
}
```

**Validation**:
- `entries` must have at least one entry (`validate_setlist_not_empty`)
- `session_notes` max 5000 chars (`validate_session_notes`)
- Each entry `notes` max 5000 chars (`validate_entry_notes`)
- `completion_status` must be "completed" or "ended_early"
- Each entry `status` must be "completed", "skipped", or "not_attempted"
- Each entry `item_type` must be "piece" or "exercise"

**Response** `201 Created`: The saved session object.
**Response** `400 Bad Request`: Validation error.

---

## Web Shell API Integration

### Data Flow

1. **App loads**: Read localStorage cache → render immediately → fetch from API in background
2. **API response arrives**: Update UI with fresh data → write to localStorage cache
3. **User creates/edits/deletes**: Send API request → on success, update UI + localStorage cache
4. **API error on write**: Show error message → keep local state intact → do not clear user input

### localStorage Keys

| Key | Content | Purpose |
|-----|---------|---------|
| `intrada:library` | `LibraryData` JSON | Cache of pieces + exercises |
| `intrada:sessions` | `SessionsData` JSON | Cache of completed sessions |
| `intrada:session-in-progress` | `ActiveSession` JSON | Crash recovery (localStorage-only, not sent to API) |

### Cache Strategy

- **Read**: localStorage first, then API fetch in background
- **Write**: API first, then update localStorage on success
- **Conflict**: Server is source of truth; background fetch replaces stale cache
- **Offline**: Display cached data + show stale-data warning; writes fail with error message
