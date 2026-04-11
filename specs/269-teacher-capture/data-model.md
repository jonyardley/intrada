# Data Model: Teacher Assignment Capture

## Entities

### Lesson

A record of a single teaching session. Lightweight — designed for speed of capture.

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| id | String (ULID) | Yes | Server-generated |
| user_id | String | Yes | From JWT `sub` claim |
| date | Date (YYYY-MM-DD) | Yes | Defaults to today, editable |
| notes | String | No | Free text, max 10,000 chars |
| created_at | DateTime (RFC3339) | Yes | Server-generated |
| updated_at | DateTime (RFC3339) | Yes | Server-managed |

**Validation rules**:
- `date`: Must be a valid date, cannot be in the future (allow today and past dates)
- `notes`: Max 10,000 characters (generous — this replaces a notebook page)
- At least one of `notes` or photos must be present (enforced at application level, not DB)

### LessonPhoto

A photo attachment belonging to a lesson. Stored as an object in Cloudflare R2; the DB holds metadata only.

| Field | Type | Required | Notes |
|-------|------|----------|-------|
| id | String (ULID) | Yes | Server-generated |
| lesson_id | String | Yes | Foreign key to lessons.id |
| user_id | String | Yes | From JWT `sub` claim |
| storage_key | String | Yes | R2 object key (e.g., `{user_id}/{lesson_id}/{id}.jpg`) |
| created_at | DateTime (RFC3339) | Yes | Server-generated |

**Relationships**:
- Lesson 1 → N LessonPhoto (cascade delete: deleting a lesson deletes its photos)
- LessonPhoto is owned by a user (scoped queries)

## Database Schema

### Migration: Create lessons table

```sql
CREATE TABLE IF NOT EXISTS lessons (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL DEFAULT '',
    date TEXT NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_lessons_user_date
    ON lessons(user_id, date DESC);
```

### Migration: Create lesson_photos table

```sql
CREATE TABLE IF NOT EXISTS lesson_photos (
    id TEXT PRIMARY KEY NOT NULL,
    lesson_id TEXT NOT NULL,
    user_id TEXT NOT NULL DEFAULT '',
    storage_key TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (lesson_id) REFERENCES lessons(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_lesson_photos_lesson_id
    ON lesson_photos(lesson_id);
```

## Column Indexing

Following the project pattern of positional `SELECT_COLUMNS`:

**Lessons**:
```
0: id
1: user_id
2: date
3: notes
4: created_at
5: updated_at
```

**LessonPhotos**:
```
0: id
1: lesson_id
2: user_id
3: storage_key
4: created_at
```

## State Transitions

Lesson has no complex state machine — it supports CRUD operations only:

```
[Not exists] → Create → [Exists] → Edit → [Exists]
                                   → Delete → [Not exists]
```

Photo lifecycle:
```
[Not exists] → Upload (shell) → [Stored in R2 + DB record] → Delete → [Removed from R2 + DB]
```

## Core Domain Types

```
Lesson {
    id: String,
    date: NaiveDate,
    notes: Option<String>,
    photos: Vec<LessonPhoto>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

LessonPhoto {
    id: String,
    storage_key: String,
    url: String,          // Derived: R2 public URL from storage_key
    created_at: DateTime<Utc>,
}

CreateLesson {
    date: NaiveDate,
    notes: Option<String>,
}

UpdateLesson {
    date: Option<NaiveDate>,
    notes: Option<String>,
}
```
