use libsql::Connection;

/// Single source of truth for all database migrations.
///
/// Each entry is `(name, sql)` where `sql` must contain exactly ONE SQL statement.
/// Production uses `run_migrations()` (via libsql_migration tracking).
/// Tests use `run_migrations_direct()` (raw execution, same SQL).
const MIGRATIONS: &[(&str, &str)] = &[
    (
        "0001_create_pieces",
        "CREATE TABLE IF NOT EXISTS pieces (
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
        );",
    ),
    (
        "0002_create_exercises",
        "CREATE TABLE IF NOT EXISTS exercises (
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
        );",
    ),
    (
        "0003_create_sessions",
        "CREATE TABLE IF NOT EXISTS sessions (
            id TEXT PRIMARY KEY NOT NULL,
            session_notes TEXT,
            started_at TEXT NOT NULL,
            completed_at TEXT NOT NULL,
            total_duration_secs INTEGER NOT NULL,
            completion_status TEXT NOT NULL
        );",
    ),
    (
        "0004_create_setlist_entries",
        "CREATE TABLE IF NOT EXISTS setlist_entries (
            id TEXT PRIMARY KEY NOT NULL,
            session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
            item_id TEXT NOT NULL,
            item_title TEXT NOT NULL,
            item_type TEXT NOT NULL,
            position INTEGER NOT NULL,
            duration_secs INTEGER NOT NULL,
            status TEXT NOT NULL,
            notes TEXT
        );",
    ),
    (
        "0005_index_setlist_entries_session_id",
        "CREATE INDEX IF NOT EXISTS idx_setlist_entries_session_id ON setlist_entries(session_id);",
    ),
    (
        "0006_add_score_to_setlist_entries",
        "ALTER TABLE setlist_entries ADD COLUMN score INTEGER;",
    ),
    (
        "0007_create_routines",
        "CREATE TABLE IF NOT EXISTS routines (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );",
    ),
    (
        "0008_create_routine_entries",
        "CREATE TABLE IF NOT EXISTS routine_entries (
            id TEXT PRIMARY KEY NOT NULL,
            routine_id TEXT NOT NULL REFERENCES routines(id) ON DELETE CASCADE,
            item_id TEXT NOT NULL,
            item_title TEXT NOT NULL,
            item_type TEXT NOT NULL,
            position INTEGER NOT NULL
        );",
    ),
    (
        "0009_index_routine_entries_routine_id",
        "CREATE INDEX IF NOT EXISTS idx_routine_entries_routine_id ON routine_entries(routine_id);",
    ),
    (
        "0010_create_items",
        "CREATE TABLE IF NOT EXISTS items (
            id TEXT PRIMARY KEY NOT NULL,
            kind TEXT NOT NULL,
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
        );",
    ),
    (
        "0011_migrate_pieces_to_items",
        "INSERT OR IGNORE INTO items (id, kind, title, composer, category, key_signature, tempo_marking, tempo_bpm, notes, tags, created_at, updated_at) SELECT id, 'piece', title, composer, NULL, key_signature, tempo_marking, tempo_bpm, notes, tags, created_at, updated_at FROM pieces;",
    ),
    (
        "0012_migrate_exercises_to_items",
        "INSERT OR IGNORE INTO items (id, kind, title, composer, category, key_signature, tempo_marking, tempo_bpm, notes, tags, created_at, updated_at) SELECT id, 'exercise', title, composer, category, key_signature, tempo_marking, tempo_bpm, notes, tags, created_at, updated_at FROM exercises;",
    ),
    (
        "0013_add_user_id_to_items",
        "ALTER TABLE items ADD COLUMN user_id TEXT NOT NULL DEFAULT '';",
    ),
    (
        "0014_add_user_id_to_sessions",
        "ALTER TABLE sessions ADD COLUMN user_id TEXT NOT NULL DEFAULT '';",
    ),
    (
        "0015_add_user_id_to_routines",
        "ALTER TABLE routines ADD COLUMN user_id TEXT NOT NULL DEFAULT '';",
    ),
    (
        "0016_index_items_user_id",
        "CREATE INDEX IF NOT EXISTS idx_items_user_id ON items(user_id);",
    ),
    (
        "0017_index_sessions_user_id",
        "CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);",
    ),
    (
        "0018_index_routines_user_id",
        "CREATE INDEX IF NOT EXISTS idx_routines_user_id ON routines(user_id);",
    ),
    (
        "0019_add_intention_to_setlist_entries",
        "ALTER TABLE setlist_entries ADD COLUMN intention TEXT;",
    ),
    (
        "0020_add_session_intention_to_sessions",
        "ALTER TABLE sessions ADD COLUMN session_intention TEXT;",
    ),
    (
        "0021_add_rep_target_to_setlist_entries",
        "ALTER TABLE setlist_entries ADD COLUMN rep_target INTEGER;",
    ),
    (
        "0022_add_rep_count_to_setlist_entries",
        "ALTER TABLE setlist_entries ADD COLUMN rep_count INTEGER;",
    ),
    (
        "0023_add_rep_target_reached_to_setlist_entries",
        "ALTER TABLE setlist_entries ADD COLUMN rep_target_reached INTEGER;",
    ),
    (
        "0024_add_rep_history_to_setlist_entries",
        "ALTER TABLE setlist_entries ADD COLUMN rep_history TEXT;",
    ),
    (
        "0025_add_planned_duration_secs_to_setlist_entries",
        "ALTER TABLE setlist_entries ADD COLUMN planned_duration_secs INTEGER;",
    ),
    (
        "0026_add_achieved_tempo_to_setlist_entries",
        "ALTER TABLE setlist_entries ADD COLUMN achieved_tempo INTEGER;",
    ),
    (
        "0027_create_goals",
        "CREATE TABLE IF NOT EXISTS goals (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL DEFAULT '',
            title TEXT NOT NULL,
            goal_type TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'active',
            target_days_per_week INTEGER,
            target_minutes_per_week INTEGER,
            item_id TEXT,
            target_score INTEGER,
            milestone_description TEXT,
            deadline TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            completed_at TEXT
        );",
    ),
    (
        "0028_index_goals_user_id",
        "CREATE INDEX IF NOT EXISTS idx_goals_user_id ON goals(user_id);",
    ),
    (
        "0029_index_items_user_created",
        "CREATE INDEX IF NOT EXISTS idx_items_user_created ON items(user_id, created_at DESC);",
    ),
    (
        "0030_index_sessions_user_started",
        "CREATE INDEX IF NOT EXISTS idx_sessions_user_started ON sessions(user_id, started_at DESC);",
    ),
    (
        "0031_drop_goals",
        "DROP TABLE IF EXISTS goals;",
    ),
    // Legacy pieces/exercises tables were replaced by the unified items table
    // (migrations 0010–0012). Data was migrated in 0011/0012. Safe to drop now.
    (
        "0032_drop_pieces",
        "DROP TABLE IF EXISTS pieces;",
    ),
    (
        "0033_drop_exercises",
        "DROP TABLE IF EXISTS exercises;",
    ),
    (
        "0034_drop_category_from_items",
        "ALTER TABLE items DROP COLUMN category;",
    ),
    (
        "0035_create_lessons",
        "CREATE TABLE IF NOT EXISTS lessons (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL DEFAULT '',
            date TEXT NOT NULL,
            notes TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );",
    ),
    (
        "0036_index_lessons_user_date",
        "CREATE INDEX IF NOT EXISTS idx_lessons_user_date ON lessons(user_id, date DESC);",
    ),
    (
        "0037_create_lesson_photos",
        "CREATE TABLE IF NOT EXISTS lesson_photos (
            id TEXT PRIMARY KEY NOT NULL,
            lesson_id TEXT NOT NULL,
            user_id TEXT NOT NULL DEFAULT '',
            storage_key TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (lesson_id) REFERENCES lessons(id) ON DELETE CASCADE
        );",
    ),
    (
        "0038_index_lesson_photos_lesson_id",
        "CREATE INDEX IF NOT EXISTS idx_lesson_photos_lesson_id ON lesson_photos(lesson_id);",
    ),
    // Drop the FK constraint from lesson_photos. Turso's remote engine
    // enforces FK regardless of the client-side `PRAGMA foreign_keys` —
    // and the FK parent-table read fails across Fly machines / replicas
    // that haven't yet observed a just-created lesson row, causing photo
    // upload 500s. SQLite can't ALTER TABLE DROP CONSTRAINT, so we do
    // the table-swap dance across 5 single-statement migrations.
    //
    // Orphan safety: `user_id` is on each row and `delete_lesson` deletes
    // child photos explicitly (be44d1a), so removing the FK doesn't leak.
    (
        "0039_create_lesson_photos_new",
        "CREATE TABLE IF NOT EXISTS lesson_photos_new (
            id TEXT PRIMARY KEY NOT NULL,
            lesson_id TEXT NOT NULL,
            user_id TEXT NOT NULL DEFAULT '',
            storage_key TEXT NOT NULL,
            created_at TEXT NOT NULL
        );",
    ),
    (
        "0040_copy_lesson_photos_to_new",
        "INSERT INTO lesson_photos_new (id, lesson_id, user_id, storage_key, created_at)
         SELECT id, lesson_id, user_id, storage_key, created_at FROM lesson_photos;",
    ),
    (
        "0041_drop_old_lesson_photos",
        "DROP TABLE lesson_photos;",
    ),
    (
        "0042_rename_lesson_photos_new",
        "ALTER TABLE lesson_photos_new RENAME TO lesson_photos;",
    ),
    (
        "0043_recreate_lesson_photos_lesson_id_index",
        "CREATE INDEX IF NOT EXISTS idx_lesson_photos_lesson_id ON lesson_photos(lesson_id);",
    ),
    // Drop FK on routine_entries.routine_id — same Turso failure mode as
    // lesson_photos (see #294). Orphan safety: delete_set and
    // update_set already delete child rows explicitly.
    (
        "0044_create_routine_entries_new",
        "CREATE TABLE IF NOT EXISTS routine_entries_new (
            id TEXT PRIMARY KEY NOT NULL,
            routine_id TEXT NOT NULL,
            item_id TEXT NOT NULL,
            item_title TEXT NOT NULL,
            item_type TEXT NOT NULL,
            position INTEGER NOT NULL
        );",
    ),
    (
        "0045_copy_routine_entries_to_new",
        "INSERT INTO routine_entries_new (id, routine_id, item_id, item_title, item_type, position)
         SELECT id, routine_id, item_id, item_title, item_type, position FROM routine_entries;",
    ),
    (
        "0046_drop_old_routine_entries",
        "DROP TABLE routine_entries;",
    ),
    (
        "0047_rename_routine_entries_new",
        "ALTER TABLE routine_entries_new RENAME TO routine_entries;",
    ),
    (
        "0048_recreate_routine_entries_routine_id_index",
        "CREATE INDEX IF NOT EXISTS idx_routine_entries_routine_id ON routine_entries(routine_id);",
    ),
    // Drop FK on setlist_entries.session_id — same Turso failure mode.
    // Orphan safety: delete_session already deletes child rows explicitly.
    // New schema must include every column added across 0006 + 0019-0025
    // (score, intention, rep_target, rep_count, rep_target_reached,
    // rep_history, planned_duration_secs, achieved_tempo).
    (
        "0049_create_setlist_entries_new",
        "CREATE TABLE IF NOT EXISTS setlist_entries_new (
            id TEXT PRIMARY KEY NOT NULL,
            session_id TEXT NOT NULL,
            item_id TEXT NOT NULL,
            item_title TEXT NOT NULL,
            item_type TEXT NOT NULL,
            position INTEGER NOT NULL,
            duration_secs INTEGER NOT NULL,
            status TEXT NOT NULL,
            notes TEXT,
            score INTEGER,
            intention TEXT,
            rep_target INTEGER,
            rep_count INTEGER,
            rep_target_reached INTEGER,
            rep_history TEXT,
            planned_duration_secs INTEGER,
            achieved_tempo INTEGER
        );",
    ),
    (
        "0050_copy_setlist_entries_to_new",
        "INSERT INTO setlist_entries_new (id, session_id, item_id, item_title, item_type, position, duration_secs, status, notes, score, intention, rep_target, rep_count, rep_target_reached, rep_history, planned_duration_secs, achieved_tempo)
         SELECT id, session_id, item_id, item_title, item_type, position, duration_secs, status, notes, score, intention, rep_target, rep_count, rep_target_reached, rep_history, planned_duration_secs, achieved_tempo FROM setlist_entries;",
    ),
    (
        "0051_drop_old_setlist_entries",
        "DROP TABLE setlist_entries;",
    ),
    (
        "0052_rename_setlist_entries_new",
        "ALTER TABLE setlist_entries_new RENAME TO setlist_entries;",
    ),
    (
        "0053_recreate_setlist_entries_session_id_index",
        "CREATE INDEX IF NOT EXISTS idx_setlist_entries_session_id ON setlist_entries(session_id);",
    ),
    (
        "0054_create_user_preferences",
        "CREATE TABLE IF NOT EXISTS user_preferences (
            user_id TEXT PRIMARY KEY NOT NULL,
            default_focus_minutes INTEGER,
            default_rep_count INTEGER,
            updated_at TEXT NOT NULL
        );",
    ),
    (
        "0055_create_mcp_tokens",
        "CREATE TABLE IF NOT EXISTS mcp_tokens (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            name TEXT NOT NULL,
            hash TEXT NOT NULL UNIQUE,
            prefix TEXT NOT NULL,
            last_used_at TEXT,
            created_at TEXT NOT NULL,
            revoked_at TEXT
        );",
    ),
    (
        "0056_index_mcp_tokens_user_id",
        "CREATE INDEX IF NOT EXISTS idx_mcp_tokens_user_id ON mcp_tokens(user_id);",
    ),
    (
        "0057_create_mcp_audit_log",
        "CREATE TABLE IF NOT EXISTS mcp_audit_log (
            id TEXT PRIMARY KEY NOT NULL,
            token_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            tool TEXT NOT NULL,
            args_hash TEXT NOT NULL,
            created_at TEXT NOT NULL
        );",
    ),
    (
        "0058_index_mcp_audit_log_user_created",
        "CREATE INDEX IF NOT EXISTS idx_mcp_audit_log_user_created ON mcp_audit_log(user_id, created_at DESC);",
    ),
    (
        "0059_create_oauth_clients",
        "CREATE TABLE IF NOT EXISTS oauth_clients (
            client_id TEXT PRIMARY KEY NOT NULL,
            client_secret_hash TEXT,
            client_name TEXT NOT NULL,
            redirect_uris TEXT NOT NULL,
            created_at TEXT NOT NULL
        );",
    ),
    (
        "0060_create_oauth_codes",
        "CREATE TABLE IF NOT EXISTS oauth_codes (
            code_hash TEXT PRIMARY KEY NOT NULL,
            client_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            redirect_uri TEXT NOT NULL,
            code_challenge TEXT NOT NULL,
            code_challenge_method TEXT NOT NULL,
            scope TEXT,
            expires_at TEXT NOT NULL,
            created_at TEXT NOT NULL
        );",
    ),
    (
        "0061_index_oauth_codes_expires",
        "CREATE INDEX IF NOT EXISTS idx_oauth_codes_expires ON oauth_codes(expires_at);",
    ),
];

/// Backoff schedule for transient-error retries during migration: try
/// immediately, then 200ms, 1s, 5s. Total max wall time per migration ≈
/// 6.2s before giving up. Sized for Turso's typical cold-start +
/// transient HTTP-stream-drop window seen in production (INTRADA-API-2):
/// the connection blip resolves on the first or second retry, but if
/// it's a sustained outage we still fail loudly within ~10s rather
/// than retrying forever.
const MIGRATION_RETRY_BACKOFF_MS: &[u64] = &[200, 1_000, 5_000];

// Transient-error classifier moved to `db::is_transient_db_error` so
// it's shared with the per-request retry helper in `state::Db`.

/// Run migrations via libsql_migration (production path — tracks applied state).
///
/// Each migration is wrapped in a retry-with-backoff loop for transient
/// connection errors against Turso (Hrana stream drops, cold-starts,
/// network blips). Permanent errors (SQL syntax, constraint violations)
/// fail immediately so we panic loudly on real bugs.
pub async fn run_migrations(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    for (id, sql) in MIGRATIONS {
        let result = run_one_migration_with_retry(conn, id, sql).await?;
        match result {
            libsql_migration::util::MigrationResult::Executed => {
                tracing::info!("Migration {id} applied");
            }
            libsql_migration::util::MigrationResult::AlreadyExecuted => {
                tracing::debug!("Migration {id} already applied");
            }
        }
    }
    Ok(())
}

/// Single migration with retry-on-transient. Caller iterates `MIGRATIONS`.
///
/// `libsql_migration::content::migrate` is itself idempotent — it
/// records applied state in `_migrations` and short-circuits on
/// `AlreadyExecuted` — so retrying a partially-applied migration is
/// safe: a successful apply on attempt N just observes `AlreadyExecuted`
/// on attempt N+1.
async fn run_one_migration_with_retry(
    conn: &Connection,
    id: &str,
    sql: &str,
) -> Result<libsql_migration::util::MigrationResult, Box<dyn std::error::Error>> {
    let mut attempt = 0;
    loop {
        match libsql_migration::content::migrate(conn, id.to_string(), sql.to_string()).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                let err_str = e.to_string();
                if attempt < MIGRATION_RETRY_BACKOFF_MS.len()
                    && crate::db::is_transient_db_error(&err_str)
                {
                    let backoff_ms = MIGRATION_RETRY_BACKOFF_MS[attempt];
                    tracing::warn!(
                        attempt = attempt + 1,
                        backoff_ms,
                        error = %err_str,
                        "Migration {id} hit transient error; retrying after backoff"
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
                    attempt += 1;
                    continue;
                }
                return Err(format!("Migration {id} failed: {e}").into());
            }
        }
    }
}

/// Run migrations via direct SQL execution (test path — no tracking overhead).
///
/// Uses the same `MIGRATIONS` source as `run_migrations()` to guarantee
/// tests and production always execute identical SQL.
pub async fn run_migrations_direct(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    for (_id, sql) in MIGRATIONS {
        conn.execute(sql, ()).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Guard: every migration must contain exactly one SQL statement.
    ///
    /// libsql_migration silently ignores all but the first statement in a
    /// multi-statement migration. This test catches that mistake at compile
    /// time rather than discovering it in production.
    #[test]
    fn each_migration_contains_single_statement() {
        for (name, sql) in MIGRATIONS {
            // Strip trailing whitespace and the final semicolon, then check
            // for any remaining semicolons — which would indicate multiple
            // statements were bundled together.
            let trimmed = sql.trim().trim_end_matches(';').trim();
            assert!(
                !trimmed.contains(';'),
                "Migration '{name}' contains multiple SQL statements. \
                 libsql_migration only executes the first statement. \
                 Split this into separate migrations."
            );
        }
    }

    /// Guard: the patterns we treat as transient cover the actual error
    /// strings we've seen (and ones we expect to see) from Turso's
    /// Hrana protocol. INTRADA-API-2 in production hit the first one.
    #[test]
    fn classifies_known_transient_errors() {
        for example in [
            // Live example from INTRADA-API-2 (2026-05-09).
            "Hrana: `http error: `connection closed before message completed``",
            // Capitalisation insensitivity.
            "CONNECTION CLOSED BEFORE MESSAGE COMPLETED",
            // Other Turso-side transient symptoms we've seen referenced
            // in the heartbeat code (state.rs).
            "stream not found: f238e949:019e0bd5-b8fb-7eb0-ade5-43aa668a9f23",
            "Connection reset by peer",
            "Connection refused",
            "Broken pipe",
            "operation timed out",
            "request timeout",
            "unexpected end of file",
        ] {
            assert!(
                crate::db::is_transient_db_error(example),
                "expected transient classification for: {example}"
            );
        }
    }

    /// Guard: real bugs (SQL errors, constraint violations, schema
    /// mismatches) MUST NOT be classified as transient. Silently
    /// retrying these would hide real problems behind 6 seconds of
    /// "looks like it's working" before the same panic.
    #[test]
    fn does_not_classify_terminal_errors_as_transient() {
        for example in [
            "near \"FROM\": syntax error",
            "no such table: routines",
            "UNIQUE constraint failed: routines.id",
            "FOREIGN KEY constraint failed",
            "duplicate column name: title",
            "table routines already exists",
        ] {
            assert!(
                !crate::db::is_transient_db_error(example),
                "should NOT classify as transient (would hide a real bug): {example}"
            );
        }
    }
}
