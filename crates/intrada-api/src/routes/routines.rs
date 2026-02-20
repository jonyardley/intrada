use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

use intrada_core::domain::routine::Routine;
use intrada_core::validation;

use crate::auth::AuthUser;
use crate::db;
use crate::db::routines::{CreateRoutineEntry, CreateRoutineRequest, UpdateRoutineRequest};
use crate::error::ApiError;
use crate::state::AppState;

const VALID_ITEM_TYPES: &[&str] = &["piece", "exercise"];

fn validate_entries(entries: &[CreateRoutineEntry]) -> Result<(), ApiError> {
    for entry in entries {
        if entry.item_id.trim().is_empty() {
            return Err(ApiError::Validation(
                "Entry item_id must not be empty".to_string(),
            ));
        }
        if entry.item_title.trim().is_empty() {
            return Err(ApiError::Validation(
                "Entry item_title must not be empty".to_string(),
            ));
        }
        if !VALID_ITEM_TYPES.contains(&entry.item_type.as_str()) {
            return Err(ApiError::Validation(format!(
                "Entry item_type must be 'piece' or 'exercise', got '{}'",
                entry.item_type
            )));
        }
    }
    Ok(())
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_routines).post(create_routine))
        .route(
            "/{id}",
            get(get_routine).put(update_routine).delete(delete_routine),
        )
}

async fn list_routines(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<Vec<Routine>>, ApiError> {
    let conn = state.db.connect()?;
    let routines = db::routines::list_routines(&conn, &user_id).await?;
    Ok(Json(routines))
}

async fn get_routine(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Routine>, ApiError> {
    let conn = state.db.connect()?;
    let routine = db::routines::get_routine(&conn, &id, &user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Routine not found: {id}")))?;
    Ok(Json(routine))
}

async fn create_routine(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(input): Json<CreateRoutineRequest>,
) -> Result<(StatusCode, Json<Routine>), ApiError> {
    // Validate routine name
    validation::validate_routine_name(&input.name)?;

    // Validate entries not empty
    if input.entries.is_empty() {
        return Err(ApiError::Validation(
            "Routine must have at least one entry".to_string(),
        ));
    }

    // Validate each entry has required fields and valid item_type
    validate_entries(&input.entries)?;

    let conn = state.db.connect()?;
    let routine = db::routines::insert_routine(&conn, &user_id, &input).await?;
    Ok((StatusCode::CREATED, Json(routine)))
}

async fn update_routine(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
    Json(input): Json<UpdateRoutineRequest>,
) -> Result<Json<Routine>, ApiError> {
    // Validate routine name
    validation::validate_routine_name(&input.name)?;

    // Validate entries not empty
    if input.entries.is_empty() {
        return Err(ApiError::Validation(
            "Routine must have at least one entry".to_string(),
        ));
    }

    // Validate each entry has required fields and valid item_type
    validate_entries(&input.entries)?;

    let conn = state.db.connect()?;
    let routine = db::routines::update_routine(&conn, &id, &user_id, &input)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Routine not found: {id}")))?;
    Ok(Json(routine))
}

async fn delete_routine(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let conn = state.db.connect()?;
    let deleted = db::routines::delete_routine(&conn, &id, &user_id).await?;
    if deleted {
        Ok(Json(serde_json::json!({ "message": "Routine deleted" })))
    } else {
        Err(ApiError::NotFound(format!("Routine not found: {id}")))
    }
}
