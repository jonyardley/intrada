use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

use intrada_core::domain::exercise::Exercise;
use intrada_core::domain::types::{CreateExercise, UpdateExercise};
use intrada_core::validation;

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
    let conn = state.db.connect()?;
    let exercises = db::exercises::list_exercises(&conn).await?;
    Ok(Json(exercises))
}

async fn get_exercise(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Exercise>, ApiError> {
    let conn = state.db.connect()?;
    let exercise = db::exercises::get_exercise(&conn, &id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Exercise not found: {id}")))?;
    Ok(Json(exercise))
}

async fn create_exercise(
    State(state): State<AppState>,
    Json(input): Json<CreateExercise>,
) -> Result<(StatusCode, Json<Exercise>), ApiError> {
    validation::validate_create_exercise(&input)?;
    let conn = state.db.connect()?;
    let exercise = db::exercises::insert_exercise(&conn, &input).await?;
    Ok((StatusCode::CREATED, Json(exercise)))
}

async fn update_exercise(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdateExercise>,
) -> Result<Json<Exercise>, ApiError> {
    validation::validate_update_exercise(&input)?;
    let conn = state.db.connect()?;
    let exercise = db::exercises::update_exercise(&conn, &id, &input)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Exercise not found: {id}")))?;
    Ok(Json(exercise))
}

async fn delete_exercise(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let conn = state.db.connect()?;
    let deleted = db::exercises::delete_exercise(&conn, &id).await?;
    if deleted {
        Ok(Json(serde_json::json!({ "message": "Exercise deleted" })))
    } else {
        Err(ApiError::NotFound(format!("Exercise not found: {id}")))
    }
}
