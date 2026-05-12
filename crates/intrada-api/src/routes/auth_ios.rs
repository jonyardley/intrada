use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde::Serialize;

use crate::auth::{AuthSource, AuthUser};
use crate::error::ApiError;
use crate::services;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/exchange", post(exchange))
}

#[derive(Serialize)]
struct ExchangeResponse {
    token: String,
    user_id: String,
    email: Option<String>,
}

async fn exchange(
    State(state): State<AppState>,
    AuthUser { user_id, source }: AuthUser,
) -> Result<(StatusCode, Json<ExchangeResponse>), ApiError> {
    if !matches!(source, AuthSource::Jwt) {
        return Err(ApiError::Unauthorized(
            "iOS exchange requires a Clerk JWT, not a PAT".into(),
        ));
    }

    let email = match &state.clerk {
        Some(clerk) => clerk
            .get_user(&user_id)
            .await?
            .primary_email()
            .map(String::from),
        None => None,
    };

    let conn = state.conn();
    services::tokens::revoke_tokens_by_name(&conn, &user_id, "iOS App").await?;
    let created = services::tokens::create_token(&conn, &user_id, "iOS App").await?;

    Ok((
        StatusCode::CREATED,
        Json(ExchangeResponse {
            token: created.token,
            user_id,
            email,
        }),
    ))
}
