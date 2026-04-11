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
];

/// Run migrations via libsql_migration (production path — tracks applied state).
pub async fn run_migrations(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    for (id, sql) in MIGRATIONS {
        let result = libsql_migration::content::migrate(conn, id.to_string(), sql.to_string())
            .await
            .map_err(|e| format!("Migration {id} failed: {e}"))?;

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
}
