use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

use intrada_core::domain::item::Item;
use intrada_core::domain::types::{CreateItem, UpdateItem};

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::services;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_items).post(create_item))
        .route("/{id}", get(get_item).put(update_item).delete(delete_item))
}

async fn list_items(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
) -> Result<Json<Vec<Item>>, ApiError> {
    let conn = state.conn();
    let items = services::items::list_items(&conn, &user_id).await?;
    Ok(Json(items))
}

async fn get_item(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Item>, ApiError> {
    let conn = state.conn();
    let item = services::items::get_item(&conn, &id, &user_id).await?;
    Ok(Json(item))
}

async fn create_item(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Json(input): Json<CreateItem>,
) -> Result<(StatusCode, Json<Item>), ApiError> {
    let conn = state.conn();
    let item = services::items::create_item(&conn, &user_id, &input).await?;
    Ok((StatusCode::CREATED, Json(item)))
}

async fn update_item(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Path(id): Path<String>,
    Json(input): Json<UpdateItem>,
) -> Result<Json<Item>, ApiError> {
    let conn = state.conn();
    let item = services::items::update_item(&conn, &id, &user_id, &input).await?;
    Ok(Json(item))
}

async fn delete_item(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let conn = state.conn();
    services::items::delete_item(&conn, &id, &user_id).await?;
    Ok(Json(serde_json::json!({ "message": "Item deleted" })))
}
