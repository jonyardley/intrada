use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

use intrada_core::domain::set::Set;

use crate::auth::AuthUser;
use crate::db::sets::{CreateSetRequest, UpdateSetRequest};
use crate::error::ApiError;
use crate::services;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_sets).post(create_set))
        .route("/{id}", get(get_set).put(update_set).delete(delete_set))
}

async fn list_sets(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<Vec<Set>>, ApiError> {
    let conn = state.conn();
    let sets = services::sets::list_sets(&conn, &user_id).await?;
    Ok(Json(sets))
}

async fn get_set(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Set>, ApiError> {
    let conn = state.conn();
    let set = services::sets::get_set(&conn, &id, &user_id).await?;
    Ok(Json(set))
}

async fn create_set(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(input): Json<CreateSetRequest>,
) -> Result<(StatusCode, Json<Set>), ApiError> {
    let conn = state.conn();
    let set = services::sets::create_set(&conn, &user_id, &input).await?;
    Ok((StatusCode::CREATED, Json(set)))
}

async fn update_set(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
    Json(input): Json<UpdateSetRequest>,
) -> Result<Json<Set>, ApiError> {
    let conn = state.conn();
    let set = services::sets::update_set(&conn, &id, &user_id, &input).await?;
    Ok(Json(set))
}

async fn delete_set(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let conn = state.conn();
    services::sets::delete_set(&conn, &id, &user_id).await?;
    Ok(Json(serde_json::json!({ "message": "Set deleted" })))
}
