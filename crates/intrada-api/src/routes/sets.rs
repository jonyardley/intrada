use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

use intrada_core::domain::set::Set;
use intrada_core::validation;

use crate::auth::AuthUser;
use crate::db;
use crate::db::sets::{CreateSetEntry, CreateSetRequest, UpdateSetRequest};
use crate::error::ApiError;
use crate::state::AppState;

fn validate_entries(entries: &[CreateSetEntry]) -> Result<(), ApiError> {
    for entry in entries {
        validation::validate_set_entry_fields(&entry.item_id, &entry.item_title)?;
    }
    Ok(())
}

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
    let sets = db::sets::list_sets(&conn, &user_id).await?;
    Ok(Json(sets))
}

async fn get_set(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Set>, ApiError> {
    let conn = state.conn();
    let set = db::sets::get_set(&conn, &id, &user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Set not found: {id}")))?;
    Ok(Json(set))
}

async fn create_set(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(input): Json<CreateSetRequest>,
) -> Result<(StatusCode, Json<Set>), ApiError> {
    // Validate set name
    validation::validate_set_name(&input.name)?;

    // Validate entries not empty
    validation::validate_entries_not_empty(&input.entries, "Set")?;

    // Validate each entry has required fields and valid item_type
    validate_entries(&input.entries)?;

    let conn = state.conn();
    let set = db::sets::insert_set(&conn, &user_id, &input).await?;
    Ok((StatusCode::CREATED, Json(set)))
}

async fn update_set(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
    Json(input): Json<UpdateSetRequest>,
) -> Result<Json<Set>, ApiError> {
    // Validate set name
    validation::validate_set_name(&input.name)?;

    // Validate entries not empty
    validation::validate_entries_not_empty(&input.entries, "Set")?;

    // Validate each entry has required fields and valid item_type
    validate_entries(&input.entries)?;

    let conn = state.conn();
    let set = db::sets::update_set(&conn, &id, &user_id, &input)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Set not found: {id}")))?;
    Ok(Json(set))
}

async fn delete_set(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let conn = state.conn();
    let deleted = db::sets::delete_set(&conn, &id, &user_id).await?;
    if deleted {
        Ok(Json(serde_json::json!({ "message": "Set deleted" })))
    } else {
        Err(ApiError::NotFound(format!("Set not found: {id}")))
    }
}
