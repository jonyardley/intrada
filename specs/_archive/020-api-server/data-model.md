# Data Model: API Server

## Entities

### Piece

A musical piece in the user's library.

| Field | Type | Constraints | Notes |
|-------|------|-------------|-------|
| id | TEXT | PRIMARY KEY, NOT NULL | ULID, generated server-side |
| title | TEXT | NOT NULL, 1-500 chars | Required |
| composer | TEXT | NOT NULL, 1-200 chars | Required |
| key_signature | TEXT | nullable, max 50 chars | Musical key (e.g., "C major") |
| tempo_marking | TEXT | nullable, max 100 chars | e.g., "Allegro" |
| tempo_bpm | INTEGER | nullable, 1-400 | Beats per minute |
| notes | TEXT | nullable, max 5000 chars | Freeform notes |
| tags | TEXT | NOT NULL, DEFAULT '[]' | JSON array of strings, each tag max 100 chars |
| created_at | TEXT | NOT NULL | ISO 8601 UTC timestamp |
| updated_at | TEXT | NOT NULL | ISO 8601 UTC timestamp |

**Tempo rule**: If tempo is present, at least one of `tempo_marking` or `tempo_bpm` must be non-null.

### Exercise

A practice exercise in the user's library.

| Field | Type | Constraints | Notes |
|-------|------|-------------|-------|
| id | TEXT | PRIMARY KEY, NOT NULL | ULID, generated server-side |
| title | TEXT | NOT NULL, 1-500 chars | Required |
| composer | TEXT | nullable, max 200 chars | Optional (unlike Piece) |
| category | TEXT | nullable, max 100 chars | Freeform category |
| key_signature | TEXT | nullable, max 50 chars | Musical key |
| tempo_marking | TEXT | nullable, max 100 chars | e.g., "Moderato" |
| tempo_bpm | INTEGER | nullable, 1-400 | Beats per minute |
| notes | TEXT | nullable, max 5000 chars | Freeform notes |
| tags | TEXT | NOT NULL, DEFAULT '[]' | JSON array of strings |
| created_at | TEXT | NOT NULL | ISO 8601 UTC timestamp |
| updated_at | TEXT | NOT NULL | ISO 8601 UTC timestamp |

### Practice Session

A completed practice session. Write-once — cannot be edited after creation.

| Field | Type | Constraints | Notes |
|-------|------|-------------|-------|
| id | TEXT | PRIMARY KEY, NOT NULL | ULID, generated server-side |
| session_notes | TEXT | nullable, max 5000 chars | Session-level notes |
| started_at | TEXT | NOT NULL | ISO 8601 UTC timestamp |
| completed_at | TEXT | NOT NULL | ISO 8601 UTC timestamp |
| total_duration_secs | INTEGER | NOT NULL | Total session duration in seconds |
| completion_status | TEXT | NOT NULL | "Completed" or "EndedEarly" |

### Setlist Entry

An individual item practiced within a session. Stored in a separate table with a foreign key to the session.

| Field | Type | Constraints | Notes |
|-------|------|-------------|-------|
| id | TEXT | PRIMARY KEY, NOT NULL | ULID, from client |
| session_id | TEXT | NOT NULL, FK → sessions.id | Parent session |
| item_id | TEXT | NOT NULL | Reference to piece/exercise (denormalised) |
| item_title | TEXT | NOT NULL | Denormalised title (survives item deletion) |
| item_type | TEXT | NOT NULL | "piece" or "exercise" |
| position | INTEGER | NOT NULL | Order in setlist (0-based) |
| duration_secs | INTEGER | NOT NULL | Time spent on this entry |
| status | TEXT | NOT NULL | "Completed", "Skipped", or "NotAttempted" |
| notes | TEXT | nullable, max 5000 chars | Entry-level notes |

## Relationships

```text
Piece (standalone)
Exercise (standalone)
PracticeSession 1──*── SetlistEntry
```

- Pieces and exercises are independent entities with no foreign keys
- Each practice session has one or more setlist entries
- Setlist entries reference pieces/exercises by `item_id` but this is denormalised (no FK constraint) — sessions remain readable even if the referenced item is deleted
- `ON DELETE CASCADE` on `session_id` FK ensures entries are cleaned up when a session is deleted

## SQLite Schema (DDL)

### Migration 0001: Create Pieces Table

```sql
CREATE TABLE IF NOT EXISTS pieces (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    composer TEXT NOT NULL,
    key_signature TEXT,
    tempo_marking TEXT,
    tempo_bpm INTEGER,
    notes TEXT,
    tags TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

### Migration 0002: Create Exercises Table

```sql
CREATE TABLE IF NOT EXISTS exercises (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    composer TEXT,
    category TEXT,
    key_signature TEXT,
    tempo_marking TEXT,
    tempo_bpm INTEGER,
    notes TEXT,
    tags TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

### Migration 0003: Create Sessions and Entries Tables

```sql
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY NOT NULL,
    session_notes TEXT,
    started_at TEXT NOT NULL,
    completed_at TEXT NOT NULL,
    total_duration_secs INTEGER NOT NULL,
    completion_status TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS setlist_entries (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL,
    item_title TEXT NOT NULL,
    item_type TEXT NOT NULL,
    position INTEGER NOT NULL,
    duration_secs INTEGER NOT NULL,
    status TEXT NOT NULL,
    notes TEXT
);

CREATE INDEX IF NOT EXISTS idx_setlist_entries_session_id ON setlist_entries(session_id);
```

## Type Mapping (Rust ↔ SQLite)

| Rust Type | SQLite Type | Conversion |
|-----------|-------------|------------|
| String | TEXT | Direct |
| Option<String> | TEXT (nullable) | None → NULL |
| u16 (BPM) | INTEGER | Cast to/from i64 |
| u64 (duration) | INTEGER | Cast to/from i64 |
| usize (position) | INTEGER | Cast to/from i64 |
| Vec<String> (tags) | TEXT (JSON) | serde_json::to_string / from_str |
| DateTime<Utc> | TEXT | to_rfc3339 / parse |
| CompletionStatus enum | TEXT | "Completed" / "EndedEarly" |
| EntryStatus enum | TEXT | "Completed" / "Skipped" / "NotAttempted" |
| Option<Tempo> | tempo_marking TEXT + tempo_bpm INTEGER | Flattened to two columns |

## Validation Rules (from intrada-core)

All validation is performed in the API route handlers using functions from `intrada_core::validation`:

| Rule | Constant | Value |
|------|----------|-------|
| Title length | MAX_TITLE | 1-500 chars |
| Composer length | MAX_COMPOSER | 1-200 chars |
| Category length | MAX_CATEGORY | 1-100 chars |
| Notes length | MAX_NOTES | 0-5000 chars |
| Tag length | MAX_TAG | 1-100 chars per tag |
| Tempo marking length | MAX_TEMPO_MARKING | 1-100 chars |
| BPM range | MIN_BPM / MAX_BPM | 1-400 |
| Session notes | MAX_NOTES | 0-5000 chars |
| Entry notes | MAX_NOTES | 0-5000 chars |
| Setlist | — | Must have at least one entry |
