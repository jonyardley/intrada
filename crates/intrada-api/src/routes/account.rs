use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{delete, get};
use axum::{Json, Router};

use crate::auth::AuthUser;
use crate::db;
use crate::db::account::AccountPreferences;
use crate::error::ApiError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", delete(delete_account))
        .route("/preferences", get(get_preferences).put(put_preferences))
}

async fn get_preferences(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<AccountPreferences>, ApiError> {
    let conn = state.conn();
    let prefs = db::account::get_preferences(&conn, &user_id).await?;
    Ok(Json(prefs))
}

async fn put_preferences(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(input): Json<AccountPreferences>,
) -> Result<Json<AccountPreferences>, ApiError> {
    if input.default_focus_minutes == 0 || input.default_focus_minutes > 600 {
        return Err(ApiError::Validation(
            "default_focus_minutes must be 1..=600".to_string(),
        ));
    }
    if input.default_rep_count == 0 || input.default_rep_count > 999 {
        return Err(ApiError::Validation(
            "default_rep_count must be 1..=999".to_string(),
        ));
    }
    let conn = state.conn();
    let prefs = db::account::upsert_preferences(&conn, &user_id, &input).await?;
    Ok(Json(prefs))
}

async fn delete_account(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<StatusCode, ApiError> {
    // Refuse empty user_id outright — auth-disabled mode (no
    // CLERK_ISSUER_URL) yields `AuthUser("")`, which would otherwise
    // turn into an R2 prefix `/` and a Clerk DELETE /v1/users/ —
    // both blast-radius hazards.
    if user_id.is_empty() {
        return Err(ApiError::Unauthorized("Unauthorized".to_string()));
    }

    // 1. DB rows. Hard fail — if data delete doesn't succeed, the user
    //    can re-run.
    let conn = state.conn();
    db::account::delete_all_user_data(&conn, &user_id).await?;

    // 2. R2 photo blobs. Best-effort: log but don't fail. The DB
    //    `lesson_photos` rows are already gone, so the blobs are
    //    orphaned-but-private (keys include user_id; bucket has no public
    //    listing).
    if let Some(r2) = &state.r2 {
        if let Err(err) = r2.delete_user_photos(&user_id).await {
            tracing::warn!(?err, %user_id, "R2 photo cleanup failed during account delete");
        }
    }

    // 3. Clerk user record. Best-effort: log but don't fail. 404 from
    //    Clerk is treated as success (idempotent retry).
    if let Some(clerk) = &state.clerk {
        if let Err(err) = clerk.delete_user(&user_id).await {
            tracing::warn!(?err, %user_id, "Clerk user delete failed during account delete");
        }
    }

    Ok(StatusCode::NO_CONTENT)
}
