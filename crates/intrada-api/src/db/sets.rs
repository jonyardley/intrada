use std::collections::HashMap;

use chrono::{DateTime, Utc};
use libsql::Connection;
use serde::{Deserialize, Serialize};

use intrada_core::domain::item::ItemKind;
use intrada_core::domain::set::{Set, SetEntry};

use super::{col, item_kind_from_str, item_kind_to_str};
use crate::error::ApiError;

/// Request body for creating a new set.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateSetRequest {
    pub name: String,
    pub entries: Vec<CreateSetEntry>,
}

/// Entry within a CreateSetRequest.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateSetEntry {
    pub item_id: String,
    pub item_title: String,
    pub item_type: ItemKind,
}

/// Request body for updating an existing set.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateSetRequest {
    pub name: String,
    pub entries: Vec<CreateSetEntry>,
}

/// Column list for routine_entries SELECTs.
///
/// Note: the table is still called `routine_entries` in the database
/// — we kept the table name to avoid a migration when renaming the
/// concept from "routine" to "set". See PR for `Routine` → `Set`.
const ENTRY_COLUMNS: &str = "id, item_id, item_title, item_type, position";

/// Subquery to select set IDs for a user. Shared between the parent query
/// and the batch entry query so filter clauses stay in sync (#152).
const SET_IDS_FOR_USER: &str = "SELECT id FROM routines WHERE user_id = ?1";

fn row_to_entry(row: &libsql::Row) -> Result<SetEntry, ApiError> {
    let id: String = col!(row, 0)?;
    let item_id: String = col!(row, 1)?;
    let item_title: String = col!(row, 2)?;
    let item_type_str: String = col!(row, 3)?;
    let item_type = item_kind_from_str(&item_type_str)?;
    let position: i64 = col!(row, 4)?;

    Ok(SetEntry {
        id,
        item_id,
        item_title,
        item_type,
        position: position as usize,
    })
}

/// Fetch entries for a set, ordered by position.
async fn fetch_entries(conn: &Connection, set_id: &str) -> Result<Vec<SetEntry>, ApiError> {
    let mut rows = conn
        .query(
            &format!(
                "SELECT {ENTRY_COLUMNS} FROM routine_entries WHERE routine_id = ?1 ORDER BY position ASC"
            ),
            libsql::params![set_id],
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

/// Parse a set row (without entries) into a partial Set.
///
/// Expects columns: id, name, created_at, updated_at
fn row_to_set_without_entries(row: &libsql::Row) -> Result<Set, ApiError> {
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

    Ok(Set {
        id,
        name,
        entries: vec![], // filled separately
        created_at,
        updated_at,
    })
}

pub async fn list_sets(conn: &Connection, user_id: &str) -> Result<Vec<Set>, ApiError> {
    // Query 1: all sets for this user.
    let mut rows = conn
        .query(
            "SELECT id, name, created_at, updated_at
             FROM routines WHERE user_id = ?1
             ORDER BY created_at ASC",
            libsql::params![user_id],
        )
        .await?;

    let mut sets: Vec<Set> = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        sets.push(row_to_set_without_entries(&row)?);
    }

    if sets.is_empty() {
        return Ok(sets);
    }

    // Query 2: all entries for those sets in one batch.
    // routine_id is appended after ENTRY_COLUMNS so row_to_entry reads columns 0–4.
    let mut entry_rows = conn
        .query(
            &format!(
                "SELECT {ENTRY_COLUMNS}, routine_id FROM routine_entries
                 WHERE routine_id IN ({SET_IDS_FOR_USER})
                 ORDER BY routine_id, position ASC"
            ),
            libsql::params![user_id],
        )
        .await?;

    let mut entries_by_set: HashMap<String, Vec<SetEntry>> = HashMap::new();
    while let Some(row) = entry_rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        let entry = row_to_entry(&row)?;
        let set_id: String = col!(row, 5)?;
        entries_by_set.entry(set_id).or_default().push(entry);
    }

    for set in &mut sets {
        if let Some(entries) = entries_by_set.remove(&set.id) {
            set.entries = entries;
        }
    }

    Ok(sets)
}

pub async fn get_set(conn: &Connection, id: &str, user_id: &str) -> Result<Option<Set>, ApiError> {
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
            let mut set = row_to_set_without_entries(&row)?;
            set.entries = fetch_entries(conn, &set.id).await?;
            Ok(Some(set))
        }
        None => Ok(None),
    }
}

pub async fn insert_set(
    conn: &Connection,
    user_id: &str,
    input: &CreateSetRequest,
) -> Result<Set, ApiError> {
    let id = ulid::Ulid::new().to_string();
    let now = Utc::now();
    let created_at_str = now.to_rfc3339();
    let updated_at_str = now.to_rfc3339();

    // Use a transaction to insert set + entries atomically
    conn.execute("BEGIN", ()).await?;

    let result: Result<Set, ApiError> = async {
        // Insert set row
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

        // Insert each set entry
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
                    item_kind_to_str(&entry.item_type),
                    position as i64
                ],
            )
            .await?;

            entries.push(SetEntry {
                id: entry_id,
                item_id: entry.item_id.clone(),
                item_title: entry.item_title.clone(),
                item_type: entry.item_type.clone(),
                position,
            });
        }

        Ok(Set {
            id: id.clone(),
            name: input.name.clone(),
            entries,
            created_at: now,
            updated_at: now,
        })
    }
    .await;

    match result {
        Ok(set) => {
            conn.execute("COMMIT", ()).await?;
            Ok(set)
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", ()).await;
            Err(e)
        }
    }
}

pub async fn update_set(
    conn: &Connection,
    id: &str,
    user_id: &str,
    input: &UpdateSetRequest,
) -> Result<Option<Set>, ApiError> {
    // Check if set exists
    let existing = get_set(conn, id, user_id).await?;
    let existing = match existing {
        Some(r) => r,
        None => return Ok(None),
    };

    let now = Utc::now();
    let updated_at_str = now.to_rfc3339();

    conn.execute("BEGIN", ()).await?;

    let result: Result<Set, ApiError> = async {
        // Update set name and updated_at
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
                    item_kind_to_str(&entry.item_type),
                    position as i64
                ],
            )
            .await?;

            entries.push(SetEntry {
                id: entry_id,
                item_id: entry.item_id.clone(),
                item_title: entry.item_title.clone(),
                item_type: entry.item_type.clone(),
                position,
            });
        }

        Ok(Set {
            id: id.to_string(),
            name: input.name.clone(),
            entries,
            created_at: existing.created_at,
            updated_at: now,
        })
    }
    .await;

    match result {
        Ok(set) => {
            conn.execute("COMMIT", ()).await?;
            Ok(Some(set))
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", ()).await;
            Err(e)
        }
    }
}

pub async fn delete_set(conn: &Connection, id: &str, user_id: &str) -> Result<bool, ApiError> {
    // Verify ownership first — only delete entries if the set belongs to this user.
    let rows_affected = conn
        .execute(
            "DELETE FROM routines WHERE id = ?1 AND user_id = ?2",
            libsql::params![id, user_id],
        )
        .await?;

    if rows_affected == 0 {
        return Ok(false);
    }

    // Explicit child-row delete — FK cascade is disabled (Turso compatibility),
    // so application code owns the cleanup.
    conn.execute(
        "DELETE FROM routine_entries WHERE routine_id = ?1",
        libsql::params![id],
    )
    .await?;

    Ok(true)
}
