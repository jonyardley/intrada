use chrono::{DateTime, Utc};
use libsql::Connection;
use serde::{Deserialize, Serialize};

use intrada_core::domain::routine::{Routine, RoutineEntry};

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

fn row_to_entry(row: &libsql::Row) -> Result<RoutineEntry, ApiError> {
    let id: String = row.get(0).map_err(|e| ApiError::Internal(e.to_string()))?;
    // skip routine_id (index 1)
    let item_id: String = row.get(2).map_err(|e| ApiError::Internal(e.to_string()))?;
    let item_title: String = row.get(3).map_err(|e| ApiError::Internal(e.to_string()))?;
    let item_type: String = row.get(4).map_err(|e| ApiError::Internal(e.to_string()))?;
    let position: i64 = row.get(5).map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(RoutineEntry {
        id,
        item_id,
        item_title,
        item_type,
        position: position as usize,
    })
}

/// Fetch entries for a routine, ordered by position.
async fn fetch_entries(conn: &Connection, routine_id: &str) -> Result<Vec<RoutineEntry>, ApiError> {
    let mut rows = conn
        .query(
            "SELECT id, routine_id, item_id, item_title, item_type, position
             FROM routine_entries WHERE routine_id = ?1 ORDER BY position ASC",
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
fn row_to_routine_without_entries(row: &libsql::Row) -> Result<Routine, ApiError> {
    let id: String = row.get(0).map_err(|e| ApiError::Internal(e.to_string()))?;
    let name: String = row.get(1).map_err(|e| ApiError::Internal(e.to_string()))?;
    let created_at_str: String = row.get(2).map_err(|e| ApiError::Internal(e.to_string()))?;
    let updated_at_str: String = row.get(3).map_err(|e| ApiError::Internal(e.to_string()))?;

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

pub async fn list_routines(conn: &Connection) -> Result<Vec<Routine>, ApiError> {
    let mut rows = conn
        .query(
            "SELECT id, name, created_at, updated_at FROM routines ORDER BY created_at ASC",
            (),
        )
        .await?;

    let mut routines = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        let mut routine = row_to_routine_without_entries(&row)?;
        routine.entries = fetch_entries(conn, &routine.id).await?;
        routines.push(routine);
    }
    Ok(routines)
}

pub async fn get_routine(conn: &Connection, id: &str) -> Result<Option<Routine>, ApiError> {
    let mut rows = conn
        .query(
            "SELECT id, name, created_at, updated_at FROM routines WHERE id = ?1",
            libsql::params![id],
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
            "INSERT INTO routines (id, name, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4)",
            libsql::params![
                id.as_str(),
                input.name.as_str(),
                created_at_str.as_str(),
                updated_at_str.as_str()
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
    input: &UpdateRoutineRequest,
) -> Result<Option<Routine>, ApiError> {
    // Check if routine exists
    let existing = get_routine(conn, id).await?;
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
            "UPDATE routines SET name = ?1, updated_at = ?2 WHERE id = ?3",
            libsql::params![input.name.as_str(), updated_at_str.as_str(), id],
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

pub async fn delete_routine(conn: &Connection, id: &str) -> Result<bool, ApiError> {
    // Explicitly delete entries first — SQLite only enforces ON DELETE CASCADE
    // when PRAGMA foreign_keys = ON, which is off by default.
    conn.execute(
        "DELETE FROM routine_entries WHERE routine_id = ?1",
        libsql::params![id],
    )
    .await?;

    let rows_affected = conn
        .execute("DELETE FROM routines WHERE id = ?1", libsql::params![id])
        .await?;

    Ok(rows_affected > 0)
}
