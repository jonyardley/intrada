use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

use intrada_core::domain::session::PracticeSession;

use crate::auth::AuthUser;
use crate::db::sessions::SaveSessionRequest;
use crate::error::ApiError;
use crate::services;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_sessions).post(save_session))
        .route("/{id}", get(get_session).delete(delete_session))
}

async fn list_sessions(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
) -> Result<Json<Vec<PracticeSession>>, ApiError> {
    let sessions = state
        .with_transient_retry(|conn| {
            let user_id = user_id.clone();
            async move { services::sessions::list_sessions(&conn, &user_id).await }
        })
        .await?;
    Ok(Json(sessions))
}

async fn get_session(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<PracticeSession>, ApiError> {
    let session = state
        .with_transient_retry(|conn| {
            let id = id.clone();
            let user_id = user_id.clone();
            async move { services::sessions::get_session(&conn, &id, &user_id).await }
        })
        .await?;
    Ok(Json(session))
}

async fn save_session(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Json(input): Json<SaveSessionRequest>,
) -> Result<(StatusCode, Json<PracticeSession>), ApiError> {
    let conn = state.conn();
    let session = services::sessions::save_session(&conn, &user_id, &input).await?;
    Ok((StatusCode::CREATED, Json(session)))
}

async fn delete_session(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let conn = state.conn();
    services::sessions::delete_session(&conn, &id, &user_id).await?;
    Ok(Json(serde_json::json!({ "message": "Session deleted" })))
}
