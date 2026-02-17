use chrono::{DateTime, Utc};
use libsql::Connection;
use serde::{Deserialize, Serialize};

use intrada_core::domain::session::{CompletionStatus, EntryStatus, PracticeSession, SetlistEntry};

use crate::error::ApiError;

/// Request body for saving a new practice session.
/// Like PracticeSession but without `id` (server generates it).
/// Entry IDs come from the client.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SaveSessionRequest {
    pub entries: Vec<SaveSessionEntry>,
    pub session_notes: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub total_duration_secs: u64,
    pub completion_status: CompletionStatus,
}

/// Entry within a SaveSessionRequest.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SaveSessionEntry {
    pub id: String,
    pub item_id: String,
    pub item_title: String,
    pub item_type: String,
    pub position: usize,
    pub duration_secs: u64,
    pub status: EntryStatus,
    pub notes: Option<String>,
    #[serde(default)]
    pub score: Option<u8>,
}

fn completion_status_to_str(status: &CompletionStatus) -> &'static str {
    match status {
        CompletionStatus::Completed => "Completed",
        CompletionStatus::EndedEarly => "EndedEarly",
    }
}

fn completion_status_from_str(s: &str) -> Result<CompletionStatus, ApiError> {
    match s {
        "Completed" => Ok(CompletionStatus::Completed),
        "EndedEarly" => Ok(CompletionStatus::EndedEarly),
        other => Err(ApiError::Internal(format!(
            "Invalid completion_status: {other}"
        ))),
    }
}

fn entry_status_to_str(status: &EntryStatus) -> &'static str {
    match status {
        EntryStatus::Completed => "Completed",
        EntryStatus::Skipped => "Skipped",
        EntryStatus::NotAttempted => "NotAttempted",
    }
}

fn entry_status_from_str(s: &str) -> Result<EntryStatus, ApiError> {
    match s {
        "Completed" => Ok(EntryStatus::Completed),
        "Skipped" => Ok(EntryStatus::Skipped),
        "NotAttempted" => Ok(EntryStatus::NotAttempted),
        other => Err(ApiError::Internal(format!("Invalid entry_status: {other}"))),
    }
}

fn row_to_entry(row: &libsql::Row) -> Result<SetlistEntry, ApiError> {
    let id: String = row.get(0).map_err(|e| ApiError::Internal(e.to_string()))?;
    // skip session_id (index 1)
    let item_id: String = row.get(2).map_err(|e| ApiError::Internal(e.to_string()))?;
    let item_title: String = row.get(3).map_err(|e| ApiError::Internal(e.to_string()))?;
    let item_type: String = row.get(4).map_err(|e| ApiError::Internal(e.to_string()))?;
    let position: i64 = row.get(5).map_err(|e| ApiError::Internal(e.to_string()))?;
    let duration_secs: i64 = row.get(6).map_err(|e| ApiError::Internal(e.to_string()))?;
    let status_str: String = row.get(7).map_err(|e| ApiError::Internal(e.to_string()))?;
    let notes: Option<String> = row.get(8).map_err(|e| ApiError::Internal(e.to_string()))?;
    let score_raw: Option<i64> = row.get(9).map_err(|e| ApiError::Internal(e.to_string()))?;
    let score = score_raw.map(|s| s as u8);

    Ok(SetlistEntry {
        id,
        item_id,
        item_title,
        item_type,
        position: position as usize,
        duration_secs: duration_secs as u64,
        status: entry_status_from_str(&status_str)?,
        notes,
        score,
    })
}

/// Fetch entries for a session, ordered by position.
async fn fetch_entries(conn: &Connection, session_id: &str) -> Result<Vec<SetlistEntry>, ApiError> {
    let mut rows = conn
        .query(
            "SELECT id, session_id, item_id, item_title, item_type, position, duration_secs, status, notes, score
             FROM setlist_entries WHERE session_id = ?1 ORDER BY position ASC",
            libsql::params![session_id],
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

/// Parse a session row (without entries) into partial PracticeSession fields.
fn row_to_session_without_entries(row: &libsql::Row) -> Result<PracticeSession, ApiError> {
    let id: String = row.get(0).map_err(|e| ApiError::Internal(e.to_string()))?;
    let session_notes: Option<String> =
        row.get(1).map_err(|e| ApiError::Internal(e.to_string()))?;
    let started_at_str: String = row.get(2).map_err(|e| ApiError::Internal(e.to_string()))?;
    let completed_at_str: String = row.get(3).map_err(|e| ApiError::Internal(e.to_string()))?;
    let total_duration_secs: i64 = row.get(4).map_err(|e| ApiError::Internal(e.to_string()))?;
    let completion_status_str: String =
        row.get(5).map_err(|e| ApiError::Internal(e.to_string()))?;

    let started_at: DateTime<Utc> = started_at_str
        .parse()
        .map_err(|e| ApiError::Internal(format!("Invalid started_at: {e}")))?;
    let completed_at: DateTime<Utc> = completed_at_str
        .parse()
        .map_err(|e| ApiError::Internal(format!("Invalid completed_at: {e}")))?;

    Ok(PracticeSession {
        id,
        entries: vec![], // filled separately
        session_notes,
        started_at,
        completed_at,
        total_duration_secs: total_duration_secs as u64,
        completion_status: completion_status_from_str(&completion_status_str)?,
    })
}

pub async fn list_sessions(conn: &Connection) -> Result<Vec<PracticeSession>, ApiError> {
    let mut rows = conn
        .query(
            "SELECT id, session_notes, started_at, completed_at, total_duration_secs, completion_status
             FROM sessions ORDER BY started_at DESC",
            (),
        )
        .await?;

    let mut sessions = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        let mut session = row_to_session_without_entries(&row)?;
        session.entries = fetch_entries(conn, &session.id).await?;
        sessions.push(session);
    }
    Ok(sessions)
}

pub async fn get_session(conn: &Connection, id: &str) -> Result<Option<PracticeSession>, ApiError> {
    let mut rows = conn
        .query(
            "SELECT id, session_notes, started_at, completed_at, total_duration_secs, completion_status
             FROM sessions WHERE id = ?1",
            libsql::params![id],
        )
        .await?;

    match rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        Some(row) => {
            let mut session = row_to_session_without_entries(&row)?;
            session.entries = fetch_entries(conn, &session.id).await?;
            Ok(Some(session))
        }
        None => Ok(None),
    }
}

pub async fn insert_session(
    conn: &Connection,
    input: &SaveSessionRequest,
) -> Result<PracticeSession, ApiError> {
    let id = ulid::Ulid::new().to_string();

    let started_at_str = input.started_at.to_rfc3339();
    let completed_at_str = input.completed_at.to_rfc3339();
    let completion_status_str = completion_status_to_str(&input.completion_status);

    // Use a transaction to insert session + entries atomically
    conn.execute("BEGIN", ()).await?;

    let result: Result<PracticeSession, ApiError> = async {
        // Insert session row
        conn.execute(
            "INSERT INTO sessions (id, session_notes, started_at, completed_at, total_duration_secs, completion_status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            libsql::params![
                id.as_str(),
                input.session_notes.as_deref(),
                started_at_str.as_str(),
                completed_at_str.as_str(),
                input.total_duration_secs as i64,
                completion_status_str
            ],
        )
        .await?;

        // Insert each setlist entry
        let mut entries = Vec::with_capacity(input.entries.len());
        for entry in &input.entries {
            let status_str = entry_status_to_str(&entry.status);
            let score_val: Option<i64> = entry.score.map(|s| s as i64);
            conn.execute(
                "INSERT INTO setlist_entries (id, session_id, item_id, item_title, item_type, position, duration_secs, status, notes, score)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                libsql::params![
                    entry.id.as_str(),
                    id.as_str(),
                    entry.item_id.as_str(),
                    entry.item_title.as_str(),
                    entry.item_type.as_str(),
                    entry.position as i64,
                    entry.duration_secs as i64,
                    status_str,
                    entry.notes.as_deref(),
                    score_val
                ],
            )
            .await?;

            entries.push(SetlistEntry {
                id: entry.id.clone(),
                item_id: entry.item_id.clone(),
                item_title: entry.item_title.clone(),
                item_type: entry.item_type.clone(),
                position: entry.position,
                duration_secs: entry.duration_secs,
                status: entry.status.clone(),
                notes: entry.notes.clone(),
                score: entry.score,
            });
        }

        Ok(PracticeSession {
            id: id.clone(),
            entries,
            session_notes: input.session_notes.clone(),
            started_at: input.started_at,
            completed_at: input.completed_at,
            total_duration_secs: input.total_duration_secs,
            completion_status: input.completion_status.clone(),
        })
    }
    .await;

    match result {
        Ok(session) => {
            conn.execute("COMMIT", ()).await?;
            Ok(session)
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", ()).await;
            Err(e)
        }
    }
}

pub async fn delete_session(conn: &Connection, id: &str) -> Result<bool, ApiError> {
    // setlist_entries will be cascade-deleted due to ON DELETE CASCADE
    let rows_affected = conn
        .execute("DELETE FROM sessions WHERE id = ?1", libsql::params![id])
        .await?;

    Ok(rows_affected > 0)
}
