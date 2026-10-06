// ── Sessions and variations ──────────────────────────────────────────
// The iPhone's session and variation operations and SessionCodec, ported
// (#2421). The core reads and writes a session's columns (#2234).

use intrada_core::domain::Variation;
use intrada_core::stored_session::{session_from_stored, session_to_stored, StoredSession};
use intrada_core::PracticeSession;
use rusqlite::{params, Row, Transaction};

use crate::codec::{parse_time, time_text, Unreadable};
use crate::StoreError;

pub(crate) fn load(
    tx: &Transaction,
    unreadable: &mut Unreadable,
) -> Result<Vec<PracticeSession>, StoreError> {
    let mut stmt =
        tx.prepare("SELECT * FROM session WHERE deleted_at IS NULL ORDER BY completed_at DESC")?;
    let mut rows = stmt.query([])?;
    let mut sessions = Vec::new();
    while let Some(row) = rows.next()? {
        let id: String = row.get("id")?;
        // The intention and the three reflection columns stay in the table
        // unread: nothing can set them and the core no longer carries them
        // (#1766, #1374).
        let read = stored(row)
            .map_err(|e| e.to_string())
            .and_then(|s| session_from_stored(&s).map_err(|e| e.to_string()));
        match read {
            Ok(read) => {
                unreadable.extend(read.unreadable);
                sessions.push(read.session);
            }
            Err(reason) => unreadable.push(format!("session {id} skipped: {reason}")),
        }
    }
    Ok(sessions)
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

/// Insert or update by id. A session is immutable once completed, so
/// `updated_at` tracks `completed_at`; the column exists for sync.
pub(crate) fn save(tx: &Transaction, session: &PracticeSession) -> Result<(), StoreError> {
    let row = session_to_stored(session)?;
    tx.execute(
        "INSERT INTO session
           (id, started_at, completed_at, total_duration_secs, completion_status,
            session_notes, entries, updated_at, deleted_at, session_score, capture_version)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, NULL, ?9, ?10)
         ON CONFLICT(id) DO UPDATE SET
           started_at = excluded.started_at, completed_at = excluded.completed_at,
           total_duration_secs = excluded.total_duration_secs,
           completion_status = excluded.completion_status,
           session_notes = excluded.session_notes,
           entries = excluded.entries, updated_at = excluded.updated_at, deleted_at = NULL,
           session_score = excluded.session_score, capture_version = excluded.capture_version",
        params![
            row.id,
            row.started_at,
            row.completed_at,
            row.total_duration_secs,
            row.completion_status,
            row.session_notes,
            row.entries,
            row.completed_at,
            row.session_score,
            row.capture_version,
        ],
    )?;
    Ok(())
}

/// Tombstones included: plays name deleted variations too (#2246).
pub(crate) fn load_variations(
    tx: &Transaction,
    unreadable: &mut Unreadable,
) -> Result<Vec<Variation>, StoreError> {
    let mut stmt = tx.prepare("SELECT * FROM variation ORDER BY rowid")?;
    let mut rows = stmt.query([])?;
    let mut variations = Vec::new();
    while let Some(row) = rows.next()? {
        let id: String = row.get("id")?;
        match variation(row) {
            Some(v) => variations.push(v),
            None => unreadable.push(format!("variation {id} skipped: a column did not read")),
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
