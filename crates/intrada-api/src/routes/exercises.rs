use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};

use intrada_core::{CreateExercise, Exercise, UpdateExercise};

use crate::db;
use crate::error::ApiError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_exercises).post(create_exercise))
        .route(
            "/{id}",
            get(get_exercise)
                .put(update_exercise)
                .delete(delete_exercise),
        )
}

async fn list_exercises(State(state): State<AppState>) -> Result<Json<Vec<Exercise>>, ApiError> {
    let exercises = db::exercises::list_exercises(&state.pool).await?;
    Ok(Json(exercises))
}

async fn get_exercise(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Exercise>, ApiError> {
    let exercise = db::exercises::get_exercise(&state.pool, &id).await?;
    match exercise {
        Some(e) => Ok(Json(e)),
        None => Err(ApiError::NotFound(format!("Exercise not found: {id}"))),
    }
}

async fn create_exercise(
    State(state): State<AppState>,
    Json(input): Json<CreateExercise>,
) -> Result<(StatusCode, Json<Exercise>), ApiError> {
    intrada_core::validation::validate_create_exercise(&input)?;
    let exercise = db::exercises::insert_exercise(&state.pool, &input).await?;
    Ok((StatusCode::CREATED, Json(exercise)))
}

async fn update_exercise(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateExercise>,
) -> Result<Json<Exercise>, ApiError> {
    intrada_core::validation::validate_update_exercise(&input)?;
    let exercise = db::exercises::update_exercise(&state.pool, &id, &input).await?;
    match exercise {
        Some(e) => Ok(Json(e)),
        None => Err(ApiError::NotFound(format!("Exercise not found: {id}"))),
    }
}

async fn delete_exercise(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let deleted = db::exercises::delete_exercise(&state.pool, &id).await?;
    if deleted {
        Ok(Json(json!({ "message": "Exercise deleted" })))
    } else {
        Err(ApiError::NotFound(format!("Exercise not found: {id}")))
    }
}
