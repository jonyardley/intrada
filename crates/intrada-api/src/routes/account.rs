use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get};
use axum::{Json, Router};
use serde::Deserialize;

use crate::auth::AuthUser;
use crate::db::account::AccountPreferences;
use crate::db::audit::AuditLogEntry;
use crate::error::ApiError;
use crate::routes::tokens;
use crate::services;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", delete(delete_account))
        .route("/preferences", get(get_preferences).put(put_preferences))
        .route("/audit", get(get_audit_log))
        .nest("/tokens", tokens::router())
}

const DEFAULT_AUDIT_LIMIT: u32 = 50;

#[derive(Debug, Deserialize)]
struct AuditQuery {
    /// Page size. Defaults to 50; service hard-caps at 500.
    limit: Option<u32>,
}

async fn get_audit_log(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Query(q): Query<AuditQuery>,
) -> Result<Json<Vec<AuditLogEntry>>, ApiError> {
    let limit = q.limit.unwrap_or(DEFAULT_AUDIT_LIMIT);
    let entries = state
        .with_transient_retry(|conn| {
            let user_id = user_id.clone();
            async move { services::audit::list_audit(&conn, &user_id, limit).await }
        })
        .await?;
    Ok(Json(entries))
}

async fn get_preferences(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
) -> Result<Json<AccountPreferences>, ApiError> {
    let prefs = state
        .with_transient_retry(|conn| {
            let user_id = user_id.clone();
            async move { services::account::get_preferences(&conn, &user_id).await }
        })
        .await?;
    Ok(Json(prefs))
}

async fn put_preferences(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Json(input): Json<AccountPreferences>,
) -> Result<Json<AccountPreferences>, ApiError> {
    let conn = state.conn();
    let prefs = services::account::put_preferences(&conn, &user_id, &input).await?;
    Ok(Json(prefs))
}

async fn delete_account(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
) -> Result<StatusCode, ApiError> {
    let conn = state.conn();
    services::account::delete_account(&conn, state.r2.as_ref(), state.clerk.as_ref(), &user_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
