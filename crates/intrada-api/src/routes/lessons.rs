use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use tower_http::limit::RequestBodyLimitLayer;

use intrada_core::domain::lesson::Lesson;
use intrada_core::domain::types::{CreateLesson, UpdateLesson};
use intrada_core::validation;

use crate::auth::AuthUser;
use crate::db;
use crate::error::ApiError;
use crate::state::AppState;
use crate::storage::R2Client;

/// Max photo upload size: 5 MB
const MAX_PHOTO_SIZE: usize = 5 * 1024 * 1024;

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

    // Delete photos from R2 if storage is configured. Log but don't fail
    // the request on R2 errors — DB is the source of truth and the lesson
    // delete below cascades the photo rows.
    if let Ok(r2) = state.r2() {
        let keys = db::lessons::list_photo_storage_keys(&conn, &id, &user_id).await?;
        for key in keys {
            if let Err(e) = r2.delete(&key).await {
                tracing::warn!(lesson_id = %id, storage_key = %key, error = ?e, "R2 photo delete failed; orphaning object");
            }
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

/// Inspect the leading bytes and return the canonical image content-type,
/// or `None` if the bytes don't match a supported format. This is the only
/// source of truth for a photo's content-type — the client-supplied
/// multipart header is never trusted.
fn sniff_image_content_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        Some("image/png")
    } else {
        None
    }
}

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

    // Determine content-type from the bytes themselves — never trust the
    // client's multipart header. Prevents XSS via spoofed Content-Type on
    // the public R2 URL.
    let content_type = sniff_image_content_type(&data)
        .ok_or_else(|| ApiError::Validation("Photo must be JPEG or PNG".into()))?;

    // Upload to R2
    let photo_id = ulid::Ulid::new().to_string();
    let storage_key = R2Client::photo_key(&user_id, &id, &photo_id);
    r2.upload(&storage_key, &data, content_type).await?;

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

    // Delete from R2. Log but don't fail the request on R2 errors — the
    // DB row below is the authoritative pointer; orphaned R2 objects are
    // recoverable via prefix sweep but a stuck DB row is not.
    if let Ok(r2) = state.r2() {
        if let Err(e) = r2.delete(&storage_key).await {
            tracing::warn!(photo_id = %photo_id, storage_key = %storage_key, error = ?e, "R2 photo delete failed; orphaning object");
        }
    }

    // Delete from DB
    db::lessons::delete_lesson_photo(&conn, &photo_id, &id, &user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}
