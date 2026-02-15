# API Contract: Exercises

**Base Path**: `/api/exercises`

---

## List Exercises

**Endpoint**: `GET /api/exercises`

**Response** `200 OK`:
```json
[
  {
    "id": "01JMABCD1234567890ABCDEF",
    "title": "Hanon No. 1",
    "composer": "Charles-Louis Hanon",
    "category": "Technique",
    "key": "C major",
    "tempo": {
      "marking": "Moderato",
      "bpm": 108
    },
    "notes": "Even finger pressure throughout",
    "tags": ["technique", "finger-independence"],
    "created_at": "2026-02-15T10:30:00Z",
    "updated_at": "2026-02-15T10:30:00Z"
  }
]
```

**Ordering**: Reverse chronological (newest `created_at` first).

---

## Get Exercise

**Endpoint**: `GET /api/exercises/{id}`

**Response** `200 OK`: Full exercise object (same shape as list item).

**Response** `404 Not Found`:
```json
{
  "error": "Exercise not found: 01JMABCD1234567890ABCDEF"
}
```

---

## Create Exercise

**Endpoint**: `POST /api/exercises`

**Request Body**:
```json
{
  "title": "Hanon No. 1",
  "composer": "Charles-Louis Hanon",
  "category": "Technique",
  "key": "C major",
  "tempo": {
    "marking": "Moderato",
    "bpm": 108
  },
  "notes": "Even finger pressure throughout",
  "tags": ["technique", "finger-independence"]
}
```

**Required fields**: `title`
**Optional fields**: `composer`, `category`, `key`, `tempo`, `notes`, `tags` (defaults to `[]`)

**Note**: Unlike pieces, `composer` is optional for exercises.

**Response** `201 Created`: Full exercise object with generated `id`, `created_at`, `updated_at`.

**Response** `400 Bad Request`:
```json
{
  "error": "Title is required"
}
```

---

## Update Exercise

**Endpoint**: `PUT /api/exercises/{id}`

**Three-state semantics** (same as pieces):
- Omit a field → leave unchanged
- Set to `null` → clear the field
- Provide a value → update the field

**Request Body** (all fields optional):
```json
{
  "title": "Updated Title",
  "category": "Sight Reading",
  "composer": null
}
```

**Response** `200 OK`: Full updated exercise object.

**Response** `404 Not Found` / `400 Bad Request`: Same format as pieces.

---

## Delete Exercise

**Endpoint**: `DELETE /api/exercises/{id}`

**Response** `200 OK`:
```json
{
  "message": "Exercise deleted"
}
```

**Response** `404 Not Found`:
```json
{
  "error": "Exercise not found: 01JMABCD1234567890ABCDEF"
}
```
