use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

use crate::auth::AuthUser;
use crate::db::tokens::{CreateTokenRequest, CreatedTokenResponse, TokenListItem};
use crate::error::ApiError;
use crate::services;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tokens).post(create_token))
        .route("/{id}", axum::routing::delete(revoke_token))
}

async fn list_tokens(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
) -> Result<Json<Vec<TokenListItem>>, ApiError> {
    let tokens = state
        .with_transient_retry(|conn| {
            let user_id = user_id.clone();
            async move { services::tokens::list_tokens(&conn, &user_id).await }
        })
        .await?;
    Ok(Json(tokens))
}

async fn create_token(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Json(input): Json<CreateTokenRequest>,
) -> Result<(StatusCode, Json<CreatedTokenResponse>), ApiError> {
    let conn = state.conn();
    let response = services::tokens::create_token(&conn, &user_id, &input.name).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

async fn revoke_token(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let conn = state.conn();
    services::tokens::revoke_token(&conn, &user_id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}
