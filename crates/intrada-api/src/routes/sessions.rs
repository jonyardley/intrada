use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

use intrada_core::domain::session::PracticeSession;
use intrada_core::validation;

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
) -> Result<Json<Vec<PracticeSession>>, ApiError> {
    let conn = state.db.connect()?;
    let sessions = db::sessions::list_sessions(&conn).await?;
    Ok(Json(sessions))
}

async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PracticeSession>, ApiError> {
    let conn = state.db.connect()?;
    let session = db::sessions::get_session(&conn, &id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Session not found: {id}")))?;
    Ok(Json(session))
}

async fn save_session(
    State(state): State<AppState>,
    Json(input): Json<SaveSessionRequest>,
) -> Result<(StatusCode, Json<PracticeSession>), ApiError> {
    // Validate session notes
    validation::validate_session_notes(&input.session_notes)?;

    // Validate each entry's notes
    for entry in &input.entries {
        validation::validate_entry_notes(&entry.notes)?;
    }

    // Validate setlist is not empty — need to convert to SetlistEntry slice
    // We validate the entries vec is not empty directly
    if input.entries.is_empty() {
        return Err(ApiError::Validation(
            "Setlist must have at least one entry".to_string(),
        ));
    }

    let conn = state.db.connect()?;
    let session = db::sessions::insert_session(&conn, &input).await?;
    Ok((StatusCode::CREATED, Json(session)))
}

async fn delete_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let conn = state.db.connect()?;
    let deleted = db::sessions::delete_session(&conn, &id).await?;
    if deleted {
        Ok(Json(serde_json::json!({ "message": "Session deleted" })))
    } else {
        Err(ApiError::NotFound(format!("Session not found: {id}")))
    }
}
