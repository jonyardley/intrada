use libsql::Connection;

use intrada_core::domain::session::PracticeSession;
use intrada_core::validation;

use crate::db;
use crate::db::sessions::SaveSessionRequest;
use crate::error::ApiError;

pub async fn list_sessions(
    conn: &Connection,
    user_id: &str,
) -> Result<Vec<PracticeSession>, ApiError> {
    db::sessions::list_sessions(conn, user_id).await
}

pub async fn get_session(
    conn: &Connection,
    id: &str,
    user_id: &str,
) -> Result<PracticeSession, ApiError> {
    db::sessions::get_session(conn, id, user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Session not found: {id}")))
}

pub async fn save_session(
    conn: &Connection,
    user_id: &str,
    input: &SaveSessionRequest,
) -> Result<PracticeSession, ApiError> {
    validation::validate_session_notes(&input.session_notes)?;
    validation::validate_intention(&input.session_intention)?;
    validation::validate_entries_not_empty(&input.entries, "Practice")?;

    for entry in &input.entries {
        validation::validate_set_entry_fields(&entry.item_id, &entry.item_title)?;
        validation::validate_entry_notes(&entry.notes)?;
        validation::validate_score(&entry.score)?;
        validation::validate_intention(&entry.intention)?;
        validation::validate_rep_target(&entry.rep_target)?;
        validation::validate_planned_duration(&entry.planned_duration_secs)?;
        validation::validate_achieved_tempo(&entry.achieved_tempo)?;
        validation::validate_rep_consistency(
            entry.rep_target,
            entry.rep_count,
            entry.rep_target_reached,
            entry.rep_history.is_some(),
        )?;
    }

    db::sessions::insert_session(conn, user_id, input).await
}

pub async fn delete_session(conn: &Connection, id: &str, user_id: &str) -> Result<(), ApiError> {
    let deleted = db::sessions::delete_session(conn, id, user_id).await?;
    if !deleted {
        return Err(ApiError::NotFound(format!("Session not found: {id}")));
    }
    Ok(())
}
