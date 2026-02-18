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
