use intrada_api::migrations;

/// Helper: create a fresh local SQLite database and return a connection.
async fn fresh_db() -> libsql::Connection {
    let tmp_dir = std::env::temp_dir();
    let db_path = tmp_dir.join(format!("intrada_migration_test_{}.db", ulid::Ulid::new()));

    let db = libsql::Builder::new_local(&db_path)
        .build()
        .await
        .expect("Failed to build test database");

    db.connect().expect("Failed to connect to test database")
}

/// Test the production migration path (libsql_migration) creates all expected tables.
#[tokio::test]
async fn run_migrations_creates_all_tables() {
    let conn = fresh_db().await;

    migrations::run_migrations(&conn)
        .await
        .expect("run_migrations should succeed on fresh database");

    // Query sqlite_master for all user-created tables (excluding libsql_migration internals).
    let mut rows = conn
        .query(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE '\\__%' ESCAPE '\\' ORDER BY name",
            (),
        )
        .await
        .expect("Failed to query sqlite_master");

    let mut tables: Vec<String> = Vec::new();
    while let Some(row) = rows.next().await.expect("Failed to read row") {
        let name: String = row.get(0).expect("Failed to get table name");
        tables.push(name);
    }

    // Legacy pieces/exercises tables were dropped in migrations 0032/0033.
    assert!(
        !tables.contains(&"pieces".to_string()),
        "Legacy 'pieces' table should have been dropped. Found: {tables:?}"
    );
    assert!(
        !tables.contains(&"exercises".to_string()),
        "Legacy 'exercises' table should have been dropped. Found: {tables:?}"
    );
    assert!(
        tables.contains(&"sessions".to_string()),
        "Missing 'sessions' table. Found: {tables:?}"
    );
    assert!(
        tables.contains(&"setlist_entries".to_string()),
        "Missing 'setlist_entries' table. Found: {tables:?}"
    );
}

/// Test that the score column exists on setlist_entries after migrations.
#[tokio::test]
async fn run_migrations_adds_score_column() {
    let conn = fresh_db().await;

    migrations::run_migrations(&conn)
        .await
        .expect("run_migrations should succeed");

    // PRAGMA table_info returns one row per column.
    let mut rows = conn
        .query("PRAGMA table_info(setlist_entries)", ())
        .await
        .expect("Failed to query table_info");

    let mut columns: Vec<String> = Vec::new();
    while let Some(row) = rows.next().await.expect("Failed to read row") {
        let name: String = row.get(1).expect("Failed to get column name");
        columns.push(name);
    }

    assert!(
        columns.contains(&"score".to_string()),
        "Missing 'score' column on setlist_entries. Found: {columns:?}"
    );
}

/// Test that running migrations twice is idempotent (no errors on second run).
#[tokio::test]
async fn run_migrations_is_idempotent() {
    let conn = fresh_db().await;

    migrations::run_migrations(&conn)
        .await
        .expect("First run should succeed");

    migrations::run_migrations(&conn)
        .await
        .expect("Second run should succeed (idempotent)");
}

/// Test that the index on setlist_entries.session_id exists after migrations.
#[tokio::test]
async fn run_migrations_creates_index() {
    let conn = fresh_db().await;

    migrations::run_migrations(&conn)
        .await
        .expect("run_migrations should succeed");

    let mut rows = conn
        .query(
            "SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='setlist_entries'",
            (),
        )
        .await
        .expect("Failed to query indexes");

    let mut indexes: Vec<String> = Vec::new();
    while let Some(row) = rows.next().await.expect("Failed to read row") {
        let name: String = row.get(0).expect("Failed to get index name");
        indexes.push(name);
    }

    assert!(
        indexes.contains(&"idx_setlist_entries_session_id".to_string()),
        "Missing index 'idx_setlist_entries_session_id'. Found: {indexes:?}"
    );
}
