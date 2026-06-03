# API Contracts: Teacher Assignment Capture

Base URL: `/api`  
Auth: JWT Bearer token (Clerk, RS256 against JWKS)  
All endpoints scoped by `user_id` from JWT `sub` claim.

---

## Lessons

### POST /api/lessons

Create a new lesson.

**Request**:
```json
{
  "date": "2026-04-11",
  "notes": "Worked on thirds, keep wrist relaxed..."
}
```

**Validation**:
- `date`: Required. Valid date string (YYYY-MM-DD). Cannot be in the future.
- `notes`: Optional. Max 10,000 characters.

**Response** (201 Created):
```json
{
  "id": "01JRZX...",
  "date": "2026-04-11",
  "notes": "Worked on thirds, keep wrist relaxed...",
  "photos": [],
  "created_at": "2026-04-11T14:30:00Z",
  "updated_at": "2026-04-11T14:30:00Z"
}
```

**Errors**:
- 400: Validation failure
- 401: Unauthorized

---

### GET /api/lessons

List all lessons for the authenticated user, reverse chronological.

**Response** (200 OK):
```json
[
  {
    "id": "01JRZX...",
    "date": "2026-04-11",
    "notes": "Worked on thirds, keep wrist relaxed...",
    "photos": [
      {
        "id": "01JRZ1...",
        "url": "https://r2.example.com/user123/lesson456/photo789.jpg",
        "created_at": "2026-04-11T14:31:00Z"
      }
    ],
    "created_at": "2026-04-11T14:30:00Z",
    "updated_at": "2026-04-11T14:30:00Z"
  }
]
```

**Errors**:
- 401: Unauthorized

---

### GET /api/lessons/:id

Get a single lesson by ID.

**Response** (200 OK): Same shape as single item in list response.

**Errors**:
- 401: Unauthorized
- 404: Not found (or belongs to another user)

---

### PUT /api/lessons/:id

Update an existing lesson.

**Request**:
```json
{
  "date": "2026-04-10",
  "notes": "Updated notes..."
}
```

**Validation**: Same as create. All fields optional — only provided fields are updated.

**Response** (200 OK): Updated lesson object (same shape as GET).

**Errors**:
- 400: Validation failure
- 401: Unauthorized
- 404: Not found

---

### DELETE /api/lessons/:id

Delete a lesson and all associated photos.

**Response** (204 No Content)

**Side effects**: All photos in R2 for this lesson are also deleted.

**Errors**:
- 401: Unauthorized
- 404: Not found

---

## Lesson Photos

Photo upload/delete is handled outside the Crux effect system. The shell calls these endpoints directly.

### POST /api/lessons/:id/photos

Upload a photo to a lesson. Multipart form data.

**Request**: `multipart/form-data`
- `photo`: Image file (JPEG/PNG, max 5MB after client-side compression)

**Response** (201 Created):
```json
{
  "id": "01JRZ1...",
  "url": "https://r2.example.com/user123/lesson456/photo789.jpg",
  "created_at": "2026-04-11T14:31:00Z"
}
```

**Validation**:
- File must be JPEG or PNG
- Max 5MB
- Lesson must exist and belong to user

**Errors**:
- 400: Invalid file type or size
- 401: Unauthorized
- 404: Lesson not found

---

### DELETE /api/lessons/:id/photos/:photo_id

Remove a photo from a lesson.

**Response** (204 No Content)

**Side effects**: Photo object deleted from R2.

**Errors**:
- 401: Unauthorized
- 404: Lesson or photo not found
