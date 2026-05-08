use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use tower_http::limit::RequestBodyLimitLayer;

use intrada_core::domain::lesson::Lesson;
use intrada_core::domain::types::{CreateLesson, UpdateLesson};

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::services;
use crate::services::lessons::MAX_PHOTO_SIZE;
use crate::state::AppState;

/// HTTP body limit for photo uploads: 5 MB photo + multipart overhead.
const PHOTO_BODY_LIMIT: usize = 6 * 1024 * 1024;

pub fn router() -> Router<AppState> {
    // Photo upload gets its own sub-router so the 6 MB body limit is
    // scoped to just that route — JSON endpoints keep axum's default.
    let photo_upload = Router::new()
        .route("/{id}/photos", post(upload_photo))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(PHOTO_BODY_LIMIT));

    Router::new()
        .route("/", get(list_lessons).post(create_lesson))
        .route(
            "/{id}",
            get(get_lesson).put(update_lesson).delete(delete_lesson),
        )
        .route("/{id}/photos/{photo_id}", delete(delete_photo))
        .merge(photo_upload)
}

async fn list_lessons(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<Vec<Lesson>>, ApiError> {
    let conn = state.conn();
    let r2 = state.r2()?;
    let lessons = services::lessons::list_lessons(&conn, r2, &user_id).await?;
    Ok(Json(lessons))
}

async fn get_lesson(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Lesson>, ApiError> {
    let conn = state.conn();
    let r2 = state.r2()?;
    let lesson = services::lessons::get_lesson(&conn, r2, &id, &user_id).await?;
    Ok(Json(lesson))
}

async fn create_lesson(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(input): Json<CreateLesson>,
) -> Result<(StatusCode, Json<Lesson>), ApiError> {
    let conn = state.conn();
    let r2 = state.r2()?;
    let lesson = services::lessons::create_lesson(&conn, r2, &user_id, &input).await?;
    Ok((StatusCode::CREATED, Json(lesson)))
}

async fn update_lesson(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
    Json(input): Json<UpdateLesson>,
) -> Result<Json<Lesson>, ApiError> {
    let conn = state.conn();
    let r2 = state.r2()?;
    let lesson = services::lessons::update_lesson(&conn, r2, &id, &user_id, &input).await?;
    Ok(Json(lesson))
}

async fn delete_lesson(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let conn = state.conn();
    services::lessons::delete_lesson(&conn, state.r2.as_ref(), &id, &user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Photo endpoints ────────────────────────────────────────────────────

async fn upload_photo(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let r2 = state.r2()?;
    let conn = state.conn();

    // Extract the photo field from multipart
    let field = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::Validation(format!("Invalid multipart data: {e}")))?
        .ok_or_else(|| ApiError::Validation("No photo field in request".into()))?;

    let data = field
        .bytes()
        .await
        .map_err(|e| ApiError::Validation(format!("Failed to read photo data: {e}")))?;

    // Early body-size guard before handing off to the service. The service
    // checks again (so non-HTTP callers like MCP get the same behaviour),
    // but rejecting here avoids a wasted clone of the bytes into the service.
    if data.len() > MAX_PHOTO_SIZE {
        return Err(ApiError::Validation(format!(
            "Photo exceeds maximum size of {} MB",
            MAX_PHOTO_SIZE / (1024 * 1024)
        )));
    }

    let photo = services::lessons::upload_lesson_photo(&conn, r2, &user_id, &id, &data).await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": photo.id,
            "url": photo.url,
            "created_at": photo.created_at.to_rfc3339(),
        })),
    ))
}

async fn delete_photo(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path((id, photo_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    let conn = state.conn();
    services::lessons::delete_lesson_photo(&conn, state.r2.as_ref(), &user_id, &id, &photo_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
