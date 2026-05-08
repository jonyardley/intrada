use libsql::Connection;

use intrada_core::domain::lesson::{Lesson, LessonPhoto};
use intrada_core::domain::types::{CreateLesson, UpdateLesson};
use intrada_core::validation;

use crate::db;
use crate::error::ApiError;
use crate::storage::R2Client;

/// Max photo upload size: 5 MB
pub const MAX_PHOTO_SIZE: usize = 5 * 1024 * 1024;

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

pub async fn list_lessons(
    conn: &Connection,
    r2: &R2Client,
    user_id: &str,
) -> Result<Vec<Lesson>, ApiError> {
    db::lessons::list_lessons(conn, user_id, r2).await
}

pub async fn get_lesson(
    conn: &Connection,
    r2: &R2Client,
    id: &str,
    user_id: &str,
) -> Result<Lesson, ApiError> {
    db::lessons::get_lesson(conn, id, user_id, r2)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Lesson not found: {id}")))
}

pub async fn create_lesson(
    conn: &Connection,
    r2: &R2Client,
    user_id: &str,
    input: &CreateLesson,
) -> Result<Lesson, ApiError> {
    validation::validate_create_lesson(input)?;
    db::lessons::insert_lesson(conn, user_id, input, r2).await
}

pub async fn update_lesson(
    conn: &Connection,
    r2: &R2Client,
    id: &str,
    user_id: &str,
    input: &UpdateLesson,
) -> Result<Lesson, ApiError> {
    validation::validate_update_lesson(input)?;
    db::lessons::update_lesson(conn, id, user_id, input, r2)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Lesson not found: {id}")))
}

pub async fn delete_lesson(
    conn: &Connection,
    r2: Option<&R2Client>,
    id: &str,
    user_id: &str,
) -> Result<(), ApiError> {
    // Delete photos from R2 if storage is configured. Log but don't fail
    // the request on R2 errors — DB is the source of truth.
    if let Some(r2) = r2 {
        let keys = db::lessons::list_photo_storage_keys(conn, id, user_id).await?;
        for key in keys {
            if let Err(e) = r2.delete(&key).await {
                tracing::warn!(lesson_id = %id, storage_key = %key, error = ?e, "R2 photo delete failed; orphaning object");
            }
        }
    }

    let deleted = db::lessons::delete_lesson(conn, id, user_id).await?;
    if !deleted {
        return Err(ApiError::NotFound(format!("Lesson not found: {id}")));
    }
    Ok(())
}

pub async fn upload_lesson_photo(
    conn: &Connection,
    r2: &R2Client,
    user_id: &str,
    lesson_id: &str,
    bytes: &[u8],
) -> Result<LessonPhoto, ApiError> {
    if bytes.len() > MAX_PHOTO_SIZE {
        return Err(ApiError::Validation(format!(
            "Photo exceeds maximum size of {} MB",
            MAX_PHOTO_SIZE / (1024 * 1024)
        )));
    }

    // Determine content-type from the bytes themselves — never trust the
    // client's multipart header. Prevents XSS via spoofed Content-Type on
    // the public R2 URL.
    let content_type = sniff_image_content_type(bytes)
        .ok_or_else(|| ApiError::Validation("Photo must be JPEG or PNG".into()))?;

    let photo_id = ulid::Ulid::new().to_string();
    let storage_key = R2Client::photo_key(user_id, lesson_id, &photo_id);
    r2.upload(&storage_key, bytes, content_type).await?;

    db::lessons::insert_lesson_photo(conn, lesson_id, user_id, &storage_key, r2).await
}

pub async fn delete_lesson_photo(
    conn: &Connection,
    r2: Option<&R2Client>,
    user_id: &str,
    lesson_id: &str,
    photo_id: &str,
) -> Result<(), ApiError> {
    let storage_key = db::lessons::get_lesson_photo_storage_key(conn, photo_id, lesson_id, user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Photo not found: {photo_id}")))?;

    // Delete from R2. Log but don't fail the request on R2 errors — the
    // DB row below is the authoritative pointer; orphaned R2 objects are
    // recoverable via prefix sweep but a stuck DB row is not.
    if let Some(r2) = r2 {
        if let Err(e) = r2.delete(&storage_key).await {
            tracing::warn!(photo_id = %photo_id, storage_key = %storage_key, error = ?e, "R2 photo delete failed; orphaning object");
        }
    }

    db::lessons::delete_lesson_photo(conn, photo_id, lesson_id, user_id).await?;
    Ok(())
}
