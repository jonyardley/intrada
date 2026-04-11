use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};

use intrada_core::domain::lesson::Lesson;
use intrada_core::domain::types::{CreateLesson, UpdateLesson};
use intrada_core::validation;

use crate::auth::AuthUser;
use crate::db;
use crate::error::ApiError;
use crate::state::AppState;
use crate::storage::R2Client;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_lessons).post(create_lesson))
        .route(
            "/{id}",
            get(get_lesson).put(update_lesson).delete(delete_lesson),
        )
        .route("/{id}/photos", post(upload_photo))
        .route("/{id}/photos/{photo_id}", delete(delete_photo))
}

async fn list_lessons(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<Vec<Lesson>>, ApiError> {
    let conn = state.connect().await?;
    let r2 = state.r2()?;
    let lessons = db::lessons::list_lessons(&conn, &user_id, r2).await?;
    Ok(Json(lessons))
}

async fn get_lesson(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Lesson>, ApiError> {
    let conn = state.connect().await?;
    let r2 = state.r2()?;
    let lesson = db::lessons::get_lesson(&conn, &id, &user_id, r2)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Lesson not found: {id}")))?;
    Ok(Json(lesson))
}

async fn create_lesson(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(input): Json<CreateLesson>,
) -> Result<(StatusCode, Json<Lesson>), ApiError> {
    validation::validate_create_lesson(&input)?;
    let conn = state.connect().await?;
    let r2 = state.r2()?;
    let lesson = db::lessons::insert_lesson(&conn, &user_id, &input, r2).await?;
    Ok((StatusCode::CREATED, Json(lesson)))
}

async fn update_lesson(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
    Json(input): Json<UpdateLesson>,
) -> Result<Json<Lesson>, ApiError> {
    validation::validate_update_lesson(&input)?;
    let conn = state.connect().await?;
    let r2 = state.r2()?;
    let lesson = db::lessons::update_lesson(&conn, &id, &user_id, &input, r2)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Lesson not found: {id}")))?;
    Ok(Json(lesson))
}

async fn delete_lesson(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let conn = state.connect().await?;

    // Delete photos from R2 if storage is configured
    if let Ok(r2) = state.r2() {
        let keys = db::lessons::list_photo_storage_keys(&conn, &id, &user_id).await?;
        for key in keys {
            let _ = r2.delete(&key).await;
        }
    }

    let deleted = db::lessons::delete_lesson(&conn, &id, &user_id).await?;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound(format!("Lesson not found: {id}")))
    }
}

// ── Photo endpoints ────────────────────────────────────────────────────

/// Max photo upload size: 5 MB
const MAX_PHOTO_SIZE: usize = 5 * 1024 * 1024;

async fn upload_photo(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let r2 = state.r2()?;
    let conn = state.connect().await?;

    // Verify lesson exists and belongs to user
    db::lessons::get_lesson(&conn, &id, &user_id, r2)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Lesson not found: {id}")))?;

    // Extract the photo field from multipart
    let field = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::Validation(format!("Invalid multipart data: {e}")))?
        .ok_or_else(|| ApiError::Validation("No photo field in request".into()))?;

    let content_type = field
        .content_type()
        .unwrap_or("application/octet-stream")
        .to_string();

    // Validate content type
    if !matches!(content_type.as_str(), "image/jpeg" | "image/png") {
        return Err(ApiError::Validation("Photo must be JPEG or PNG".into()));
    }

    let data = field
        .bytes()
        .await
        .map_err(|e| ApiError::Validation(format!("Failed to read photo data: {e}")))?;

    if data.len() > MAX_PHOTO_SIZE {
        return Err(ApiError::Validation(format!(
            "Photo exceeds maximum size of {} MB",
            MAX_PHOTO_SIZE / (1024 * 1024)
        )));
    }

    // Upload to R2
    let photo_id = ulid::Ulid::new().to_string();
    let storage_key = R2Client::photo_key(&user_id, &id, &photo_id);
    r2.upload(&storage_key, &data, &content_type).await?;

    // Record in DB
    let photo = db::lessons::insert_lesson_photo(&conn, &id, &user_id, &storage_key, r2).await?;

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
    let conn = state.connect().await?;

    // Get storage key before deleting from DB
    let storage_key = db::lessons::get_lesson_photo_storage_key(&conn, &photo_id, &id, &user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Photo not found: {photo_id}")))?;

    // Delete from R2
    if let Ok(r2) = state.r2() {
        let _ = r2.delete(&storage_key).await;
    }

    // Delete from DB
    db::lessons::delete_lesson_photo(&conn, &photo_id, &id, &user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
