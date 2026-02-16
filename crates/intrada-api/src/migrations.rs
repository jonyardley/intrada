use libsql::Connection;

/// Raw SQL statements for each migration (used by tests that skip libsql_migration).
pub const MIGRATION_SQL: &[&str] = &[
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
    "CREATE TABLE IF NOT EXISTS sessions (
        id TEXT PRIMARY KEY NOT NULL,
        session_notes TEXT,
        started_at TEXT NOT NULL,
        completed_at TEXT NOT NULL,
        total_duration_secs INTEGER NOT NULL,
        completion_status TEXT NOT NULL
    );",
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
    "CREATE INDEX IF NOT EXISTS idx_setlist_entries_session_id ON setlist_entries(session_id);",
];

/// Run migrations directly via SQL (for testing with in-memory databases).
pub async fn run_migrations_sql(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    for sql in MIGRATION_SQL {
        conn.execute(sql, ()).await?;
    }
    Ok(())
}

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

        CREATE INDEX IF NOT EXISTS idx_setlist_entries_session_id ON setlist_entries(session_id);",
    ),
];

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
