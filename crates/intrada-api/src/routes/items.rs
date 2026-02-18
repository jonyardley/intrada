use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

use intrada_core::domain::item::Item;
use intrada_core::domain::types::{CreateItem, UpdateItem};
use intrada_core::validation;

use crate::db;
use crate::error::ApiError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_items).post(create_item))
        .route("/{id}", get(get_item).put(update_item).delete(delete_item))
}

async fn list_items(State(state): State<AppState>) -> Result<Json<Vec<Item>>, ApiError> {
    let conn = state.db.connect()?;
    let items = db::items::list_items(&conn).await?;
    Ok(Json(items))
}

async fn get_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Item>, ApiError> {
    let conn = state.db.connect()?;
    let item = db::items::get_item(&conn, &id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Item not found: {id}")))?;
    Ok(Json(item))
}

async fn create_item(
    State(state): State<AppState>,
    Json(input): Json<CreateItem>,
) -> Result<(StatusCode, Json<Item>), ApiError> {
    validation::validate_create_item(&input)?;
    let conn = state.db.connect()?;
    let item = db::items::insert_item(&conn, &input).await?;
    Ok((StatusCode::CREATED, Json(item)))
}

async fn update_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateItem>,
) -> Result<Json<Item>, ApiError> {
    validation::validate_update_item(&input)?;
    let conn = state.db.connect()?;
    let item = db::items::update_item(&conn, &id, &input)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Item not found: {id}")))?;
    Ok(Json(item))
}

async fn delete_item(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let conn = state.db.connect()?;
    let deleted = db::items::delete_item(&conn, &id).await?;
    if deleted {
        Ok(Json(serde_json::json!({ "message": "Item deleted" })))
    } else {
        Err(ApiError::NotFound(format!("Item not found: {id}")))
    }
}
