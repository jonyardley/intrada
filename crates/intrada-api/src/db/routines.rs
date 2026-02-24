use chrono::{DateTime, Utc};
use libsql::Connection;
use serde::{Deserialize, Serialize};

use intrada_core::domain::routine::{Routine, RoutineEntry};

use super::col;
use crate::error::ApiError;

/// Request body for creating a new routine.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateRoutineRequest {
    pub name: String,
    pub entries: Vec<CreateRoutineEntry>,
}

/// Entry within a CreateRoutineRequest.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateRoutineEntry {
    pub item_id: String,
    pub item_title: String,
    pub item_type: String,
}

/// Request body for updating an existing routine.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateRoutineRequest {
    pub name: String,
    pub entries: Vec<CreateRoutineEntry>,
}

/// Column list for routine_entries SELECTs.
const ENTRY_COLUMNS: &str = "id, item_id, item_title, item_type, position";

/// Parse 5 sequential entry columns (see [`ENTRY_COLUMNS`]) starting at `offset`.
fn parse_entry_cols(row: &libsql::Row, offset: i32) -> Result<RoutineEntry, ApiError> {
    let id: String = col!(row, offset)?;
    let item_id: String = col!(row, offset + 1)?;
    let item_title: String = col!(row, offset + 2)?;
    let item_type: String = col!(row, offset + 3)?;
    let position: i64 = col!(row, offset + 4)?;

    Ok(RoutineEntry {
        id,
        item_id,
        item_title,
        item_type,
        position: position as usize,
    })
}

/// Parse an entry row into a RoutineEntry (standalone SELECT, offset 0).
fn row_to_entry(row: &libsql::Row) -> Result<RoutineEntry, ApiError> {
    parse_entry_cols(row, 0)
}

/// Fetch entries for a routine, ordered by position.
async fn fetch_entries(conn: &Connection, routine_id: &str) -> Result<Vec<RoutineEntry>, ApiError> {
    let mut rows = conn
        .query(
            &format!(
                "SELECT {ENTRY_COLUMNS} FROM routine_entries WHERE routine_id = ?1 ORDER BY position ASC"
            ),
            libsql::params![routine_id],
        )
        .await?;

    let mut entries = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        entries.push(row_to_entry(&row)?);
    }
    Ok(entries)
}

/// Parse a routine row (without entries) into a partial Routine.
///
/// Expects columns: id, name, created_at, updated_at
fn row_to_routine_without_entries(row: &libsql::Row) -> Result<Routine, ApiError> {
    let id: String = col!(row, 0)?;
    let name: String = col!(row, 1)?;
    let created_at_str: String = col!(row, 2)?;
    let updated_at_str: String = col!(row, 3)?;

    let created_at: DateTime<Utc> = created_at_str
        .parse()
        .map_err(|e| ApiError::Internal(format!("Invalid created_at: {e}")))?;
    let updated_at: DateTime<Utc> = updated_at_str
        .parse()
        .map_err(|e| ApiError::Internal(format!("Invalid updated_at: {e}")))?;

    Ok(Routine {
        id,
        name,
        entries: vec![], // filled separately
        created_at,
        updated_at,
    })
}

/// Parse an entry from a LEFT JOIN row where entry columns start at `offset`.
/// Returns `None` when the entry id column is NULL (routine has no entries).
fn joined_row_to_entry(row: &libsql::Row, offset: i32) -> Result<Option<RoutineEntry>, ApiError> {
    let entry_id: Option<String> = col!(row, offset)?;
    if entry_id.is_none() {
        return Ok(None);
    }
    Ok(Some(parse_entry_cols(row, offset)?))
}

pub async fn list_routines(conn: &Connection, user_id: &str) -> Result<Vec<Routine>, ApiError> {
    // Single query with LEFT JOIN replaces N+1 (1 routine query + N entry queries).
    let mut rows = conn
        .query(
            "SELECT r.id, r.name, r.created_at, r.updated_at,
                    e.id, e.item_id, e.item_title, e.item_type, e.position
             FROM routines r
             LEFT JOIN routine_entries e ON r.id = e.routine_id
             WHERE r.user_id = ?1
             ORDER BY r.created_at ASC, r.id, e.position ASC",
            libsql::params![user_id],
        )
        .await?;

    // Group joined rows by routine id, preserving order
    let mut routines: Vec<Routine> = Vec::new();
    let mut last_routine_id: Option<String> = None;

    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        let routine_id: String = col!(row, 0)?;

        // If this is a new routine, parse the routine columns and push
        if last_routine_id.as_ref() != Some(&routine_id) {
            let routine = row_to_routine_without_entries(&row)?;
            routines.push(routine);
            last_routine_id = Some(routine_id);
        }

        if let Some(entry) = joined_row_to_entry(&row, 4)? {
            if let Some(current) = routines.last_mut() {
                current.entries.push(entry);
            }
        }
    }

    Ok(routines)
}

pub async fn get_routine(
    conn: &Connection,
    id: &str,
    user_id: &str,
) -> Result<Option<Routine>, ApiError> {
    let mut rows = conn
        .query(
            "SELECT id, name, created_at, updated_at FROM routines WHERE id = ?1 AND user_id = ?2",
            libsql::params![id, user_id],
        )
        .await?;

    match rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        Some(row) => {
            let mut routine = row_to_routine_without_entries(&row)?;
            routine.entries = fetch_entries(conn, &routine.id).await?;
            Ok(Some(routine))
        }
        None => Ok(None),
    }
}

pub async fn insert_routine(
    conn: &Connection,
    user_id: &str,
    input: &CreateRoutineRequest,
) -> Result<Routine, ApiError> {
    let id = ulid::Ulid::new().to_string();
    let now = Utc::now();
    let created_at_str = now.to_rfc3339();
    let updated_at_str = now.to_rfc3339();

    // Use a transaction to insert routine + entries atomically
    conn.execute("BEGIN", ()).await?;

    let result: Result<Routine, ApiError> = async {
        // Insert routine row
        conn.execute(
            "INSERT INTO routines (id, name, created_at, updated_at, user_id)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            libsql::params![
                id.as_str(),
                input.name.as_str(),
                created_at_str.as_str(),
                updated_at_str.as_str(),
                user_id
            ],
        )
        .await?;

        // Insert each routine entry
        let mut entries = Vec::with_capacity(input.entries.len());
        for (position, entry) in input.entries.iter().enumerate() {
            let entry_id = ulid::Ulid::new().to_string();
            conn.execute(
                "INSERT INTO routine_entries (id, routine_id, item_id, item_title, item_type, position)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                libsql::params![
                    entry_id.as_str(),
                    id.as_str(),
                    entry.item_id.as_str(),
                    entry.item_title.as_str(),
                    entry.item_type.as_str(),
                    position as i64
                ],
            )
            .await?;

            entries.push(RoutineEntry {
                id: entry_id,
                item_id: entry.item_id.clone(),
                item_title: entry.item_title.clone(),
                item_type: entry.item_type.clone(),
                position,
            });
        }

        Ok(Routine {
            id: id.clone(),
            name: input.name.clone(),
            entries,
            created_at: now,
            updated_at: now,
        })
    }
    .await;

    match result {
        Ok(routine) => {
            conn.execute("COMMIT", ()).await?;
            Ok(routine)
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", ()).await;
            Err(e)
        }
    }
}

pub async fn update_routine(
    conn: &Connection,
    id: &str,
    user_id: &str,
    input: &UpdateRoutineRequest,
) -> Result<Option<Routine>, ApiError> {
    // Check if routine exists
    let existing = get_routine(conn, id, user_id).await?;
    let existing = match existing {
        Some(r) => r,
        None => return Ok(None),
    };

    let now = Utc::now();
    let updated_at_str = now.to_rfc3339();

    conn.execute("BEGIN", ()).await?;

    let result: Result<Routine, ApiError> = async {
        // Update routine name and updated_at
        conn.execute(
            "UPDATE routines SET name = ?1, updated_at = ?2 WHERE id = ?3 AND user_id = ?4",
            libsql::params![input.name.as_str(), updated_at_str.as_str(), id, user_id],
        )
        .await?;

        // Delete all existing entries
        conn.execute(
            "DELETE FROM routine_entries WHERE routine_id = ?1",
            libsql::params![id],
        )
        .await?;

        // Insert new entries
        let mut entries = Vec::with_capacity(input.entries.len());
        for (position, entry) in input.entries.iter().enumerate() {
            let entry_id = ulid::Ulid::new().to_string();
            conn.execute(
                "INSERT INTO routine_entries (id, routine_id, item_id, item_title, item_type, position)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                libsql::params![
                    entry_id.as_str(),
                    id,
                    entry.item_id.as_str(),
                    entry.item_title.as_str(),
                    entry.item_type.as_str(),
                    position as i64
                ],
            )
            .await?;

            entries.push(RoutineEntry {
                id: entry_id,
                item_id: entry.item_id.clone(),
                item_title: entry.item_title.clone(),
                item_type: entry.item_type.clone(),
                position,
            });
        }

        Ok(Routine {
            id: id.to_string(),
            name: input.name.clone(),
            entries,
            created_at: existing.created_at,
            updated_at: now,
        })
    }
    .await;

    match result {
        Ok(routine) => {
            conn.execute("COMMIT", ()).await?;
            Ok(Some(routine))
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", ()).await;
            Err(e)
        }
    }
}

pub async fn delete_routine(conn: &Connection, id: &str, user_id: &str) -> Result<bool, ApiError> {
    // Verify ownership first — only delete entries if the routine belongs to this user.
    let rows_affected = conn
        .execute(
            "DELETE FROM routines WHERE id = ?1 AND user_id = ?2",
            libsql::params![id, user_id],
        )
        .await?;

    if rows_affected == 0 {
        return Ok(false);
    }

    // Now safe to delete entries — we confirmed ownership above.
    // SQLite only enforces ON DELETE CASCADE when PRAGMA foreign_keys = ON,
    // which is off by default, so we delete explicitly.
    conn.execute(
        "DELETE FROM routine_entries WHERE routine_id = ?1",
        libsql::params![id],
    )
    .await?;

    Ok(true)
}
