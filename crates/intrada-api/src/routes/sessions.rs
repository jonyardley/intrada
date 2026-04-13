use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

use intrada_core::domain::session::PracticeSession;
use intrada_core::validation;

use crate::auth::AuthUser;
use crate::db;
use crate::db::sessions::SaveSessionRequest;
use crate::error::ApiError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_sessions).post(save_session))
        .route("/{id}", get(get_session).delete(delete_session))
}

async fn list_sessions(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<Vec<PracticeSession>>, ApiError> {
    let conn = state.conn();
    let sessions = db::sessions::list_sessions(&conn, &user_id).await?;
    Ok(Json(sessions))
}

async fn get_session(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
) -> Result<Json<PracticeSession>, ApiError> {
    let conn = state.conn();
    let session = db::sessions::get_session(&conn, &id, &user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Session not found: {id}")))?;
    Ok(Json(session))
}

async fn save_session(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(input): Json<SaveSessionRequest>,
) -> Result<(StatusCode, Json<PracticeSession>), ApiError> {
    // Validate session-level fields
    validation::validate_session_notes(&input.session_notes)?;
    validation::validate_intention(&input.session_intention)?;

    // Validate entries not empty
    validation::validate_entries_not_empty(&input.entries, "Practice")?;

    // Validate each entry's fields
    for entry in &input.entries {
        validation::validate_routine_entry_fields(&entry.item_id, &entry.item_title)?;
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

    let conn = state.conn();
    let session = db::sessions::insert_session(&conn, &user_id, &input).await?;
    Ok((StatusCode::CREATED, Json(session)))
}

async fn delete_session(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let conn = state.conn();
    let deleted = db::sessions::delete_session(&conn, &id, &user_id).await?;
    if deleted {
        Ok(Json(serde_json::json!({ "message": "Session deleted" })))
    } else {
        Err(ApiError::NotFound(format!("Session not found: {id}")))
    }
}
