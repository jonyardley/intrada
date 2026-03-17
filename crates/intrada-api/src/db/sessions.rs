use std::collections::HashMap;

use chrono::{DateTime, Utc};
use libsql::Connection;
use serde::{Deserialize, Serialize};

use intrada_core::domain::item::ItemKind;
use intrada_core::domain::session::{
    CompletionStatus, EntryStatus, PracticeSession, RepAction, SetlistEntry,
};

use super::{col, item_kind_from_str, item_kind_to_str};
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
    #[serde(default)]
    pub session_intention: Option<String>,
}

/// Entry within a SaveSessionRequest.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SaveSessionEntry {
    pub id: String,
    pub item_id: String,
    pub item_title: String,
    pub item_type: ItemKind,
    pub position: usize,
    pub duration_secs: u64,
    pub status: EntryStatus,
    pub notes: Option<String>,
    #[serde(default)]
    pub score: Option<u8>,
    #[serde(default)]
    pub intention: Option<String>,
    #[serde(default)]
    pub rep_target: Option<u8>,
    #[serde(default)]
    pub rep_count: Option<u8>,
    #[serde(default)]
    pub rep_target_reached: Option<bool>,
    #[serde(default)]
    pub rep_history: Option<Vec<RepAction>>,
    #[serde(default)]
    pub planned_duration_secs: Option<u32>,
    #[serde(default)]
    pub achieved_tempo: Option<u16>,
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

/// Parse rep_history JSON from the database.
///
/// Handles both the legacy integer format (`[-1, 1, 1]`) and the current
/// string format (`["Missed", "Success", "Success"]`). Legacy data was written
/// when RepAction used `serde_repr` with `#[repr(i8)]`.
fn parse_rep_history(json_str: &str) -> Result<Vec<RepAction>, ApiError> {
    // Try the current string format first (most common going forward)
    if let Ok(actions) = serde_json::from_str::<Vec<RepAction>>(json_str) {
        return Ok(actions);
    }

    // Fall back to legacy integer format: -1 = Missed, 1 = Success
    let integers: Vec<i8> = serde_json::from_str(json_str)
        .map_err(|e| ApiError::Internal(format!("Invalid rep_history JSON: {e}")))?;

    integers
        .into_iter()
        .map(|v| match v {
            -1 => Ok(RepAction::Missed),
            1 => Ok(RepAction::Success),
            other => Err(ApiError::Internal(format!(
                "Invalid legacy rep_history value: {other}"
            ))),
        })
        .collect()
}

/// Column list for setlist_entries SELECTs.
const ENTRY_COLUMNS: &str = "id, item_id, item_title, item_type, position, duration_secs, status, notes, score, intention, rep_target, rep_count, rep_target_reached, rep_history, planned_duration_secs, achieved_tempo";

/// Subquery to select session IDs for a user. Shared between the parent query
/// and the batch entry query so filter clauses stay in sync (#152).
const SESSION_IDS_FOR_USER: &str = "SELECT id FROM sessions WHERE user_id = ?1";

/// Parse an entry row into a SetlistEntry (columns 0–14 matching [`ENTRY_COLUMNS`]).
fn row_to_entry(row: &libsql::Row) -> Result<SetlistEntry, ApiError> {
    let id: String = col!(row, 0)?;
    let item_id: String = col!(row, 1)?;
    let item_title: String = col!(row, 2)?;
    let item_type_str: String = col!(row, 3)?;
    let item_type = item_kind_from_str(&item_type_str)?;
    let position: i64 = col!(row, 4)?;
    let duration_secs: i64 = col!(row, 5)?;
    let status_str: String = col!(row, 6)?;
    let notes: Option<String> = col!(row, 7)?;
    let score: Option<i64> = col!(row, 8)?;
    let intention: Option<String> = col!(row, 9)?;
    let rep_target: Option<i64> = col!(row, 10)?;
    let rep_count: Option<i64> = col!(row, 11)?;
    let rep_target_reached: Option<i64> = col!(row, 12)?;
    let rep_history_raw: Option<String> = col!(row, 13)?;
    let rep_history = match rep_history_raw {
        Some(json_str) => Some(parse_rep_history(&json_str)?),
        None => None,
    };
    let planned_duration_secs_raw: Option<i64> = col!(row, 14)?;
    let planned_duration_secs = planned_duration_secs_raw.map(|v| v as u32);
    let achieved_tempo_raw: Option<i64> = col!(row, 15)?;
    let achieved_tempo = achieved_tempo_raw.map(|v| v as u16);

    Ok(SetlistEntry {
        id,
        item_id,
        item_title,
        item_type,
        position: position as usize,
        duration_secs: duration_secs as u64,
        status: entry_status_from_str(&status_str)?,
        notes,
        score: score.map(|s| s as u8),
        intention,
        rep_target: rep_target.map(|v| v as u8),
        rep_count: rep_count.map(|v| v as u8),
        rep_target_reached: rep_target_reached.map(|v| v != 0),
        rep_history,
        planned_duration_secs,
        achieved_tempo,
    })
}

/// Fetch entries for a session, ordered by position.
async fn fetch_entries(conn: &Connection, session_id: &str) -> Result<Vec<SetlistEntry>, ApiError> {
    let mut rows = conn
        .query(
            &format!(
                "SELECT {ENTRY_COLUMNS} FROM setlist_entries WHERE session_id = ?1 ORDER BY position ASC"
            ),
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
///
/// Expects columns: id, session_notes, started_at, completed_at,
///   total_duration_secs, completion_status, session_intention
fn row_to_session_without_entries(row: &libsql::Row) -> Result<PracticeSession, ApiError> {
    let id: String = col!(row, 0)?;
    let session_notes: Option<String> = col!(row, 1)?;
    let started_at_str: String = col!(row, 2)?;
    let completed_at_str: String = col!(row, 3)?;
    let total_duration_secs: i64 = col!(row, 4)?;
    let completion_status_str: String = col!(row, 5)?;
    let session_intention: Option<String> = col!(row, 6)?;

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
        session_intention,
    })
}

pub async fn list_sessions(
    conn: &Connection,
    user_id: &str,
) -> Result<Vec<PracticeSession>, ApiError> {
    // Query 1: all sessions for this user.
    let mut rows = conn
        .query(
            "SELECT id, session_notes, started_at, completed_at,
                    total_duration_secs, completion_status, session_intention
             FROM sessions WHERE user_id = ?1
             ORDER BY started_at DESC",
            libsql::params![user_id],
        )
        .await?;

    let mut sessions: Vec<PracticeSession> = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        sessions.push(row_to_session_without_entries(&row)?);
    }

    if sessions.is_empty() {
        return Ok(sessions);
    }

    // Query 2: all entries for those sessions in one batch.
    // session_id is appended after ENTRY_COLUMNS so row_to_entry reads columns 0–14.
    let mut entry_rows = conn
        .query(
            &format!(
                "SELECT {ENTRY_COLUMNS}, session_id FROM setlist_entries
                 WHERE session_id IN ({SESSION_IDS_FOR_USER})
                 ORDER BY session_id, position ASC"
            ),
            libsql::params![user_id],
        )
        .await?;

    let mut entries_by_session: HashMap<String, Vec<SetlistEntry>> = HashMap::new();
    while let Some(row) = entry_rows
        .next()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
    {
        let entry = row_to_entry(&row)?;
        let session_id: String = col!(row, 16)?;
        entries_by_session
            .entry(session_id)
            .or_default()
            .push(entry);
    }

    for session in &mut sessions {
        if let Some(entries) = entries_by_session.remove(&session.id) {
            session.entries = entries;
        }
    }

    Ok(sessions)
}

pub async fn get_session(
    conn: &Connection,
    id: &str,
    user_id: &str,
) -> Result<Option<PracticeSession>, ApiError> {
    let mut rows = conn
        .query(
            "SELECT id, session_notes, started_at, completed_at, total_duration_secs, completion_status, session_intention
             FROM sessions WHERE id = ?1 AND user_id = ?2",
            libsql::params![id, user_id],
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
    user_id: &str,
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
            "INSERT INTO sessions (id, session_notes, started_at, completed_at, total_duration_secs, completion_status, user_id, session_intention)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            libsql::params![
                id.as_str(),
                input.session_notes.as_deref(),
                started_at_str.as_str(),
                completed_at_str.as_str(),
                input.total_duration_secs as i64,
                completion_status_str,
                user_id,
                input.session_intention.as_deref()
            ],
        )
        .await?;

        // Insert each setlist entry
        let mut entries = Vec::with_capacity(input.entries.len());
        for entry in &input.entries {
            let status_str = entry_status_to_str(&entry.status);
            let score_val: Option<i64> = entry.score.map(|s| s as i64);
            let rep_target_val: Option<i64> = entry.rep_target.map(|v| v as i64);
            let rep_count_val: Option<i64> = entry.rep_count.map(|v| v as i64);
            let rep_target_reached_val: Option<i64> =
                entry.rep_target_reached.map(|v| if v { 1 } else { 0 });
            let rep_history_json: Option<String> = entry
                .rep_history
                .as_ref()
                .map(serde_json::to_string)
                .transpose()
                .map_err(|e| ApiError::Internal(format!("Failed to serialise rep_history: {e}")))?;
            let planned_duration_secs_val: Option<i64> =
                entry.planned_duration_secs.map(|v| v as i64);
            let achieved_tempo_val: Option<i64> = entry.achieved_tempo.map(|v| v as i64);
            conn.execute(
                "INSERT INTO setlist_entries (id, session_id, item_id, item_title, item_type, position, duration_secs, status, notes, score, intention, rep_target, rep_count, rep_target_reached, rep_history, planned_duration_secs, achieved_tempo)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
                libsql::params![
                    entry.id.as_str(),
                    id.as_str(),
                    entry.item_id.as_str(),
                    entry.item_title.as_str(),
                    item_kind_to_str(&entry.item_type),
                    entry.position as i64,
                    entry.duration_secs as i64,
                    status_str,
                    entry.notes.as_deref(),
                    score_val,
                    entry.intention.as_deref(),
                    rep_target_val,
                    rep_count_val,
                    rep_target_reached_val,
                    rep_history_json.as_deref(),
                    planned_duration_secs_val,
                    achieved_tempo_val
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
                intention: entry.intention.clone(),
                rep_target: entry.rep_target,
                rep_count: entry.rep_count,
                rep_target_reached: entry.rep_target_reached,
                rep_history: entry.rep_history.clone(),
                planned_duration_secs: entry.planned_duration_secs,
                achieved_tempo: entry.achieved_tempo,
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
            session_intention: input.session_intention.clone(),
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

pub async fn delete_session(conn: &Connection, id: &str, user_id: &str) -> Result<bool, ApiError> {
    // Verify ownership first — only delete entries if the session belongs to this user.
    let rows_affected = conn
        .execute(
            "DELETE FROM sessions WHERE id = ?1 AND user_id = ?2",
            libsql::params![id, user_id],
        )
        .await?;

    if rows_affected == 0 {
        return Ok(false);
    }

    // PRAGMA foreign_keys = ON is set on every connection (see AppState::connect),
    // so ON DELETE CASCADE will handle this automatically. We keep the explicit
    // delete as a belt-and-suspenders safety net, matching delete_routine's pattern.
    conn.execute(
        "DELETE FROM setlist_entries WHERE session_id = ?1",
        libsql::params![id],
    )
    .await?;

    Ok(true)
}
