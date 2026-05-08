use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{delete, get};
use axum::{Json, Router};

use crate::auth::AuthUser;
use crate::db::account::AccountPreferences;
use crate::error::ApiError;
use crate::routes::tokens;
use crate::services;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", delete(delete_account))
        .route("/preferences", get(get_preferences).put(put_preferences))
        .nest("/tokens", tokens::router())
}

async fn get_preferences(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
) -> Result<Json<AccountPreferences>, ApiError> {
    let conn = state.conn();
    let prefs = services::account::get_preferences(&conn, &user_id).await?;
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
