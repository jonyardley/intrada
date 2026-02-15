use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

use intrada_core::domain::piece::Piece;
use intrada_core::domain::types::{CreatePiece, UpdatePiece};
use intrada_core::validation;

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
    let conn = state.db.connect()?;
    let pieces = db::pieces::list_pieces(&conn).await?;
    Ok(Json(pieces))
}

async fn get_piece(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Piece>, ApiError> {
    let conn = state.db.connect()?;
    let piece = db::pieces::get_piece(&conn, &id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Piece not found: {id}")))?;
    Ok(Json(piece))
}

async fn create_piece(
    State(state): State<AppState>,
    Json(input): Json<CreatePiece>,
) -> Result<(StatusCode, Json<Piece>), ApiError> {
    validation::validate_create_piece(&input)?;
    let conn = state.db.connect()?;
    let piece = db::pieces::insert_piece(&conn, &input).await?;
    Ok((StatusCode::CREATED, Json(piece)))
}

async fn update_piece(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<UpdatePiece>,
) -> Result<Json<Piece>, ApiError> {
    validation::validate_update_piece(&input)?;
    let conn = state.db.connect()?;
    let piece = db::pieces::update_piece(&conn, &id, &input)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Piece not found: {id}")))?;
    Ok(Json(piece))
}

async fn delete_piece(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let conn = state.db.connect()?;
    let deleted = db::pieces::delete_piece(&conn, &id).await?;
    if deleted {
        Ok(Json(serde_json::json!({ "message": "Piece deleted" })))
    } else {
        Err(ApiError::NotFound(format!("Piece not found: {id}")))
    }
}
