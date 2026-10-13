// ── Sessions and variations ──────────────────────────────────────────

use chrono::{DateTime, Utc};
use intrada_core::domain::Variation;
use intrada_core::persistence::StoredPracticeSession;
use intrada_core::stored_session::{session_from_stored, session_to_stored, StoredSession};
use intrada_core::PracticeSession;
use rusqlite::{params, Row, Transaction};

use crate::codec::{parse_time, time_text, Unreadable};
use crate::StoreError;

pub(crate) fn load(
    tx: &Transaction,
    unreadable: &mut Unreadable,
) -> Result<Vec<PracticeSession>, StoreError> {
    Ok(
        load_rows(tx, "WHERE deleted_at IS NULL", unreadable, &mut Vec::new())?
            .into_iter()
            .map(|stored| stored.session)
            .collect(),
    )
}

/// Tombstones included, for the sync merge (`specs/icloud-sync.md`).
pub(crate) fn load_with_tombstones(
    tx: &Transaction,
    unreadable: &mut Unreadable,
    skipped: &mut Vec<String>,
) -> Result<Vec<StoredPracticeSession>, StoreError> {
    load_rows(tx, "", unreadable, skipped)
}

fn load_rows(
    tx: &Transaction,
    filter: &str,
    unreadable: &mut Unreadable,
    skipped: &mut Vec<String>,
) -> Result<Vec<StoredPracticeSession>, StoreError> {
    let mut stmt = tx.prepare(&format!(
        "SELECT * FROM session {filter} ORDER BY completed_at DESC"
    ))?;
    let mut rows = stmt.query([])?;
    let mut sessions = Vec::new();
    while let Some(row) = rows.next()? {
        let id: String = row.get("id")?;
        let read = stored(row)
            .map_err(|e| e.to_string())
            .and_then(|s| session_from_stored(&s).map_err(|e| e.to_string()))
            .and_then(|read| Ok((read, times(row)?)));
        match read {
            Ok((read, (updated_at, deleted_at))) => {
                unreadable.extend(read.unreadable);
                sessions.push(StoredPracticeSession {
                    session: read.session,
                    updated_at,
                    deleted_at,
                });
            }
            Err(reason) => {
                unreadable.push(format!("session {id} skipped: {reason}"));
                skipped.push(id);
            }
        }
    }
    Ok(sessions)
}

type Times = (DateTime<Utc>, Option<DateTime<Utc>>);

fn times(row: &Row) -> Result<Times, String> {
    let time = |text: String, name: &str| {
        parse_time(&text).ok_or_else(|| format!("{name} {text:?} is not a time"))
    };
    let updated_at: String = row.get("updated_at").map_err(|e| e.to_string())?;
    let deleted_at: Option<String> = row.get("deleted_at").map_err(|e| e.to_string())?;
    Ok((
        time(updated_at, "updated_at")?,
        deleted_at.map(|t| time(t, "deleted_at")).transpose()?,
    ))
}

fn stored(row: &Row) -> rusqlite::Result<StoredSession> {
    Ok(StoredSession {
        id: row.get("id")?,
        started_at: row.get("started_at")?,
        completed_at: row.get("completed_at")?,
        total_duration_secs: row.get("total_duration_secs")?,
        completion_status: row.get("completion_status")?,
        session_notes: row.get("session_notes")?,
        entries: row.get("entries")?,
        session_score: row.get("session_score")?,
        capture_version: row.get("capture_version")?,
    })
}

/// Insert or update by id. A new row changed when it was completed; an
/// existing one keeps its change time and its delete, so a later save never
/// brings a deleted session back (#2581).
pub(crate) fn save(tx: &Transaction, session: &PracticeSession) -> Result<(), StoreError> {
    write(tx, session, &session.completed_at, None, "")
}

/// The merge's copy, both times written exactly as they arrived.
pub(crate) fn put_merged(
    tx: &Transaction,
    stored: &StoredPracticeSession,
) -> Result<(), StoreError> {
    write(
        tx,
        &stored.session,
        &stored.updated_at,
        stored.deleted_at.as_ref(),
        ", updated_at = excluded.updated_at, deleted_at = excluded.deleted_at",
    )
}

fn write(
    tx: &Transaction,
    session: &PracticeSession,
    updated_at: &DateTime<Utc>,
    deleted_at: Option<&DateTime<Utc>>,
    times_on_conflict: &str,
) -> Result<(), StoreError> {
    let row = session_to_stored(session)?;
    tx.execute(
        &format!(
            "INSERT INTO session
               (id, started_at, completed_at, total_duration_secs, completion_status,
                session_notes, entries, updated_at, deleted_at, session_score, capture_version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO UPDATE SET
               started_at = excluded.started_at, completed_at = excluded.completed_at,
               total_duration_secs = excluded.total_duration_secs,
               completion_status = excluded.completion_status,
               session_notes = excluded.session_notes, entries = excluded.entries,
               session_score = excluded.session_score,
               capture_version = excluded.capture_version{times_on_conflict}"
        ),
        params![
            row.id,
            row.started_at,
            row.completed_at,
            row.total_duration_secs,
            row.completion_status,
            row.session_notes,
            row.entries,
            time_text(updated_at),
            deleted_at.map(time_text),
            row.session_score,
            row.capture_version,
        ],
    )?;
    Ok(())
}

pub(crate) fn delete(
    tx: &Transaction,
    id: &str,
    deleted_at: &DateTime<Utc>,
) -> Result<(), StoreError> {
    tx.execute(
        "UPDATE session SET deleted_at = ?1 WHERE id = ?2",
        params![time_text(deleted_at), id],
    )?;
    Ok(())
}

pub(crate) fn restore(
    tx: &Transaction,
    id: &str,
    restored_at: &DateTime<Utc>,
) -> Result<(), StoreError> {
    tx.execute(
        "UPDATE session SET deleted_at = NULL, updated_at = ?1 WHERE id = ?2",
        params![time_text(restored_at), id],
    )?;
    Ok(())
}

/// Tombstones included: plays name deleted variations too (#2246).
pub(crate) fn load_variations(
    tx: &Transaction,
    unreadable: &mut Unreadable,
) -> Result<Vec<Variation>, StoreError> {
    variation_rows(tx, unreadable, &mut Vec::new())
}

pub(crate) fn variation_rows(
    tx: &Transaction,
    unreadable: &mut Unreadable,
    skipped: &mut Vec<String>,
) -> Result<Vec<Variation>, StoreError> {
    let mut stmt = tx.prepare("SELECT * FROM variation ORDER BY rowid")?;
    let mut rows = stmt.query([])?;
    let mut variations = Vec::new();
    while let Some(row) = rows.next()? {
        let id: String = row.get("id")?;
        match variation(row) {
            Some(v) => variations.push(v),
            None => {
                unreadable.push(format!("variation {id} skipped: a column did not read"));
                skipped.push(id);
            }
        }
    }
    Ok(variations)
}

fn variation(row: &Row) -> Option<Variation> {
    let deleted_at: Option<String> = row.get("deleted_at").ok()?;
    Some(Variation {
        id: row.get("id").ok()?,
        label: row.get("label").ok()?,
        updated_at: parse_time(&row.get::<_, String>("updated_at").ok()?)?,
        deleted_at: match deleted_at {
            Some(text) => Some(parse_time(&text)?),
            None => None,
        },
    })
}

/// Keyed by id, tombstones written as sent: the core owns every row.
pub(crate) fn save_variations(
    tx: &Transaction,
    variations: &[Variation],
) -> Result<(), StoreError> {
    for v in variations {
        tx.execute(
            "INSERT INTO variation (id, label, updated_at, deleted_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET
               label = excluded.label, updated_at = excluded.updated_at,
               deleted_at = excluded.deleted_at",
            params![
                v.id,
                v.label,
                time_text(&v.updated_at),
                v.deleted_at.as_ref().map(time_text),
            ],
        )?;
    }
    Ok(())
}
