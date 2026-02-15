use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};

use intrada_core::{CreatePiece, Piece, UpdatePiece};

use crate::db;
use crate::error::ApiError;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_pieces).post(create_piece))
        .route(
            "/{id}",
            get(get_piece).put(update_piece).delete(delete_piece),
        )
}

async fn list_pieces(State(state): State<AppState>) -> Result<Json<Vec<Piece>>, ApiError> {
    let pieces = db::pieces::list_pieces(&state.pool).await?;
    Ok(Json(pieces))
}

async fn get_piece(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Piece>, ApiError> {
    let piece = db::pieces::get_piece(&state.pool, &id).await?;
    match piece {
        Some(p) => Ok(Json(p)),
        None => Err(ApiError::NotFound(format!("Piece not found: {id}"))),
    }
}

async fn create_piece(
    State(state): State<AppState>,
    Json(input): Json<CreatePiece>,
) -> Result<(StatusCode, Json<Piece>), ApiError> {
    intrada_core::validation::validate_create_piece(&input)?;
    let piece = db::pieces::insert_piece(&state.pool, &input).await?;
    Ok((StatusCode::CREATED, Json(piece)))
}

async fn update_piece(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdatePiece>,
) -> Result<Json<Piece>, ApiError> {
    intrada_core::validation::validate_update_piece(&input)?;
    let piece = db::pieces::update_piece(&state.pool, &id, &input).await?;
    match piece {
        Some(p) => Ok(Json(p)),
        None => Err(ApiError::NotFound(format!("Piece not found: {id}"))),
    }
}

async fn delete_piece(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let deleted = db::pieces::delete_piece(&state.pool, &id).await?;
    if deleted {
        Ok(Json(json!({ "message": "Piece deleted" })))
    } else {
        Err(ApiError::NotFound(format!("Piece not found: {id}")))
    }
}
