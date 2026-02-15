# Data Model: Shuttle API Server & Database

**Feature**: 017-shuttle-api-deploy
**Date**: 2026-02-15
**Source**: Domain types in `intrada-core/src/domain/`

## Entity Relationship Diagram

```text
┌──────────────┐     ┌───────────────────┐
│    pieces     │     │     exercises      │
├──────────────┤     ├───────────────────┤
│ id (PK)      │     │ id (PK)           │
│ title        │     │ title             │
│ composer     │     │ composer          │
│ key          │     │ category          │
│ tempo_marking│     │ key               │
│ tempo_bpm    │     │ tempo_marking     │
│ notes        │     │ tempo_bpm         │
│ tags         │     │ notes             │
│ created_at   │     │ tags              │
│ updated_at   │     │ created_at        │
└──────────────┘     │ updated_at        │
                     └───────────────────┘

┌───────────────────┐     ┌────────────────────┐
│ practice_sessions │     │  setlist_entries    │
├───────────────────┤     ├────────────────────┤
│ id (PK)           │◄────│ session_id (FK)    │
│ session_notes     │     │ id (PK)            │
│ started_at        │     │ item_id            │
│ completed_at      │     │ item_title         │
│ total_duration_s  │     │ item_type          │
│ completion_status │     │ position           │
└───────────────────┘     │ duration_secs      │
                          │ status             │
                          │ notes              │
                          └────────────────────┘
```

## Tables

### `pieces`

Maps to `intrada_core::Piece`. Composer is required for pieces.

| Column | Type | Nullable | Constraints | Notes |
|--------|------|----------|-------------|-------|
| `id` | `TEXT` | NOT NULL | PRIMARY KEY | ULID string |
| `title` | `TEXT` | NOT NULL | 1-500 chars | Validated by `validate_create_piece` |
| `composer` | `TEXT` | NOT NULL | 1-200 chars | Required for pieces |
| `key` | `TEXT` | NULL | | Musical key (e.g., "C major") |
| `tempo_marking` | `TEXT` | NULL | | e.g., "Allegro" (max 100 chars) |
| `tempo_bpm` | `SMALLINT` | NULL | 1-400 | Beats per minute |
| `notes` | `TEXT` | NULL | max 5000 chars | Free-form notes |
| `tags` | `TEXT[]` | NOT NULL | DEFAULT '{}' | Array of tag strings (each max 100 chars) |
| `created_at` | `TIMESTAMPTZ` | NOT NULL | | UTC timestamp |
| `updated_at` | `TIMESTAMPTZ` | NOT NULL | | UTC timestamp |

**Design note**: The `Tempo` struct from core has two optional fields (`marking` and `bpm`). These are stored as two separate columns rather than a JSONB column, for queryability and simplicity. When both are NULL, the application interprets this as "no tempo set" (matching `Option<Tempo>` = `None`).

### `exercises`

Maps to `intrada_core::Exercise`. Composer is optional for exercises.

| Column | Type | Nullable | Constraints | Notes |
|--------|------|----------|-------------|-------|
| `id` | `TEXT` | NOT NULL | PRIMARY KEY | ULID string |
| `title` | `TEXT` | NOT NULL | 1-500 chars | Validated by `validate_create_exercise` |
| `composer` | `TEXT` | NULL | 1-200 chars | Optional for exercises |
| `category` | `TEXT` | NULL | 1-100 chars | e.g., "Scales", "Arpeggios" |
| `key` | `TEXT` | NULL | | Musical key |
| `tempo_marking` | `TEXT` | NULL | max 100 chars | e.g., "Andante" |
| `tempo_bpm` | `SMALLINT` | NULL | 1-400 | Beats per minute |
| `notes` | `TEXT` | NULL | max 5000 chars | Free-form notes |
| `tags` | `TEXT[]` | NOT NULL | DEFAULT '{}' | Array of tag strings |
| `created_at` | `TIMESTAMPTZ` | NOT NULL | | UTC timestamp |
| `updated_at` | `TIMESTAMPTZ` | NOT NULL | | UTC timestamp |

### `practice_sessions`

Maps to `intrada_core::PracticeSession`. Immutable once saved (no update/delete endpoints per spec FR-005).

| Column | Type | Nullable | Constraints | Notes |
|--------|------|----------|-------------|-------|
| `id` | `TEXT` | NOT NULL | PRIMARY KEY | ULID string |
| `session_notes` | `TEXT` | NULL | max 5000 chars | Overall session notes |
| `started_at` | `TIMESTAMPTZ` | NOT NULL | | When the session began |
| `completed_at` | `TIMESTAMPTZ` | NOT NULL | | When the session ended |
| `total_duration_secs` | `BIGINT` | NOT NULL | >= 0 | Total active practice time |
| `completion_status` | `TEXT` | NOT NULL | 'completed' or 'ended_early' | Maps to `CompletionStatus` enum |

### `setlist_entries`

Maps to `intrada_core::SetlistEntry`. Child of `practice_sessions`.

| Column | Type | Nullable | Constraints | Notes |
|--------|------|----------|-------------|-------|
| `id` | `TEXT` | NOT NULL | PRIMARY KEY | ULID string |
| `session_id` | `TEXT` | NOT NULL | FK → practice_sessions.id, ON DELETE CASCADE | Parent session |
| `item_id` | `TEXT` | NOT NULL | | Reference to piece/exercise ID |
| `item_title` | `TEXT` | NOT NULL | | Snapshot of item title at practice time |
| `item_type` | `TEXT` | NOT NULL | 'piece' or 'exercise' | Item type discriminator |
| `position` | `INTEGER` | NOT NULL | >= 0 | Order within the setlist |
| `duration_secs` | `BIGINT` | NOT NULL | >= 0 | Time spent on this item |
| `status` | `TEXT` | NOT NULL | 'completed', 'skipped', or 'not_attempted' | Maps to `EntryStatus` enum |
| `notes` | `TEXT` | NULL | max 5000 chars | Per-entry practice notes |

**Ordering**: Entries within a session are ordered by `position` (set during session creation, immutable after save).

## SQL Migrations

### 001_create_pieces.sql

```sql
CREATE TABLE IF NOT EXISTS pieces (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    composer TEXT NOT NULL,
    key TEXT,
    tempo_marking TEXT,
    tempo_bpm SMALLINT CHECK (tempo_bpm IS NULL OR (tempo_bpm >= 1 AND tempo_bpm <= 400)),
    notes TEXT,
    tags TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);
```

### 002_create_exercises.sql

```sql
CREATE TABLE IF NOT EXISTS exercises (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    composer TEXT,
    category TEXT,
    key TEXT,
    tempo_marking TEXT,
    tempo_bpm SMALLINT CHECK (tempo_bpm IS NULL OR (tempo_bpm >= 1 AND tempo_bpm <= 400)),
    notes TEXT,
    tags TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);
```

### 003_create_sessions.sql

```sql
CREATE TABLE IF NOT EXISTS practice_sessions (
    id TEXT PRIMARY KEY,
    session_notes TEXT,
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ NOT NULL,
    total_duration_secs BIGINT NOT NULL CHECK (total_duration_secs >= 0),
    completion_status TEXT NOT NULL CHECK (completion_status IN ('completed', 'ended_early'))
);

CREATE TABLE IF NOT EXISTS setlist_entries (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES practice_sessions(id) ON DELETE CASCADE,
    item_id TEXT NOT NULL,
    item_title TEXT NOT NULL,
    item_type TEXT NOT NULL CHECK (item_type IN ('piece', 'exercise')),
    position INTEGER NOT NULL CHECK (position >= 0),
    duration_secs BIGINT NOT NULL CHECK (duration_secs >= 0),
    status TEXT NOT NULL CHECK (status IN ('completed', 'skipped', 'not_attempted')),
    notes TEXT
);

CREATE INDEX idx_setlist_entries_session_id ON setlist_entries(session_id);
```

## Type Mapping: Rust ↔ Postgres

| Rust Type | Postgres Type | Conversion Notes |
|-----------|--------------|------------------|
| `String` (ULID) | `TEXT` | Stored as string, generated by server |
| `String` | `TEXT` | Direct mapping |
| `Option<String>` | `TEXT` (nullable) | `None` → SQL NULL |
| `Option<Tempo>` | `TEXT` + `SMALLINT` (nullable cols) | Flattened to `tempo_marking` + `tempo_bpm`; both NULL = None |
| `Vec<String>` (tags) | `TEXT[]` | Postgres array; empty = `'{}'` |
| `DateTime<Utc>` | `TIMESTAMPTZ` | sqlx handles chrono ↔ TIMESTAMPTZ automatically |
| `u64` (duration) | `BIGINT` | Rust u64 fits in Postgres BIGINT |
| `usize` (position) | `INTEGER` | Position values are small |
| `EntryStatus` enum | `TEXT` | Stored as lowercase string ('completed', 'skipped', 'not_attempted') |
| `CompletionStatus` enum | `TEXT` | Stored as lowercase string ('completed', 'ended_early') |
| `Option<u16>` (BPM) | `SMALLINT` (nullable) | sqlx handles Option mapping |

## Validation Rules (from intrada-core)

All validation is performed server-side by calling `intrada_core::validation::*` functions before database writes. The same validation runs client-side in the Crux core.

| Field | Rule | Constant |
|-------|------|----------|
| title | 1-500 characters | `MAX_TITLE = 500` |
| composer | 1-200 characters | `MAX_COMPOSER = 200` |
| category | 1-100 characters | `MAX_CATEGORY = 100` |
| notes | 0-5000 characters | `MAX_NOTES = 5000` |
| tag (each) | 1-100 characters | `MAX_TAG = 100` |
| tempo_marking | 0-100 characters | `MAX_TEMPO_MARKING = 100` |
| BPM | 1-400 | `MIN_BPM = 1`, `MAX_BPM = 400` |
| session_notes | 0-5000 characters | `MAX_NOTES = 5000` |
| entry_notes | 0-5000 characters | `MAX_NOTES = 5000` |
| setlist entries | at least 1 entry | `validate_setlist_not_empty` |
