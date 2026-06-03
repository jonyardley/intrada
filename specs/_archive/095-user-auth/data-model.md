# Data Model: User Authentication

## Entity Changes

Three existing tables gain a `user_id` column. No new tables are created — Clerk manages user profiles externally.

### items

| Column | Type | Change |
|--------|------|--------|
| user_id | TEXT NOT NULL DEFAULT '' | **Added** — Clerk user ID (e.g., `user_2abc123def`) |

### sessions

| Column | Type | Change |
|--------|------|--------|
| user_id | TEXT NOT NULL DEFAULT '' | **Added** — Clerk user ID |

### routines

| Column | Type | Change |
|--------|------|--------|
| user_id | TEXT NOT NULL DEFAULT '' | **Added** — Clerk user ID |

### Child tables (no changes)

- `setlist_entries` — accessed via `session_id` FK, no direct user scoping needed
- `routine_entries` — accessed via `routine_id` FK, no direct user scoping needed

## Why no `users` table

Clerk manages user profiles (name, email, avatar). The `user_id` column stores Clerk's user ID string. No local user data is needed for this feature. A `users` table may be added later if local user preferences are needed.

## Why `DEFAULT ''`

SQLite `ALTER TABLE ADD COLUMN` requires a default for `NOT NULL` columns. Existing anonymous rows get `user_id = ''`. Authenticated queries filter `WHERE user_id = ?` with a real Clerk user ID (never empty), so anonymous data is naturally inaccessible. In auth-optional mode (tests/dev), the extractor returns `""`, matching the default.

## Migrations

Starting from 0013 (current last migration is 0012: `migrate_exercises_to_items`).

### 0013: add_user_id_to_items

```sql
ALTER TABLE items ADD COLUMN user_id TEXT NOT NULL DEFAULT '';
```

### 0014: add_user_id_to_sessions

```sql
ALTER TABLE sessions ADD COLUMN user_id TEXT NOT NULL DEFAULT '';
```

### 0015: add_user_id_to_routines

```sql
ALTER TABLE routines ADD COLUMN user_id TEXT NOT NULL DEFAULT '';
```

### 0016: index_items_user_id

```sql
CREATE INDEX IF NOT EXISTS idx_items_user_id ON items(user_id);
```

### 0017: index_sessions_user_id

```sql
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
```

### 0018: index_routines_user_id

```sql
CREATE INDEX IF NOT EXISTS idx_routines_user_id ON routines(user_id);
```

## Query Changes

All DB functions gain a `user_id: &str` parameter. The scoping follows this pattern:

**SELECT** (list/get): Add `WHERE user_id = ?`
```sql
-- Before
SELECT * FROM items ORDER BY created_at DESC
-- After
SELECT * FROM items WHERE user_id = ? ORDER BY created_at DESC
```

**INSERT** (create): Include `user_id` in values
```sql
-- Before
INSERT INTO items (id, kind, title, ...) VALUES (?, ?, ?, ...)
-- After
INSERT INTO items (id, kind, title, ..., user_id) VALUES (?, ?, ?, ..., ?)
```

**UPDATE**: Add `AND user_id = ?` to WHERE clause
```sql
-- Before
UPDATE items SET title = ?, ... WHERE id = ?
-- After
UPDATE items SET title = ?, ... WHERE id = ? AND user_id = ?
```

**DELETE**: Add `AND user_id = ?` to WHERE clause
```sql
-- Before
DELETE FROM items WHERE id = ?
-- After
DELETE FROM items WHERE id = ? AND user_id = ?
```

This prevents cross-user access at the query level — even if a user guesses another user's item ID, the `user_id` filter rejects the operation.
