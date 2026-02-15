# API Contract: Pieces

**Base Path**: `/api/pieces`

---

## List Pieces

**Endpoint**: `GET /api/pieces`

**Response** `200 OK`:
```json
[
  {
    "id": "01JMABCD1234567890ABCDEF",
    "title": "Moonlight Sonata",
    "composer": "Beethoven",
    "key": "C# minor",
    "tempo": {
      "marking": "Adagio sostenuto",
      "bpm": 56
    },
    "notes": "Focus on dynamics in the first movement",
    "tags": ["classical", "piano"],
    "created_at": "2026-02-15T10:30:00Z",
    "updated_at": "2026-02-15T10:30:00Z"
  }
]
```

**Ordering**: Reverse chronological (newest `created_at` first).

---

## Get Piece

**Endpoint**: `GET /api/pieces/{id}`

**Response** `200 OK`:
```json
{
  "id": "01JMABCD1234567890ABCDEF",
  "title": "Moonlight Sonata",
  "composer": "Beethoven",
  "key": "C# minor",
  "tempo": {
    "marking": "Adagio sostenuto",
    "bpm": 56
  },
  "notes": "Focus on dynamics in the first movement",
  "tags": ["classical", "piano"],
  "created_at": "2026-02-15T10:30:00Z",
  "updated_at": "2026-02-15T10:30:00Z"
}
```

**Response** `404 Not Found`:
```json
{
  "error": "Piece not found: 01JMABCD1234567890ABCDEF"
}
```

---

## Create Piece

**Endpoint**: `POST /api/pieces`

**Request Body**:
```json
{
  "title": "Moonlight Sonata",
  "composer": "Beethoven",
  "key": "C# minor",
  "tempo": {
    "marking": "Adagio sostenuto",
    "bpm": 56
  },
  "notes": "Focus on dynamics",
  "tags": ["classical", "piano"]
}
```

**Required fields**: `title`, `composer`
**Optional fields**: `key`, `tempo`, `notes`, `tags` (defaults to `[]`)

**Response** `201 Created`:
```json
{
  "id": "01JMABCD1234567890ABCDEF",
  "title": "Moonlight Sonata",
  "composer": "Beethoven",
  "key": "C# minor",
  "tempo": {
    "marking": "Adagio sostenuto",
    "bpm": 56
  },
  "notes": "Focus on dynamics",
  "tags": ["classical", "piano"],
  "created_at": "2026-02-15T10:30:00Z",
  "updated_at": "2026-02-15T10:30:00Z"
}
```

**Response** `400 Bad Request` (validation error):
```json
{
  "error": "Title is required"
}
```

---

## Update Piece

**Endpoint**: `PUT /api/pieces/{id}`

**Three-state semantics**:
- Omit a field → leave unchanged
- Set to `null` → clear the field
- Provide a value → update the field

**Request Body** (all fields optional):
```json
{
  "title": "Updated Title",
  "key": null,
  "tags": ["new-tag"]
}
```

**Response** `200 OK`: Returns the full updated piece (same shape as Get).

**Response** `404 Not Found`:
```json
{
  "error": "Piece not found: 01JMABCD1234567890ABCDEF"
}
```

**Response** `400 Bad Request`:
```json
{
  "error": "Title must be between 1 and 500 characters"
}
```

---

## Delete Piece

**Endpoint**: `DELETE /api/pieces/{id}`

**Response** `200 OK`:
```json
{
  "message": "Piece deleted"
}
```

**Response** `404 Not Found`:
```json
{
  "error": "Piece not found: 01JMABCD1234567890ABCDEF"
}
```
